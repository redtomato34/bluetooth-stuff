use std::{sync::Arc, time::Duration};
use chrono::{Local, Utc};
use tokio::time::sleep;
use tray_icon::Icon;
use futures::{executor::block_on, lock::Mutex};
use windows::{core::HSTRING, Devices::{Bluetooth::{BluetoothDevice, Rfcomm::RfcommDeviceService}, Enumeration::DeviceInformation}, Networking::Sockets::StreamSocket, Storage::Streams::{Buffer, DataReader, DataWriter, IBuffer, InputStreamOptions}};

use crate::util::{load_icons, READ_COMMANDS, WRITE_COMMANDS};

#[derive(Clone)]
pub struct BluetoothInfo {
    pub adapter_is_on: Arc<Mutex<bool>>,
    pub connected_device: Arc<Mutex<Option<DeviceInfo>>>,
    pub message: Arc<Mutex<Option<String>>>
}
pub struct DeviceInfo {
    pub device_name: String,
    pub battery_level: Option<u8> ,
    pub battery_icon: Option<Icon>,
    pub checked_timestamp: i64, 
}
impl DeviceInfo {
    pub fn init(device_name: String, battery_level: Option<u8>, battery_icon: Option<Icon>, checked_timestamp: i64) -> Self {
        DeviceInfo {
            device_name,
            battery_level,
            battery_icon,
            checked_timestamp
        }
    }
    pub fn set_battery(&mut self, battery_level: u8) {
        let icons = load_icons().unwrap();
        let icon_index = (100 - battery_level) / 20;
        let icon: &Icon = icons.get(icon_index as usize).unwrap();
        self.battery_level = Some(battery_level);
        self.battery_icon = Some(icon.clone());
    }
}
// todo: later
// pub enum DeviceType {
//     Headset
// }
// impl From<HSTRING> for DeviceType {
//     fn from(value: HSTRING) -> Self {
//         todo!()
//     }
// }

pub async fn run_bluetooth_thread(info: BluetoothInfo) -> tokio::task::JoinHandle<Result<(), ()>> {
    tokio::spawn( async move {
        loop {
            let bt_info = &info;
            block_on(run_bluetooth(bt_info.clone()));
            sleep(Duration::from_secs(5)).await;
        }
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
                        {
                            let mut guard = info.connected_device.lock().await;
                            *guard = None;
                            let device_info = DeviceInfo::init(e.DeviceInformation().unwrap().Name().unwrap().to_string(), None, None, Utc::now().timestamp().try_into().unwrap());
                            *guard = Some(device_info);
                        }
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
            match action.await {
                Ok(_) => {
                    init_bluetooth_communication(&socket).await;
                }
                Err(_) => {
                    {
                        let mut guard = info.connected_device.lock().await;
                        *guard = None;
                    }
                    println!("Oops")
                }
            };
            // println!("Connected");
        }
        Err(e) => {
            println!("oopsies");
            println!("{e:?}");
            return;
        }
    }
    

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
                    
                        // println!("Reading: {}", read_result);
                        
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
                                        send_response("OK", &socket, false).await;
                                        {
                                            let timestamp = Utc::now();
                                            let mut guard = info.connected_device.lock().await;
                                            guard.as_mut().unwrap().checked_timestamp = timestamp.timestamp();
                                            guard.as_mut().unwrap().set_battery(convert_to_battery_percentage(&read_result));
                                        }
                                    }
                                    // device disconnected
                                    6 => {
                                        {
                                            let mut guard = info.connected_device.lock().await;
                                            *guard = None;
                                        }
                                        return;
                                    }
                                    // default handled response
                                    _ => {
                                        send_response(WRITE_COMMANDS[index], &socket, true).await;
                                    }   
                                }
                            }
                            // unhandled response
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
                {
                    let mut guard = info.connected_device.lock().await;
                    *guard = None;
                }
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
            result = (value + 1) * 10;
            break;
        }
    }
    result
}