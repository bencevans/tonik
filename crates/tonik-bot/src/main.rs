use chrono::{DateTime, Utc};
use clap::Parser;
use rs_openai::{
    OpenAI,
    interfaces::chat::{ChatCompletionMessageRequestBuilder, CreateChatRequestBuilder, Role},
};
use std::{collections::HashMap, sync::Arc};
use tonik;
use tonik_bot::{events::Event, providers::device::DeviceProvider};

#[derive(Debug, Parser)]
struct App {
    #[clap(long)]
    openai_key: String,

    #[clap(long, default_value = "192.168.7.1")]
    host: String,

    #[clap(long, default_value = "admin")]
    username: String,

    #[clap(long, default_value = "password")]
    password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = App::parse();

    let openai = OpenAI::new(&OpenAI {
        api_key: app.openai_key,
        org_id: None,
    });

    let tonik = Arc::new(tonik::TeltonikaClient::new(app.host.clone()));

    tonik
        .authenticate(&app.username, &app.password)
        .await
        .unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(100);

    let mut device_watcher = DeviceProvider::new(tonik.clone());

    tokio::spawn(async move { device_watcher.hoist(&mut tx.clone()).await });

    let mut user_db = UserDb::new();
    user_db.add_user(User {
        name: "Ben".to_string(),
        phone_number: "+447799555832".to_string(),
        mac_address: "82:87:EE:2A:86:AE".to_string(),
        last_seen: None,
    });

    while let Some(event) = rx.recv().await {
        match event {
            Event::DeviceArrived(mac_addr) => {
                let Some(user) = user_db.get_user_by_mac_address(&mac_addr) else {
                    println!("Unknown device connected: {}", mac_addr);
                    continue;
                };

                println!("Device connected: {} : {}", mac_addr, user.name);

                let message = generate_a_message(
                    &openai,
                    &format!(
                        "{} has just arrived on the boat, welcome them back",
                        user.name
                    ),
                )
                .await;

                tonik
                    .send_sms_message(&user.phone_number, &message)
                    .await
                    .unwrap();
            }
            Event::DeviceLeft(mac_addr) => {
                let Some(user) = user_db.get_user_by_mac_address(&mac_addr) else {
                    println!("Unknown device disconnected: {}", mac_addr);
                    continue;
                };

                println!("Device disconnected: {} : {}", mac_addr, user.name);

                let message = generate_a_message(
                    &openai,
                    &format!(
                        "{} has just left the boat, say something bidding them farewell",
                        user.name
                    ),
                )
                .await;

                tonik
                    .send_sms_message(&user.phone_number, &message)
                    .await
                    .unwrap();
            }
            Event::MessageReceived(sms_message) => {
                println!("New SMS Message {:?}", sms_message)
            }
        }
    }
}

async fn generate_a_message(openai: &OpenAI, prompt: &str) -> String {
    let request = CreateChatRequestBuilder::default()
        .model("gpt-4o-mini")
        .messages(vec![
            ChatCompletionMessageRequestBuilder::default()
                .role(Role::System)
                .content("You are a helpful assistant.")
                .build()
                .unwrap(),
            ChatCompletionMessageRequestBuilder::default()
                .role(Role::System)
                .content(prompt)
                .build()
                .unwrap(),
        ])
        .build()
        .unwrap();

    openai.chat().create(&request).await.unwrap().choices[0]
        .message
        .content
        .clone()
}

pub struct User {
    pub name: String,
    pub phone_number: String,
    pub mac_address: String,
    pub last_seen: Option<DateTime<Utc>>,
}

struct UserDb {
    users: HashMap<String, User>,
}

impl UserDb {
    pub fn new() -> Self {
        UserDb {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.phone_number.clone(), user);
    }

    pub fn get_user_by_phone_number(&self, phone_number: &str) -> Option<&User> {
        self.users.get(phone_number)
    }

    pub fn get_user_by_mac_address(&self, mac_address: &str) -> Option<&User> {
        self.users
            .values()
            .find(|user| user.mac_address == mac_address)
    }
}
