use std::sync::Arc;

use futures::lock::Mutex;
use tao::window::Icon;

pub mod image;

pub static WRITE_COMMANDS: [&str; 5] = [
    "BRSF:0",
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

#[derive(Clone)]
pub struct BluetoothInfo {
    pub adapter_is_on: Arc<Mutex<bool>>,
    pub connected_devices: Arc<Mutex<Option<Vec<DeviceInfo>>>>,
    pub message: Arc<Mutex<Option<String>>>
}


pub struct DeviceInfo {
    device_id: String,
    device_name: String,
    device_type: DeviceType,
    battery_level: Option<u8> ,
    battery_icon: Option<Icon>,
    checked_timestamp: u32,
    is_ble: bool
}

enum DeviceType {
    Headset
}