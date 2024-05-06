use windows::Devices::Bluetooth::BluetoothDevice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let device_address = u64::from_str_radix(&args[1], 16).unwrap();
    let yep3 = BluetoothDevice::FromBluetoothAddressAsync(device_address).unwrap().await.unwrap();
    let yep4 = yep3.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap();
    for service in yep4 {
        let yep5 = service.ServiceId().unwrap().AsString().unwrap();
        println!("{:?}", yep5);
    }
    Ok(())
}