use clap::Parser;
use tonik;

#[derive(Debug, Parser)]
struct App {
    #[clap(subcommand)]
    command: Commands,

    #[clap(long, default_value = "192.168.7.1")]
    host: String,

    #[clap(long, default_value = "admin")]
    username: String,

    #[clap(long, default_value = "password")]
    password: String,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    List,
    Read,
    Send,
}

#[tokio::main]
async fn main() {
    let args = App::parse();

    let mut client = tonik::TeltonikaClient::new(args.host);
    let auth_response = client
        .authenticate(&args.username, &args.password)
        .await
        .unwrap();
    println!("Authentication response: {:?}", auth_response);

    match args.command {
        Commands::List => {
            println!("Listing SMS messages...");
            let messages = client.list_sms_messages().await.unwrap().data.unwrap();
            for message in messages {
                println!("{:?}", message);
            }
        }
        Commands::Read => {
            println!("Reading SMS messages...");
        }
        Commands::Send => {
            println!("Sending SMS messages...");
            let response = client
                .send_sms_message("+447799555832", "Testing")
                .await
                .unwrap();
            println!("Response: {:?}", response);
        }
    }
}
