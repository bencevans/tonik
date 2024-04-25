use clap::{CommandFactory, Parser};

#[derive(Debug, clap::Parser)]
struct App {
    /// Teltonika host
    #[clap(long, env = "TELTONIKA_HOST", default_value = "192.168.7.1")]
    host: Option<String>,

    /// Teltonika username
    #[clap(long, env = "TELTONIKA_USERNAME", default_value = "admin")]
    username: Option<String>,

    /// Teltonika password
    #[clap(long, env = "TELTONIKA_PASSWORD")]
    password: Option<String>,

    #[clap(subcommand)]
    command: Option<Command>,

    /// Output in JSON format
    #[clap(long)]
    json: bool,

    /// Generate shell completion
    #[clap(long)]
    completion: Option<clap_complete::Shell>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// DHCP related commands
    #[clap(name = "dhcp")]
    DhcpCommand(DhcpCommand),

    /// Firmware related commands
    #[clap(name = "firmware")]
    Firmware(FirmwareCommand),

    Gps(GpsCommand),
}

/// Global Positioning System related commands
#[derive(Debug, clap::Args)]
struct GpsCommand {
    #[clap(subcommand)]
    command: GpsCommandSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum GpsCommandSubcommand {
    // /// DHCP IPv4 related commands
    // #[clap(name = "global")]
    // Global,

    // #[clap(name = "status")]
    // Status,
    /// Get GPS Position
    #[clap(name = "position")]
    Position,
}

#[derive(Debug, clap::Args)]
struct DhcpCommand {
    #[clap(subcommand)]
    command: DhcpCommandSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum DhcpCommandSubcommand {
    /// DHCP IPv4 related commands
    #[clap(name = "ipv4")]
    DhcpCommandIpv4(DhcpCommandIpv4),
}

#[derive(Debug, clap::Args)]
struct DhcpCommandIpv4 {
    #[clap(subcommand)]
    command: DhcpCommandIpv4Subcommand,
}

#[derive(Debug, clap::Subcommand)]
enum DhcpCommandIpv4Subcommand {
    /// Get DHCP IPv4 leases status
    #[clap(name = "status")]
    Status,
}

#[derive(Debug, clap::Args)]
struct FirmwareCommand {
    #[clap(subcommand)]
    command: FirmwareCommandSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum FirmwareCommandSubcommand {
    /// Get firmware device status
    Device(FirmwareDeviceCommand),
    // / Actions related to firmware
    // Actions(FirmwareActionsCommand),
}

#[derive(Debug, clap::Args)]
struct FirmwareDeviceCommand {
    /// Get firmware device status
    #[clap(name = "status")]
    #[clap(subcommand)]
    command: FirmwareDeviceCommandSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum FirmwareDeviceCommandSubcommand {
    /// Get firmware device status
    #[clap(name = "status")]
    Status,
}

#[derive(Debug, clap::Subcommand)]
enum FirmwareActionsCommand {
    /// Download firmware over the air
    #[clap(name = "fota_download")]
    FotaDownload,
}

#[tokio::main]
async fn main() {
    let _app = App::parse();

    if let Some(shell) = _app.completion {
        let mut app = App::command();
        let app_name = app.get_name().to_string();
        clap_complete::generate(shell, &mut app, app_name, &mut std::io::stdout());
        return;
    }

    let client = async {
        let mut teltonika = tonik::TeltonikaClient::new(_app.host.expect("Host Required"));

        let authentication_response = teltonika
            .authenticate(
                _app.username.expect("Username Required").as_str(),
                _app.password.expect("Password Required").as_str(),
            )
            .await;

        if let Err(e) = authentication_response {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }

        teltonika
    };

    match _app.command {
        Some(Command::DhcpCommand(dhcp_command)) => match dhcp_command.command {
            DhcpCommandSubcommand::DhcpCommandIpv4(dhcp_ipv4_command) => {
                match dhcp_ipv4_command.command {
                    DhcpCommandIpv4Subcommand::Status => {
                        let response = client.await.dhcp_leases_ipv4_status().await.unwrap();
                        if _app.json {
                            println!("{}", serde_json::to_string_pretty(&response.data).unwrap());
                        } else {
                            for lease in response.data.unwrap() {
                                println!("{}", lease);
                            }
                        }
                    }
                }
            }
        },
        Some(Command::Firmware(firmware_command)) => match firmware_command.command {
            FirmwareCommandSubcommand::Device(firmware_device_command) => {
                match firmware_device_command.command {
                    FirmwareDeviceCommandSubcommand::Status => {
                        let response = client.await.firmware_device_status().await.unwrap();
                        if _app.json {
                            println!("{}", serde_json::to_string_pretty(&response.data).unwrap());
                        } else {
                            println!("{}", response.data.unwrap());
                        }
                        // println!("{}", response.data.unwrap());
                    }
                }
            }
        },
        Some(Command::Gps(gps_command)) => match gps_command.command {
            GpsCommandSubcommand::Position => {
                let response = client.await.gps_position_status().await.unwrap();
                if _app.json {
                    println!("{}", serde_json::to_string_pretty(&response.data).unwrap());
                } else {
                    println!("{}", response.data.unwrap());
                }
            }
        },
        None => {
            // Print help
            let mut app = App::command();
            app.print_help().unwrap();
            std::process::exit(1);
        }
    }
}
