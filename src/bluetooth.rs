use std::time::Duration;
use chrono::Local;
use tokio::time::sleep;
use futures::executor::block_on;
use windows::{core::HSTRING, Devices::{Bluetooth::{BluetoothDevice, Rfcomm::RfcommDeviceService}, Enumeration::DeviceInformation}, Networking::Sockets::StreamSocket, Storage::Streams::{Buffer, DataReader, DataWriter, IBuffer, InputStreamOptions}};

use crate::{util::{READ_COMMANDS, WRITE_COMMANDS}, BluetoothInfo};

pub async fn run_bluetooth_thread(info: BluetoothInfo) -> tokio::task::JoinHandle<Result<(), ()>> {
    tokio::spawn( async {
        block_on(run_bluetooth(info));
        Ok(())
    })}


async fn run_bluetooth(info: BluetoothInfo) {
    let bt_device_aqs_filter = BluetoothDevice::GetDeviceSelector().unwrap();
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&bt_device_aqs_filter).unwrap().await.unwrap();
    let mut devices_with_hfp_service: Vec<Option<RfcommDeviceService>> = Vec::new();

    for device in devices {
        let device_id = device.Id().unwrap();
        let bt_device = BluetoothDevice::FromIdAsync(&device_id).unwrap().await;

        match bt_device {
            Ok(e) => {
                let status = e.ConnectionStatus().unwrap().0;
                if status == 0 {
                    continue;
                }
                let services = e.GetRfcommServicesAsync().unwrap().await.unwrap().Services().unwrap();
                for service in services {
                    let stuff = service.ConnectionServiceName().unwrap();
                    if stuff.to_string().contains("111e") {
                        devices_with_hfp_service.push(Some(service));
                    }
                }
            }
            Err(_) => {
                println!("Bluetooth is off");
                return;
            }
        }
    }
    if devices_with_hfp_service.is_empty() {
        println!("No bluetooth devices found");
        return;
    }
    let hfp_device: RfcommDeviceService = devices_with_hfp_service.get(0).unwrap().clone().unwrap();
    {
        let mut guard = info.message.lock().await;
        *guard = Some(format!("{}", hfp_device.Device().unwrap().DeviceInformation().unwrap().Name().unwrap()))
    }
    let socket = StreamSocket::new().unwrap();
    let result = socket.ConnectAsync(&hfp_device.ConnectionHostName().unwrap(), &hfp_device.ConnectionServiceName().unwrap());
    
    match result {
        Ok(action) => {
            action.await.unwrap();
            println!("Connected");
        }
        Err(e) => {
            println!("oopsies");
            println!("{e:?}");
            return;
        }
    }
    init_bluetooth_communication(&socket).await;

    loop {
        let read_buffer = Buffer::Create(1024).unwrap();
        let input_buffer = socket.InputStream();

        match input_buffer {
            Ok(stream) => {
                let mut found_handled_command: Option<usize> = None;                                    
                
                let buffer = stream.ReadAsync(&read_buffer, 32, InputStreamOptions::Partial);
                match buffer {
                    Ok(e)=> {
                        let read_result = read_input_buffer(e.await.unwrap()); 
                    
                        println!("Reading: {}", read_result);
                        
                        for (index, command) in READ_COMMANDS.iter().enumerate() {
                            if read_result.starts_with(command) {
                                // println!("Found command {} at index: {}", command, index);
                                found_handled_command = Some(index);
                                break;
                            } else if read_result.is_empty() {
                                println!("Empty");
                                found_handled_command = Some(6);
                                break;
                            }
                        }
                        
                        match found_handled_command {
                            Some(index) => {
                                match index {
                                    // battery info
                                    5 => {
                                        let timestamp = Local::now().time();
                                        // println!("Should be battery info");
                                        println!("{:?}", timestamp);
                                        println!("Battery percent: {}%", convert_to_battery_percentage(&read_result));

                                        send_response("OK", &socket, false).await;
                                        // update_battery()
                                        // socket.Close().unwrap();
                                    }
                                    6 => {
                                        return;
                                    }
                                    // 0 => {
                                    //     send_response(&read_result, &socket, true).await;
                                    // }
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
                    Err(e) => {
                        println!("{}", e);
                        break;
                    }
                    }
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
    println!("Done");
}

async fn init_bluetooth_communication(socket: &StreamSocket) {
    let start_cmd = HSTRING::from("AT+CIND?");
    let writer = DataWriter::new().unwrap();
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
fn convert_to_battery_percentage(res: &str) -> u8 {
    let mut result: u8 = 0;
    // AT+IPHONEACCEV=2,1,8,2,0
    let res_split: Vec<&str> = res.split("=").collect();
    let bat_data: Vec<&str> = res_split.get(1).unwrap().split(",").collect();
    for index in 0..(bat_data[0].parse::<u8>().unwrap()) {
        let index = index as u8;
        let key = bat_data[(index * 2 + 1) as usize].trim().parse::<u8>().unwrap();
        let value = bat_data[(index * 2 + 2) as usize].trim().parse::<u8>().unwrap();
        
        if key == 1 {
            // println!("Converted");
            result = (value + 1) * 10;
            break;
        }
    }
    result
}