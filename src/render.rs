use std::time::{Duration, Instant};

use futures::executor::block_on;
use tao::{event::{Event, StartCause}, event_loop::{ControlFlow, EventLoopBuilder}};
use tray_icon::{menu::{Menu, MenuEvent, MenuItem}, TrayIconBuilder, TrayIconEvent};

use crate::{util::image::load_icons, BluetoothInfo};



const SHORT_TIMER: Duration = Duration::from_secs(2);



pub async fn run_render_thread(info: BluetoothInfo) {
    let bt_info = info;
    let event_loop = EventLoopBuilder::new().build();
    
    let tray_battery_icons = load_icons();
    let tray_menu = Menu::new();
    
    let quit_i = MenuItem::new("Quit", true, None);
    let menu_item = MenuItem::with_id(0, "Yep", true, None);
    tray_menu.append_items(&[
        &menu_item,
        &quit_i,
    ]).unwrap();

    let mut tray_icon_app = None;
    
    // let menu_channel = MenuEvent::receiver();
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
                    let message = block_on(bt_info.message.as_ref().lock()).clone();
                    println!("Updating: {:?}", message);
                    tray_icon_app.as_ref().unwrap().set_tooltip(message).unwrap();
                }
            }
            _ => {

            }
        }
    });
}