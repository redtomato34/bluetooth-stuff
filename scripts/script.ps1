$local:BatteryLevel = '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2';
$local:IsConnected = "{83DA6326-97A6-4088-9453-A1923F573B29} 15";

$local:Result = @();
Get-PnpDevice -Class "Bluetooth" | ForEach-Object {
    $local:DeviceConnected = $_ | Get-PnpDeviceProperty -KeyName $IsConnected;
    $local:TestForBattery = $_ | Get-PnpDeviceProperty -KeyName $BatteryLevel | Where-Object Type -NE Empty;
    if ($TestForBattery -and $DeviceConnected.Data) {
        $local:DeviceInfo = @{
            "device_name" = $_.friendlyName
            "battery_info" = $TestForBattery.Data
        }
        $Result += $DeviceInfo
    }
}
ConvertTo-Json -InputObject $Result;
