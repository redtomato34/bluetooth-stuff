
use bluest::{Adapter, Uuid};
use futures::FutureExt;



pub async fn get_bluetooth_adapter() -> Option<Adapter>{
    let adapter = Adapter::default().await.unwrap();
    match adapter {
        Ok(a) => Some(a),
        Err(_) => None
    }
}
pub async fn find_devices_with_adapter(adapter: &mut Option<Adapter>) -> Result<(), ()>{
    let battery_service_uuid = Uuid::try_parse("0000180F-0000-1000-8000-00805f9b34fb").unwrap(); // bluetooth assigned numbers 3.4.1     
    
    match adapter {
        Some(x) => {
            x.wait_available().now_or_never();
            println!("Checking devices");
            let devices = x.connected_devices_with_services(&[battery_service_uuid]).await;
            println!("{:?}", devices);
            // match devices {
            //     Ok(devices) => {
            //         println!("getting connected devices");
                    
            //         if devices.len() == 0 {
            //             return Err(());
            //         }
            //         for device in devices {
            //             let device_connected = device.is_connected().await;
            //             if device_connected {
            //                 println!("found {:?}", device);
                            
                            
            //                 // println!("Hello");
            //                 // match device_with_battery_service {
            //                 //     Ok(services) => {
            //                 //         println!("{:?}", &services);
            //                 //         let characteristics = services.get(0).unwrap().characteristics().await.unwrap();
                                    
            //                 //         println!("{:?}", characteristics.get(0).unwrap().read().await.unwrap().get(0));
            //                 //     }
            //                 //     Err(_) => {
            //                 //         println!("No battery service found");
            //                 //         return Err(())
            //                 //     }
            //                 // }
            //             } else {
            //                 println!("Device: {:?} not connected", device.name().unwrap());
            //                 return Err(())
            //             }
            //         }
            //     }
            //     Err(_) => {
            //         println!("Oops");
            //         return Err(())
            //     }
            // }
        }
        None => {
            return Err(())
        }
    }
    
    
    
    // let devices = adapter.as_ref().unwrap().connected_devices().await.unwrap();
    // if devices.len() == 0 {
    //     return Err(())
    // }
    // for device in devices {
        
    //     let services = device.services().await.unwrap();
    //     for service in services {
    //         let service_as_byte_array = *service.uuid().as_bytes();
    //         if service_as_byte_array[2] == 0x18 && service_as_byte_array[3] == 0x0F {
    //             println!("Battery service found");
    //             println!("*{:?}", &service);
    //             let characteristics = service.characteristics().await.unwrap();
    //             for characteristic in characteristics {
    //                 println!("**{:?}", characteristic);
    //                 println!("{}", characteristic.uuid());
    //                 let props = characteristic.properties().await.unwrap();
    //                 println!("***props: {:?}", props);
    //                 if props.read {
    //                     println!("****value: {:?}", characteristic.read().await);
    //                 }
    //                 let descriptors = characteristic.descriptors().await.unwrap();
    //                 for descriptor in descriptors {
    //                     println!("*****{:?}: {:?}", descriptor, descriptor.read().await);
    //                 }
    //             }
    //         } else {
    //             println!("*{:?}", &service);
    //         };
    //     }
    // }
    Ok(())
}
