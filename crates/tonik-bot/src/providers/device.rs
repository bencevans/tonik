use std::{sync::Arc, time::Duration};
use tracing::{debug, info, warn};

use crate::events::Event;

pub struct DeviceProvider {
    current_devices: Vec<String>,
    tonik: Arc<tonik::TeltonikaClient>,
}

impl DeviceProvider {
    pub fn new(tonik: Arc<tonik::TeltonikaClient>) -> Self {
        DeviceProvider {
            current_devices: Vec::new(),
            tonik,
        }
    }

    pub async fn list_devices(&self) -> Vec<String> {
        let response = match self.tonik.ip_neighbors_ipv4_status().await {
            Ok(res) => res,
            Err(_) => {
                warn!("Failed to fetch device list");
                return Vec::new();
            }
        };

        let devices = match response.data {
            Some(devices) => devices,
            None => {
                warn!("Response is empty");
                return Vec::new();
            }
        };

        devices
            .into_iter()
            .filter_map(|device| device.mac)
            .collect()
    }

    pub async fn hoist(&mut self, tx: &mut tokio::sync::mpsc::Sender<Event>) {
        info!("Hoisting device watch");
        self.current_devices = self.list_devices().await;
        info!(
            "Initial device list fetched, found {} devices",
            self.current_devices.len()
        );

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let new_device_list = self.list_devices().await;
            debug!(
                "New device list fetched, found {} devices",
                new_device_list.len()
            );

            let new_devices: Vec<String> = new_device_list
                .iter()
                .filter(|device| !self.current_devices.contains(device))
                .cloned()
                .collect();

            let departed_devices: Vec<String> = self
                .current_devices
                .iter()
                .filter(|device| !new_device_list.contains(device))
                .cloned()
                .collect();

            for device in departed_devices {
                tx.send(Event::DeviceLeft(device.clone())).await.unwrap();
            }

            for device in new_devices {
                tx.send(Event::DeviceArrived(device.clone())).await.unwrap();
            }

            self.current_devices = new_device_list;
        }
    }
}
