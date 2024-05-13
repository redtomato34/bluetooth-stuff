

use windows::{core::HSTRING, Devices::Bluetooth::BluetoothDevice, Networking::Sockets::StreamSocket, Storage::Streams::{Buffer, DataReader, DataWriter, IBuffer, InputStreamOptions}};
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
    // https://inthehand.com/2022/12/30/12-days-of-bluetooth-10-hands-free/
    let write_commands = [
        "BRSF:0",
        "AT+CIND?",
        "+CIND: (\"service\",(0,1)),(\"call\",(0,1))",
        "+CIND: 1,0",
        "+CHLD: 0",
        "+XAPL=iPhone,2"
    ];
    let read_commands = [
        "AT+BRSF",
        "AT+CIND=?",
        "AT+CIND?",
        "AT+CHLD=?",
        "AT+XAPL",
        "AT+IPHONEACCEV"
    ];
    
    println!("Starting");
    let start_cmd = HSTRING::from("AT+CIND?");
    
    let writer = DataWriter::new().unwrap();
    println!("Sending command: {}", &start_cmd);
    writer.WriteString(&start_cmd).unwrap();
    let write_buffer = writer.DetachBuffer().unwrap();
    socket.OutputStream().unwrap().WriteAsync(&write_buffer).unwrap().await.unwrap();
    

    let mut iteration = 0;
    loop {
        println!("iteration: {}", iteration);
        println!("Listening to input stream");
        let read_buffer = Buffer::Create(1024).unwrap();
        let something = socket.InputStream().unwrap().ReadAsync(&read_buffer, 32, InputStreamOptions::Partial).unwrap().await.unwrap();
        println!("Reading returned buffer");
        
        let read_result = read_input_buffer(something);
        if read_result.starts_with(read_commands[0]) {
            println!("Found: {}", read_result);
            send_response(write_commands[0], &socket, true).await;
            
        } else if read_result.starts_with(read_commands[1]) {
            println!("Found: {}", read_result);
            send_response(write_commands[1], &socket, true).await;

        } else if read_result.starts_with(read_commands[2]) {
            println!("Found: {}", read_result);
            send_response(write_commands[2], &socket, true).await;

        } else if read_result.starts_with(read_commands[3]) {
            println!("Found: {}", read_result);
            send_response(write_commands[3], &socket, true).await;

        } else if read_result.starts_with(read_commands[4]) {
            println!("Found: {}", read_result);
            send_response(write_commands[4], &socket, true).await;

        } else if read_result.starts_with(read_commands[5]) {
            println!("Should be battery info");
            println!("Found: {}", read_result);
            println!("------");
            println!("Battery percent: {}", convert_to_battery_percentage(&read_result));
            println!("------");
            send_response("OK", &socket, false).await;

        } else {
            println!("Command didn't match: {}", read_result);
            send_response("OK", &socket, false).await;
        }
        iteration += 1;
    }
    Ok(())
}

async fn send_response(res: &str, socket: &StreamSocket, cmd_found: bool) {
    let cmd_write_buffer = create_write_command_buffer(res);
    socket.OutputStream().unwrap().WriteAsync(&cmd_write_buffer).unwrap().await.unwrap();
    if cmd_found {
        let ok_response_buffer = create_write_command_buffer("OK");
        socket.OutputStream().unwrap().WriteAsync(&ok_response_buffer).unwrap().await.unwrap();
    }
    println!("Response sent");
}


fn create_write_command_buffer(cmd: &str) -> IBuffer {
    let writer = DataWriter::new().unwrap();
    let formatted_command = format!("\r\n{}\r\n", cmd);
    let cmd_hstring = HSTRING::from(formatted_command);
    writer.WriteString(&cmd_hstring).unwrap();
    writer.DetachBuffer().unwrap()
}
fn read_input_buffer(buffer: IBuffer) -> String {
    let reader = DataReader::FromBuffer(&buffer).unwrap();
    let stuff = reader.ReadString(reader.UnconsumedBufferLength().unwrap()).unwrap();
    print!("Read: {}", stuff);
    stuff.to_string()
}
fn convert_to_battery_percentage(res: &str) -> String {
    let mut result: u8 = 0;

    let res_split: Vec<&str> = res.split("=").collect();
    let bat_data: Vec<&str> = res_split.get(1).unwrap().split(",").collect();
    for index in 0..bat_data.len() {
        let index = index as u8;
        let key = bat_data[(index * 2 + 1) as usize].parse::<u8>().unwrap();
        let value = bat_data[(index * 2 + 2) as usize].parse::<u8>().unwrap();
        if key == 1 {
            println!("Converted");
            result = (value + 1) * 10;
            break;
        }
       
    }
    format!("{}%", result)
}


#[cfg(test)]
mod tests {
    // sending only OK until AT+IPHONEACCEV
    // sending +XAPL=iPhone,2 first
    // after OKing/responding, read in a loop until +CIEV or fail
}