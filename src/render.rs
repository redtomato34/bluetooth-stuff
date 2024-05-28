use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use futures::executor::block_on;
use tao::{event::{Event, StartCause}, event_loop::{ControlFlow, EventLoopBuilder}};
use tray_icon::{menu::{Menu, MenuEvent, MenuItem}, Icon, TrayIconBuilder};

use crate::{util::load_icons, BluetoothInfo};

const SHORT_TIMER: Duration = Duration::from_secs(2);

pub async fn  run_render_thread(info: BluetoothInfo) {
    let bt_info = info;
    let mut cached_info = None;
    let event_loop = EventLoopBuilder::new().build();
    
    let tray_battery_icons = load_icons();
    let tray_menu = Menu::new();
    
    let quit_i = MenuItem::new("Quit", true, None);
    let menu_item = MenuItem::with_id(0, "Refresh", true, None);
    tray_menu.append_items(&[
        &menu_item,
        &quit_i,
    ]).unwrap();
    
    let mut tray_icon_app = None;
    
    let menu_channel = MenuEvent::receiver();
    // let tray_channel = TrayIconEvent::receiver();
    
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
                {
                    let mut device_tooltip: Option<String> = None;
                    let mut device_icon: Option<Icon> = None;
                    let device_info = block_on(bt_info.connected_device.as_ref().lock());
                    match device_info.as_ref() {
                        Some(e) => {
                            match cached_info {
                                Some(n) => {
                                    if n == e.checked_timestamp {
                                        return;
                                    }
                                    cached_info = Some(e.checked_timestamp);
                                }
                                None => {
                                    cached_info = Some(e.checked_timestamp);
                                }
                            }                            
                            let mut battery = None;
                            let mut icon = None;
                            let device_name = e.device_name.clone();
                            let last_checked_local: DateTime<Local> = DateTime::from_timestamp(e.checked_timestamp, 0).unwrap().into();
                            let last_checked_formatted = last_checked_local.format("%I:%M:%S").to_string();
                            match e.battery_level {
                                Some(bat) => {
                                    battery = Some(bat);
                                }
                                None => {
                                    battery = None;
                                }
                            }
                            let battery_format = if battery.is_some() {
                                format!("{}%", battery.unwrap())
                            } else {
                                format!("NA")
                            };
                            icon = e.battery_icon.clone();
                            let formatted_tooltip = format!("{device_name} {battery_format} \nLast checked {last_checked_formatted}");
                            device_tooltip = Some(formatted_tooltip);
                            device_icon = icon;
                        }
                        None => {
                            cached_info = None;
                            device_tooltip = Some("No device found or bluetooth is off".to_string());
                            device_icon = Some(tray_battery_icons.as_ref().unwrap().get(5).unwrap().clone());
                        }
                    }
                    // println!("Updating tooltip: {:?}", device_tooltip);
                    tray_icon_app.as_ref().unwrap().set_tooltip(device_tooltip).unwrap();
                    tray_icon_app.as_ref().unwrap().set_icon(device_icon).unwrap();
                }
            }
            _ => {

            }
        }
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == menu_item.id() {
                cached_info = None;
            } else if event.id == quit_i.id() {
                tray_icon_app.take();
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}