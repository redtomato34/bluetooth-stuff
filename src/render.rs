use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use futures::executor::block_on;
use tao::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

use crate::{util::load_icons, BluetoothInfo};

const SHORT_TIMER: Duration = Duration::from_secs(2);

pub async fn run_render_thread(info: BluetoothInfo) {
    let bt_info = info;
    let mut cached_info = None;
    let event_loop = EventLoopBuilder::new().build();

    let tray_battery_icons = load_icons().unwrap();
    let tray_menu = Menu::new();

    let quit_i: MenuItem = MenuItem::new("Quit", true, None);
    let refresh_i = MenuItem::with_id(0, "Refresh", true, None);
    tray_menu.append_items(&[&refresh_i, &quit_i]).unwrap();

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
                let default_icon = tray_battery_icons.get(5).unwrap();
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
                        Some(device_info) => {
                            // skip updating icon if battery didn't change
                            match cached_info {
                                Some(timestamp) => {
                                    if timestamp == device_info.checked_timestamp {
                                        return;
                                    }
                                }
                                None => {}
                            }
                            cached_info = Some(device_info.checked_timestamp);
                            let device_name = device_info.device_name.clone();
                            let last_checked_local: DateTime<Local> = DateTime::from_timestamp(device_info.checked_timestamp, 0).unwrap().into();
                            let last_checked_formatted = last_checked_local.format("%I:%M:%S").to_string();
                            let battery_format = if device_info.battery_level.is_some() {
                                format!("{}%", device_info.battery_level.unwrap())
                            } else {
                                format!("NA")
                            };
                            let icon = if device_info.battery_icon_index.is_some() {
                                tray_battery_icons.get(device_info.battery_icon_index.unwrap())
                            } else {
                                tray_battery_icons.get(5)
                            };
                            let formatted_tooltip = format!("{device_name} {battery_format} \nLast checked {last_checked_formatted}");
                            device_tooltip = Some(formatted_tooltip);
                            device_icon = icon.cloned();
                        }
                        None => {
                            cached_info = None;
                            device_tooltip = Some("No device found or bluetooth is off".to_string());
                            device_icon = Some(tray_battery_icons.get(5).unwrap().clone());
                        }
                    }
                    // println!("Updating tooltip: {:?}", device_tooltip);
                    match tray_icon_app.as_ref() {
                        Some(app) => {
                            let _ = app.set_tooltip(device_tooltip);
                            let _ = app.set_icon(device_icon);
                        }
                        None => {
                        }
                    }
                }
            }
            _ => {}
        }
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == refresh_i.id() {
                cached_info = None;
            } else if event.id == quit_i.id() {
                tray_icon_app.take();
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}