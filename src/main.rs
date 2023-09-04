// fn main() {
//     for device in rusb::devices().unwrap().iter() {
//         let device_desc = device.device_descriptor().unwrap();

//         println!(
//             "Bus {:03} Device {:03} ID {:04x}:{:04x}",
//             device.bus_number(),
//             device.address(),
//             device_desc.vendor_id(),
//             device_desc.product_id()
//         );
//     }
// }

// use rusb::{Context, Device, HotplugBuilder, UsbContext};

// struct HotPlugHandler;

// impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
//     fn device_arrived(&mut self, device: Device<T>) {
//         println!("device arrived {:?}", device);
//     }

//     fn device_left(&mut self, device: Device<T>) {
//         println!("device left {:?}", device);
//     }
// }

// impl Drop for HotPlugHandler {
//     fn drop(&mut self) {
//         println!("HotPlugHandler dropped");
//     }
// }

// fn main() -> rusb::Result<()> {
//     if rusb::has_hotplug() {
//         let context = Context::new()?;

//         let mut reg = Some(
//             HotplugBuilder::new()
//                 .vendor_id(0x_0e4c)
//                 .enumerate(true)
//                 .register(&context, Box::new(HotPlugHandler {}))?,
//         );

//         loop {
//             context.handle_events(None).unwrap();
//             if let Some(reg) = reg.take() {
//                 context.unregister_callback(reg);
//                 // break;
//             }
//         }
//         Ok(())
//     } else {
//         eprint!("libusb hotplug api unsupported");
//         Ok(())
//     }
// }

use std::time::Duration;

use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};

fn main() {
    let mut ctx = Context::new().unwrap();
    let (device, device_desc, mut handle) = open_device(&mut ctx, 0x0e4c, 0x7288).unwrap();

    // dbg!(handle.kernel_driver_active(0));

    dbg!(device.get_parent());

    let config_desc = device.config_descriptor(0).unwrap();

    let [interface]: [_; 1] = config_desc
        .interfaces()
        .flat_map(|i| i.descriptors().collect::<Vec<_>>())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    dbg!(&interface);

    let [endpoint]: [_; 1] = interface
        .endpoint_descriptors()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    dbg!(&endpoint);

    handle
        .set_active_configuration(dbg!(config_desc.number()))
        .unwrap();
    handle
        .claim_interface(dbg!(interface.interface_number()))
        .unwrap();
    handle
        .set_alternate_setting(
            interface.interface_number(),
            dbg!(interface.setting_number()),
        )
        .unwrap();

    loop {
        let mut buf = [0; 256];
        let timeout = Duration::from_secs(5);

        let bytes_read = handle.read_interrupt(0x81, &mut buf, timeout);

        println!("read {bytes_read:?} bytes");

        dbg!(u64::from_le_bytes(buf[0..8].try_into().unwrap()));
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            println!("found device {device_desc:#?}");
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => panic!("Device found but failed to open: {}", e),
            }
        }
    }

    None
}
