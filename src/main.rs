

use windows::{core::HSTRING, Devices::Bluetooth::BluetoothDevice, Networking::Sockets::StreamSocket, Storage::Streams::{Buffer, DataReader, DataWriter, InputStreamOptions}};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let args: Vec<String> = std::env::args().collect();
    
    let device_addr: u64 = u64::from_str_radix(&args[1], 16).unwrap();
    
    let device = BluetoothDevice::FromBluetoothAddressAsync(device_addr).unwrap().await.unwrap();
    println!("{:?}", device.Name().unwrap());
    let services = device.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap();
    for service in  &services{
        let service = service.ServiceId().unwrap().Uuid().unwrap();
        println!("{service:?}");
    }
    let hfp_service_check =  device.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap().GetAt(0).unwrap();
    println!("Service should start with: 111E");
    println!("{:?}", hfp_service_check.ServiceId().unwrap().Uuid().unwrap());
    let service = device.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap().GetAt(0).unwrap();
    
    let socket = StreamSocket::new().unwrap();
    socket.Control().unwrap().NoDelay().unwrap();
    
    println!("{:?} - {:?}", &service.ConnectionHostName().unwrap(), &service.ConnectionServiceName().unwrap());
    let result = socket.ConnectAsync(&service.ConnectionHostName().unwrap(), &service.ConnectionServiceName().unwrap());
    match result {
        Ok(action) => {
            action.await.unwrap();
            println!("Connected");
            
        }
        Err(e) => {
            println!("oopsies");
            println!("{e:?}");
            return Ok(())
        }
    }
    
    
    // println!("Creating write buffer for command");
    // let string_to_send = HSTRING::from("AT+CIND=?");
    
    // let writer = DataWriter::new().unwrap();
    // println!("Sending command: {}", &string_to_send);
    // writer.WriteString(&string_to_send).unwrap();
    // let write_buffer = writer.DetachBuffer().unwrap();
    // socket.OutputStream().unwrap().WriteAsync(&write_buffer).unwrap().await.unwrap();
    
    println!("Listening to input stream");
    let read_buffer = Buffer::Create(256).unwrap();
    let something = socket.InputStream().unwrap().ReadAsync(&read_buffer, 16, InputStreamOptions::Partial).unwrap().await.unwrap();

    println!("Reading returned buffer");
    let reader = DataReader::FromBuffer(&something).unwrap();
    
    while reader.UnconsumedBufferLength().unwrap() != 0 {
        println!("Reading at: {}", reader.UnconsumedBufferLength().unwrap());
        let stuff = reader.ReadString(reader.UnconsumedBufferLength().unwrap()).unwrap();
        print!("- {}", stuff);
    }
    println!("Done");
    
    Ok(())
}