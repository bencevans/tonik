use std::collections::HashMap;

use clap::Parser;
use tonik::{IpNeighborStatusV4, Response, SessionStatus, TeltonikaClient};
use tracing::{debug, error, info, warn};

/// Tonic Activity Water
///
/// Emits events as devices connect/disconnect from the network
#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value = "admin")]
    username: String,

    #[clap(long)]
    password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut known_devices = HashMap::new();
    known_devices.insert("82:87:EE:2A:86:AE".to_string(), "Ben's iPhone".to_string());
    known_devices.insert(
        "5C:50:D9:C3:4D:F7".to_string(),
        "Jenny's iPhone".to_string(),
    );
    known_devices.insert("96:32:70:B5:C7:8B".to_string(), "iPad".to_string());
    known_devices.insert("84:F3:EB:E0:12:12".to_string(), "Shelly H/T".to_string());
    known_devices.insert("68:C6:3A:F9:3A:84".to_string(), "Shelly H/T".to_string());
    known_devices.insert("84:F3:EB:E0:12:88".to_string(), "Shelly H/T".to_string());
    known_devices.insert("20:97:27:4E:7A:35".to_string(), "TAP200".to_string());
    known_devices.insert("84:CC:A8:AF:B1:11".to_string(), "Shelly RGBW".to_string());
    known_devices.insert("36:95:D8:AD:B3:C4".to_string(), "Ben's MacBook".to_string());
    known_devices.insert(
        "B8:27:EB:FB:2A:45".to_string(),
        "Jack / Raspberry Pi".to_string(),
    );
    known_devices.insert("EC:62:60:8F:DD:7C".to_string(), "Spring".to_string());
    known_devices.insert("10:52:1C:45:6D:27".to_string(), "Shelly 1".to_string());
    known_devices.insert("F0:A7:31:34:F5:19".to_string(), "P110M Monitor".to_string());
    known_devices.insert(
        "E6:DC:A6:AF:39:80".to_string(),
        "Jenny's MacBook Air".to_string(),
    );

    let mut client = TeltonikaClient::new(args.host);

    client
        .authenticate(&args.username, &args.password)
        .await
        .expect("Failed to authenticate");

    let mut last_status: Vec<IpNeighborStatusV4> = Vec::new();

    loop {
        match client.session_status().await {
            Ok(Response {
                data: Some(SessionStatus { active }),
                success: _,
                errors: _,
            }) => {
                if !active {
                    warn!("Session is not active, authenticating...");
                    client
                        .authenticate(&args.username, &args.password)
                        .await
                        .expect("Failed to authenticate");
                }
            }
            Ok(Response {
                data: None,
                success: _,
                errors: _,
            }) => {
                warn!("Received unexpected response");
            }
            Err(err) => {
                error!("Failed to fetch session status: {}", err);
                continue;
            }
        }

        let response = match client.ip_neighbors_ipv4_status().await {
            Ok(response) => response,
            Err(err) => {
                error!("Failed to fetch IP neighbor status: {}", err);
                continue;
            }
        };

        if let Some(new_status) = response.data.clone() {
            debug!("Received new status");
            let macs_in_new_status: Vec<String> = new_status
                .iter()
                .filter_map(|neighbor| neighbor.mac.clone())
                .collect();
            let macs_in_last_status: Vec<String> = last_status
                .iter()
                .filter_map(|neighbor| neighbor.mac.clone())
                .collect();

            let macs_added = macs_in_new_status
                .iter()
                .filter(|mac| !macs_in_last_status.contains(mac))
                .collect::<Vec<&String>>();

            let macs_removed = macs_in_last_status
                .iter()
                .filter(|mac| !macs_in_new_status.contains(mac))
                .collect::<Vec<&String>>();

            for mac in macs_added {
                if let Some(device_name) = known_devices.get(mac) {
                    info!("Added: {:?} ({})", mac, device_name);
                } else {
                    warn!("Added: {:?}", mac);
                }
            }
            for mac in macs_removed {
                if let Some(device_name) = known_devices.get(mac) {
                    info!("Removed: {:?} ({})", mac, device_name);
                } else {
                    warn!("Removed: {:?}", mac);
                }
            }

            last_status = new_status;
        } else {
            warn!("No data received");
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
