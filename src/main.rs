/*
    Battery icons created by Stockio - Flaticon
    https://www.flaticon.com/authors/stockio


*/
pub mod util;
use std::{process::Command, sync::{Arc, Mutex}, thread::JoinHandle, time::{Duration, Instant}};


use bluest::{Adapter, AdapterEvent, Uuid};
use futures::{executor::block_on, StreamExt};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tao::{event::{Event, StartCause}, event_loop::{ControlFlow, EventLoopBuilder}};


use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem}, Icon, TrayIcon, TrayIconBuilder, TrayIconEvent
};
use util::structs::{BluetoothDevices, DisplayDeviceInformation};

use crate::util::{image::load_icons, structs::BluetoothDevice};

const SHORT_TIMER: Duration = Duration::from_secs(2);
#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfo {
    device_name: String,
    battery_info: u8
}
#[derive(Serialize, Deserialize, Debug)]

struct BluetoothDevices2 {
    device: Vec<DeviceInfo>
}

#[derive(Clone)]
pub struct BluetoothInfo {
    adapter_on: Arc<Mutex<bool>>,
    devices: Arc<Mutex<BluetoothDevices>>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bluetooth = BluetoothInfo { adapter_on: Arc::new(Mutex::new(false)), devices: Arc::new(Mutex::new(BluetoothDevices {devices: Some(Vec::new())})) };
    let bluetooth_is_on_render = bluetooth.adapter_on.clone();
    let bluetooth_is_on_checker = bluetooth.adapter_on.clone();
    
    
    let bluetooth_thread_handle = run_bluetooth_loop(&bluetooth_is_on_checker).await;
    
    run_render_loop(&bluetooth_is_on_render, &bluetooth.devices).await;
    bluetooth_thread_handle.abort();
    Ok(())
}
async fn run_bluetooth_loop(adapter_on: &Arc<Mutex<bool>>) -> tokio::task::JoinHandle<()> {
    let adapter_on = adapter_on.clone();
    tokio::spawn(async move {
        let adapter = Adapter::default().await.unwrap();
        match adapter.wait_available().await {
            Ok(()) => {
                let mut bluetooth_adapter_on_guard = adapter_on.lock().unwrap();
                *bluetooth_adapter_on_guard = true;
            }
            Err(_) => {
                
            }
        };
        let mut yep = adapter.events().await.unwrap();
        
        while let next_yep = yep.next().await {
            match next_yep.unwrap().unwrap() {
                AdapterEvent::Available => {
                    {
                        let mut bluetooth_adapter_on_guard = adapter_on.lock().unwrap();
                        *bluetooth_adapter_on_guard = true;
                    }
                    println!("Adapter turned on");
                }
                AdapterEvent::Unavailable => {
                    {
                        let mut bluetooth_adapter_on_guard = adapter_on.lock().unwrap();
                        *bluetooth_adapter_on_guard = false;
                    }
                    
                    println!("Adapter turned off");
                }
            }
        }
    })
}

fn query_bluetooth_devices() -> Vec<DeviceInfo> {
    let mut binding = Command::new("powershell.exe");
    let cmd = binding
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "ByPass",
                "-File",
                "./scripts/script.ps1",
            ]);
    let v: Vec<DeviceInfo> = serde_json::from_str(&String::from_utf8_lossy(&cmd.output().unwrap().stdout)).unwrap();
    return v;
}

async fn run_render_loop(adapter_on: &Arc<Mutex<bool>>, bluetooth_devices: &Arc<std::sync::Mutex<BluetoothDevices>>) {
    let adapter_on = adapter_on.clone();
    
    let current_devices = bluetooth_devices.clone();
    let mut selected_device: Option<BluetoothDevice> = None;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "\\icons\\");
    let event_loop = EventLoopBuilder::new().build();
    let adapter = Adapter::default().await.unwrap();
    
    let mut battery_index: usize = 0;
    let tray_battery_icons = load_icons(std::path::Path::new(path));
    let mut tray_menu = Menu::new();
    
    let quit_i = MenuItem::new("Quit", true, None);
    let menu_item = MenuItem::with_id(0, "Yep", true, None);
    tray_menu.append_items(&[
        &menu_item,
        &quit_i,
    ]);

    let mut tray_icon_app = None;
    
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    
    event_loop.run(move |event, _, control_flow| {
        
        if let tao::event::Event::NewEvents(tao::event::StartCause::Init) = event {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_secs(2));
            
        }
        match event {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + SHORT_TIMER);
                
                let default_icon = tray_battery_icons.as_ref().expect("yep").get(5).unwrap();
               
                
                tray_icon_app = Some(
                            TrayIconBuilder::new()
                            .with_menu(Box::new(tray_menu.clone()))
                            .with_icon(default_icon.clone())
                            .with_tooltip("Bluetooth battery")
                            .build()
                            .unwrap());
            }
            
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {

                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_secs(2));
                
                
                let mut devices_guard = current_devices.lock().unwrap();
                if *adapter_on.lock().unwrap() {
                    
                    let connected_devices: Vec<DeviceInfo> = query_bluetooth_devices();
                    println!("{connected_devices:?}");
                    // println!("Adapter is on");
                    //
                    // TODO: move everything below to their own functions
                    //
                    if connected_devices.len() == 0 {
                        tray_icon_app.as_ref().unwrap().set_tooltip(Some("No connected device with battery")).unwrap();
                        devices_guard.devices.take();
                        flash_bluetooth_battery(&tray_battery_icons, &mut tray_icon_app, battery_index);
                    } else {
                        let mut devices: Vec<BluetoothDevice> = Vec::new();
                        println!("Connected devices with battery service: ");
                        for device in &connected_devices {
                            let new_device_name = &device.device_name;
                            let mut new_device_battery_percent: Option<u8> = None;
                            let mut new_device_icon: Option<Icon> = None;
                            let battery_level = device.battery_info;
                            new_device_battery_percent = Some(battery_level);
                            battery_index = ((100 - battery_level) / 25) as usize;
                            let find_icon = tray_battery_icons.as_ref().unwrap().get(battery_index).unwrap().clone();
                            new_device_icon = Some(find_icon);
                            
                        if new_device_battery_percent.is_some() {
                            devices.push(BluetoothDevice {
                                device_info: DisplayDeviceInformation::new(new_device_name.to_string(), new_device_battery_percent, new_device_icon, false)
                            })
                        }}
                        *devices_guard = BluetoothDevices { devices: Some(devices) };
                    }
                } else {
                    println!("Adapter is off");
                    devices_guard.devices.take();
                    tray_icon_app.as_ref().unwrap().set_tooltip(Some("Bluetooth is not on.")).unwrap();
                    
                    battery_index = if battery_index == 4 {
                        0
                    } else {
                        battery_index + 1
                    };
                    flash_bluetooth_battery(&tray_battery_icons, &mut tray_icon_app, battery_index);
                }
                let updated_menu = Menu::new();
                updated_menu.append(&quit_i);
                let mut items_to_add = Vec::new();
                match devices_guard.devices.as_ref() {
                    Some(devices) => {
                        updated_menu.prepend(&PredefinedMenuItem::separator());
                        for (index, device) in devices.iter().enumerate() {

                            let menu_text = format!("{} - {}%", device.device_info.device_name(), device.device_info.battery_info().unwrap());
                            //
                            // TODO: indicate to user which device is selected in menu
                            //
                            let item = MenuItem::with_id(index, menu_text, true, None);
                            items_to_add.push(item);
                            
                            println!("{:?}", device);
                        }
                        tray_icon_app.as_ref().unwrap().set_tooltip(Some("Select a device"));
                    }
                    None => {
                        selected_device.take();
                        println!("No devices found");
                    }
                }
                for item in items_to_add {
                    updated_menu.prepend(&item);
                }
                if menu_changed(&tray_menu, &updated_menu) {
                    tray_icon_app.as_ref().unwrap().set_menu(Some(Box::new(updated_menu.clone())));
                    tray_menu = updated_menu;
                }
                
                if let Some(display_device) = &selected_device {
                    tray_icon_app.as_ref().unwrap().set_tooltip(Some(format!("{} {}%", display_device.device_info.device_name(), display_device.device_info.battery_info().unwrap())));
                    tray_icon_app.as_ref().unwrap().set_icon(display_device.device_info.battery_icon.as_ref().cloned());
                } 
                // else {
                //     tray_icon_app.as_ref().unwrap().set_tooltip(Some("Select a device"));
                // }
            }
            _ => {

            }
        }
        match menu_channel.try_recv() {
            Ok(e) => {
                if e.id == quit_i.id() {
                    *control_flow = ControlFlow::Exit;
                } else {
                    let guard = current_devices.lock().unwrap();
                    let yep = guard.devices.as_ref().unwrap().get(e.id.0.parse::<usize>().unwrap());
                    selected_device = yep.cloned();
                    selected_device.as_mut().unwrap().device_info.selected = true;
                }
                println!("Menu: {event:?}");
            }
            Err(_e) => {

            }
        }
        if let Ok(event) = tray_channel.try_recv() {
            println!("{event:?}");
        }
    });
}

fn flash_bluetooth_battery(icons: &Option<Vec<Icon>>, tray: &mut Option<TrayIcon>, index: usize) {
    tray.as_ref().unwrap().set_icon(Some(icons.as_ref().unwrap().get(index).unwrap().clone()));
}

fn menu_changed(old_menu: &Menu, new_menu: &Menu) -> bool {
    old_menu.items().len() != new_menu.items().len()
}