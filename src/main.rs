/*
    Assets:
        Battery icons created by Stockio - Flaticon
        https://www.flaticon.com/authors/stockio
    Learning materials:
        Bluetooth HFP
        https://inthehand.com/2022/12/30/12-days-of-bluetooth-10-hands-free/
        Windows API
        https://learn.microsoft.com/en-us/windows/dev-environment/rust/rust-for-windows
        And too many stackoverflow links
*/
#![cfg_attr(
    all(
      target_os = "windows",
      not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]
mod util;
mod render;
mod bluetooth;

use std::sync::Arc;

use bluetooth::{run_bluetooth_thread, BluetoothInfo};
use futures::lock::Mutex;
use render::run_render_thread;
use util::share_bluetooth_info;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bluetooth_info = BluetoothInfo {
        connected_device: Arc::new(Mutex::new(None))
    };
    let display_info = share_bluetooth_info(&bluetooth_info);
    let update_info = share_bluetooth_info(&bluetooth_info);
    
    // run bluetooth thread
    // todo: spawn new thread for each device
    // run render on main thread for tray
    // if any exit, don't crash the program
    let bluetooth_thread_handle = run_bluetooth_thread(update_info).await;
    run_render_thread(display_info).await;
    bluetooth_thread_handle.abort();
    // should never execute this unless user pressed quit in tray menu
    Ok(())
}
