extern crate libusb;

use std::time::Duration;

struct UsbDevice<'a> {
    handle: libusb::DeviceHandle<'a>,
    language: libusb::Language,
    timeout: Duration
}


fn main() {
    list_devices().unwrap();
}

fn list_devices() -> libusb::Result<()> {
    let timeout = Duration::from_secs(1);

    let context = try!(libusb::Context::new());

    for device in try!(context.devices()).iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue
        };

        let mut usb_device = {
            match device.open() {
                Ok(h) => {
                    match h.read_languages(timeout) {
                        Ok(l) => {
                            if l.len() > 0 {
                                Some(UsbDevice {
                                    handle: h,
                                    language: l[0],
                                    timeout: timeout
                                })
                            }
                            else {
                                None
                            }
                        },
                        Err(_) => None
                    }
                },
                Err(_) => None
            }
        };

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x} {}", device.bus_number(), device.address(), device_desc.vendor_id(), device_desc.product_id(), get_speed(device.speed()));
        print_device(&device_desc, &mut usb_device);

        print_bos(&mut usb_device);

        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue
            };

            print_config(&config_desc, &mut usb_device);

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    print_interface(&interface_desc, &mut usb_device);

                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        print_endpoint(&endpoint_desc);
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_device(device_desc: &libusb::DeviceDescriptor, handle: &mut Option<UsbDevice>) {
    println!("Device Descriptor:");
    println!("  bcdUSB             {:2}.{}{}", device_desc.usb_version().major(), device_desc.usb_version().minor(), device_desc.usb_version().sub_minor());
    println!("  bDeviceClass        {:#04x}", device_desc.class_code());
    println!("  bDeviceSubClass     {:#04x}", device_desc.sub_class_code());
    println!("  bDeviceProtocol     {:#04x}", device_desc.protocol_code());
    println!("  bMaxPacketSize0      {:3}", device_desc.max_packet_size());
    println!("  idVendor          {:#06x}", device_desc.vendor_id());
    println!("  idProduct         {:#06x}", device_desc.product_id());
    println!("  bcdDevice          {:2}.{}{}", device_desc.device_version().major(), device_desc.device_version().minor(), device_desc.device_version().sub_minor());
    println!("  iManufacturer        {:3} {}",
             device_desc.manufacturer_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_manufacturer_string(h.language, device_desc, h.timeout).unwrap_or(String::new())));
    println!("  iProduct             {:3} {}",
             device_desc.product_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_product_string(h.language, device_desc, h.timeout).unwrap_or(String::new())));
    println!("  iSerialNumber        {:3} {}",
             device_desc.serial_number_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_serial_number_string(h.language, device_desc, h.timeout).unwrap_or(String::new())));
    println!("  bNumConfigurations   {:3}", device_desc.num_configurations());
}

fn print_bos(UsbDevice: &mut Option<UsbDevice>) {
    let h = match UsbDevice {
        Some(h) => &h.handle,
        None => return,
    };
    let bos_desc = match h.bos_descriptor() {
            Ok(desc) => desc,
            Err(_) => { return; },

    };

    println!("  BOS Descriptor:");
    println!("    bDescriptorType      {}", bos_desc.descriptor_type());
    println!("    bNumDeviceCaps       {}", bos_desc.num_device_caps());

    for dev_cap in bos_desc.dev_capability() {
        match dev_cap.dev_capability_type() {
            2 => {
                match h.bos_usb_2_0_extension_descriptor(&bos_desc, &dev_cap) {
                    Ok(desc) => {
                        println!("    USB 2.0 Extension Capabilities:");
                        println!("      bDevCapabilityType:    {}", desc.dev_capability_type());
                        println!("      bmAttributes           {:08x}h", desc.attributes());
                    },
                    Err(e) => println!("{}", e),
                }
            },
            3 => {
                match h.bos_superspeed_usb_descriptor(&bos_desc, &dev_cap) {
                    Ok(desc) => {
                        println!("    USB 3.0 Capabilities:");
                        println!("      bDevCapabilityType:    {}", desc.dev_capability_type());
                        println!("      bmAttributes:          {:02x}h", desc.attributes());
                        println!("      wSpeedSupported:       {}", desc.speed_supported());
                        println!("      bFunctionalitySupport: {}", desc.functionality_support());
                        println!("      bU1devExitLat:         {}", desc.u1_dev_exit_lat());
                        println!("      bU2devExitLat:         {}", desc.u2_dev_exit_lat());
                    },
                    Err(e) => println!("{}", e),
                }
            },
            4 => {
                match h.bos_container_id_descriptor(&bos_desc, &dev_cap) {
                    Ok(desc) => {
                        print!("    Container ID:            ");
                        for i in desc.container_id() {
                            print!("{:02X}", i);
                        }
                        println!();
                    },
                    Err(e) => println!("{}", e),
                }
            },
            _ => {},
        }
    }
}

fn print_config(config_desc: &libusb::ConfigDescriptor, handle: &mut Option<UsbDevice>) {
    println!("  Config Descriptor:");
    println!("    bNumInterfaces       {:3}", config_desc.num_interfaces());
    println!("    bConfigurationValue  {:3}", config_desc.number());
    println!("    iConfiguration       {:3} {}",
             config_desc.description_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_configuration_string(h.language, config_desc, h.timeout).unwrap_or(String::new())));
    println!("    bmAttributes:");
    println!("      Self Powered     {:>5}", config_desc.self_powered());
    println!("      Remote Wakeup    {:>5}", config_desc.remote_wakeup());
    println!("    bMaxPower           {:4}mW", config_desc.max_power());
}

fn print_interface(interface_desc: &libusb::InterfaceDescriptor, handle: &mut Option<UsbDevice>) {
    println!("    Interface Descriptor:");
    println!("      bInterfaceNumber     {:3}", interface_desc.interface_number());
    println!("      bAlternateSetting    {:3}", interface_desc.setting_number());
    println!("      bNumEndpoints        {:3}", interface_desc.num_endpoints());
    println!("      bInterfaceClass     {:#04x}", interface_desc.class_code());
    println!("      bInterfaceSubClass  {:#04x}", interface_desc.sub_class_code());
    println!("      bInterfaceProtocol  {:#04x}", interface_desc.protocol_code());
    println!("      iInterface           {:3} {}",
             interface_desc.description_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_interface_string(h.language, interface_desc, h.timeout).unwrap_or(String::new())));
}

fn print_endpoint(endpoint_desc: &libusb::EndpointDescriptor) {
    println!("      Endpoint Descriptor:");
    println!("        bEndpointAddress    {:#04x} EP {} {:?}", endpoint_desc.address(), endpoint_desc.number(), endpoint_desc.direction());
    println!("        bmAttributes:");
    println!("          Transfer Type          {:?}", endpoint_desc.transfer_type());
    println!("          Synch Type             {:?}", endpoint_desc.sync_type());
    println!("          Usage Type             {:?}", endpoint_desc.usage_type());
    println!("        wMaxPacketSize    {:#06x}", endpoint_desc.max_packet_size());
    println!("        bInterval            {:3}", endpoint_desc.interval());
}

fn get_speed(speed: libusb::Speed) -> &'static str {
    match speed {
        libusb::Speed::Super   => "5000 Mbps",
        libusb::Speed::High    => " 480 Mbps",
        libusb::Speed::Full    => "  12 Mbps",
        libusb::Speed::Low     => " 1.5 Mbps",
        libusb::Speed::Unknown => "(unknown)"
    }
}
