use windows::{core::HSTRING, Devices::Bluetooth::BluetoothDevice, Networking::Sockets::StreamSocket, Storage::Streams::{Buffer, DataReader, DataWriter, IBuffer, InputStreamOptions}};

/*
    https://inthehand.com/2022/12/30/12-days-of-bluetooth-10-hands-free/

*/

static WRITE_COMMANDS: [&str; 5] = [
    "BRSF:0",
    "+CIND: (\"service\",(0,1)),(\"call\",(0,1))",
    "+CIND: 1,0",
    "+CHLD: 0",
    "+XAPL=iPhone,6"
];

static READ_COMMANDS: [&str; 6] = [
    "AT+BRSF",
    "AT+CIND=?",
    "AT+CIND?",
    "AT+CHLD=?",
    "AT+XAPL",
    "AT+IPHONEACCEV"
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter device bluetooth address and press enter:");
    let mut device_addr_string = String::new();
    std::io::stdin().read_line(&mut device_addr_string).unwrap();
    let device_addr: u64 = u64::from_str_radix(&device_addr_string.trim(), 16).unwrap();
    
    let device = BluetoothDevice::FromBluetoothAddressAsync(device_addr).unwrap().await.unwrap();
    let services = device.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap();
    for (index, service) in services.into_iter().enumerate() {
        println!("Service at index: {} -- {}", index, service.ConnectionServiceName().unwrap())
    }
    println!("Type in the index of the one with 111E: ");
    let mut device_hfp_service_select = String::new();
    std::io::stdin().read_line(&mut device_hfp_service_select).unwrap();
    let service_index = device_hfp_service_select.trim().parse::<u32>().unwrap();
    let hfp_service = device.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap().GetAt(service_index).unwrap();
    let socket = StreamSocket::new().unwrap();
    socket.Control().unwrap().NoDelay().unwrap(); // test KeepAlive

    let result = socket.ConnectAsync(&hfp_service.ConnectionHostName().unwrap(), &hfp_service.ConnectionServiceName().unwrap());
    
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
    init_bluetooth_communication(&socket).await;
    let mut last_command_sent = false;
    loop {
        if last_command_sent {
            send_response(WRITE_COMMANDS[4], &socket, true).await;
            last_command_sent = false;
        }
        println!("---- Listening ----");
        let read_buffer = Buffer::Create(1024).unwrap();
        let input_buffer = socket.InputStream().unwrap().ReadAsync(&read_buffer, 32, InputStreamOptions::Partial).unwrap().await.unwrap();

        let read_result = read_input_buffer(input_buffer);
        println!("Response: {}", read_result);
        let mut found_handled_command: Option<usize> = None;
        for (index, command) in READ_COMMANDS.iter().enumerate() {
            if read_result.starts_with(command) {
                println!("Found command {} at index: {}", command, index);
                found_handled_command = Some(index);
            }
        }
  
        match found_handled_command {
            Some(index) => {
                match index {
                    // battery info
                    5 => {
                        println!("Should be battery info");
                        println!("*****");
                        println!("Battery percent: {}%", convert_to_battery_percentage(&read_result));
                        println!("*****");
                        send_response("OK", &socket, false).await;
                    }
                    0 => {
                        send_response(&read_result, &socket, true).await;
                    }
                    // default response
                    _ => {
                        send_response(WRITE_COMMANDS[index], &socket, true).await;
                    }   
                }
            }
            None => {
                send_response("OK", &socket, false).await;
            }
            
        }
        
    }
    // Ok(())
}

async fn init_bluetooth_communication(socket: &StreamSocket) {
    let start_cmd = HSTRING::from("AT+CIND?");
    let writer = DataWriter::new().unwrap();
    println!("Sending starting command: {}", &start_cmd);
    
    writer.WriteString(&start_cmd).unwrap();
    let write_buffer = writer.DetachBuffer().unwrap();
    socket.OutputStream().unwrap().WriteAsync(&write_buffer).unwrap().await.unwrap();
}

async fn send_response(res: &str, socket: &StreamSocket, send_extra_ok: bool) {
    println!("- Writing: {}", res);
    let cmd_write_buffer = create_write_command_buffer(res);
    socket.OutputStream().unwrap().WriteAsync(&cmd_write_buffer).unwrap().await.unwrap();
    if send_extra_ok {
        let ok_response_buffer = create_write_command_buffer("OK");
        socket.OutputStream().unwrap().WriteAsync(&ok_response_buffer).unwrap().await.unwrap();
    }
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
    
    stuff.to_string()
}
fn convert_to_battery_percentage(res: &str) -> String {
    let mut result: u8 = 0;
    // AT+IPHONEACCEV=2,1,8,2,0
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
    result.to_string()
}


#[cfg(test)]
mod tests {
    // sending only OK until AT+IPHONEACCEV
    // sending +XAPL=iPhone,2 first
    // after OKing/responding, read in a loop until +CIEV or fail
}