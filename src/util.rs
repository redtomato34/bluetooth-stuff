
use std::{fs, path::Path};



use tray_icon::Icon;

use crate::bluetooth::BluetoothInfo;

const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\icons\\");


pub static WRITE_COMMANDS: [&str; 5] = [
    "BRSF:23", // m,inimum required for Cowin headphones to work
    "+CIND: (\"service\",(0,1)),(\"call\",(0,1))",
    "+CIND: 1,0",
    "+CHLD: 0",
    "+XAPL=iPhone,6"
];

pub static READ_COMMANDS: [&str; 6] = [
    "AT+BRSF",
    "AT+CIND=?",
    "AT+CIND?",
    "AT+CHLD=?",
    "AT+XAPL",
    "AT+IPHONEACCEV"
];

pub fn load_icons() -> Option<Vec<Icon>> {
    let mut battery_icons: Vec<Icon> = Vec::new();
    let file_path = Path::new(ICON_PATH);
    let icon_paths = fs::read_dir(file_path).unwrap();
    
    for icon in icon_paths {
        let (_, icon_rgba, icon_width, icon_height) = {
            let mut img = image::open(icon.as_ref().unwrap().path())
                .expect("Failed to open icon path");
            img.invert();
            let inverted_img = img.into_rgba8();
            let (width, height) = inverted_img.dimensions();
            let rgba = inverted_img.clone().into_raw();
            (icon.unwrap().file_name().into_string().unwrap(), rgba, width, height)
        };
        
        let create_icon = tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

        battery_icons.push(create_icon);
    }
    Some(battery_icons)
}

pub fn share_bluetooth_info(info: &BluetoothInfo) -> BluetoothInfo {
    BluetoothInfo {
        connected_device: info.connected_device.clone(),
    }
}