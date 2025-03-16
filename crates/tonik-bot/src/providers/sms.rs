
use std::collections::HashMap;

use tonik::SmsMessage;

pub type PhoneNumber = String;

pub struct Sms {
    history: HashMap<PhoneNumber, Vec<String>>,
    client: tonik::TeltonikaClient,
}

impl Sms {
    pub fn new(client: tonik::TeltonikaClient) -> Self {
        Sms {
            history: HashMap::new(),
            client,
        }
    }

    pub async fn send_message(
        &mut self,
        phone_number: &PhoneNumber,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client.send_sms_message(phone_number, message).await?;
        self.history
            .entry(phone_number.clone())
            .or_default()
            .push(message.to_string());
        Ok(())
    }

    pub async fn new_messages(&mut self) -> Result<Vec<SmsMessage>, Box<dyn std::error::Error>> {
        let new_messages = self
            .client
            .list_sms_messages()
            .await?
            .data
            .unwrap_or_default();
        for message in new_messages {
            self.history
                .entry(message.sender)
                .or_default()
                .push(message);
        }
        Ok(())
    }
}
