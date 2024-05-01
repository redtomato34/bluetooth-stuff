use tray_icon::Icon;

pub struct BluetoothDevices {
    pub devices: Option<Vec<BluetoothDevice>>
}

impl BluetoothDevices {
    pub fn new(&self, devices: Option<Vec<BluetoothDevice>>) -> Self {
        BluetoothDevices { devices }
    }
}


#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub device_info: DisplayDeviceInformation
}



#[derive(Debug, Clone)]

pub struct DisplayDeviceInformation {
    name: String, 
    battery_percentage: Option<u8>,
    pub battery_icon: Option<Icon>,
    pub selected: bool
}
impl DisplayDeviceInformation {
    pub fn new(name: String, battery_percentage: Option<u8>, battery_icon: Option<Icon>, selected: bool) -> Self {
        DisplayDeviceInformation {
            name,
            battery_percentage,
            battery_icon,
            selected
        }
    }
    pub fn device_name(&self) -> String {
        return self.name.clone()
    }
    pub fn battery_info(&self) -> Option<u8> {
        return self.battery_percentage
    }
}