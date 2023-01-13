use btleplug::api::{Central, CharPropFlags, Manager, Peripheral, ScanFilter};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use uuid::{uuid, Uuid};

const RACEBOX_LOCAL_NAME_PREFIX: &str = "RaceBox Mini ";

// RaceBox mini characteristics
#[allow(dead_code)]
const DEVICE_INFO_CHAR: Uuid = uuid!("0000180a-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const MODEL_CHAR: Uuid = uuid!("00002a24-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const SERIAL_NUMBER_CHAR: Uuid = uuid!("00002a25-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const FIRMWARE_REV_CHAR: Uuid = uuid!("00002a26-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const HARDWARE_REV_CHAR: Uuid = uuid!("00002a27-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const MANUFACTURER_CHAR: Uuid = uuid!("00002a29-0000-1000-8000-00805f9b34fb");
#[allow(dead_code)]
const UART_SERVICE_CHAR: Uuid = uuid!("6E400001-B5A3-F393-E0A9-E50E24DCCA9E");
#[allow(dead_code)]
const RX_CHAR: Uuid = uuid!("6E400002-B5A3-F393-E0A9-E50E24DCCA9E");
const TX_CHAR: Uuid = uuid!("6E400003-B5A3-F393-E0A9-E50E24DCCA9E");

// btle connection management
#[allow(dead_code)]
pub struct RbManager {
    adapter_list: Vec<btleplug::platform::Adapter>,
    manager: Box<btleplug::platform::Manager>,
    peripherals: Vec<btleplug::platform::Peripheral>,
}

// RaceBox Mini connection
#[allow(dead_code)]
pub struct RbConnection {
    peripheral: btleplug::platform::Peripheral,
    pub serial: String,
}

impl RbManager {
    pub async fn new() -> Result<RbManager, String> {
        let manager = Box::new(btleplug::platform::Manager::new().await.unwrap());

        let adapter_list = manager.adapters().await.unwrap();
        if adapter_list.is_empty() {
            return Err(String::from("No adapters found"));
        }
        let mut peripherals = Vec::new();
        for adapter in adapter_list.iter() {
            if adapter.start_scan(ScanFilter::default()).await.is_err() {
                return Err(String::from("Failed to scan for adapters"));
            }
            time::sleep(Duration::from_secs(10)).await;
            peripherals = adapter.peripherals().await.unwrap();
            if peripherals.is_empty() {
                return Err(String::from("No devices found"));
            }
        }

        Ok(RbManager {
            adapter_list,
            manager,
            peripherals,
        })
    }

    pub async fn connect(&mut self) -> Result<RbConnection, String> {
        for peripheral in self.peripherals.iter() {
            let properties = peripheral.properties().await.unwrap();
            let is_connected = peripheral.is_connected().await.unwrap();
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or_else(|| String::from("unknown name"));

            if !local_name.starts_with(RACEBOX_LOCAL_NAME_PREFIX) {
                continue;
            }

            if !is_connected && peripheral.connect().await.is_err() {
                continue;
            }

            let serial = local_name
                .strip_prefix(RACEBOX_LOCAL_NAME_PREFIX)
                .unwrap()
                .to_string();

            let r_peripheral = peripheral.clone();
            return Ok(RbConnection {
                peripheral: r_peripheral,
                serial,
            });
        }
        Err(String::from("failed to find racebox mini"))
    }
}

impl RbConnection {
    pub async fn stream(
        &self,
        channel: mpsc::Sender<btleplug::api::ValueNotification>,
    ) -> Result<(), Box<dyn Error>> {
        if self.peripheral.discover_services().await.is_err() {
            panic!("Couldn't discover services, fix this panic please")
        };
        for characteristic in self.peripheral.characteristics() {
            if characteristic.uuid == TX_CHAR
                && characteristic.properties.contains(CharPropFlags::NOTIFY)
            {
                self.peripheral.subscribe(&characteristic).await?;
                let mut stream = self.peripheral.notifications().await?;

                while let Some(data) = stream.next().await {
                    channel.send(data).await?;
                }
            }
        }
        Ok(())
    }
}
