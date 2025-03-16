use std::collections::HashMap;

pub mod providers;
pub mod events;

// pub struct Bot {
//     pub users: HashMap<String, user::User>,
//     pub tonik: tonik::TeltonikaClient,
// }

// impl Bot {
//     pub fn new(host: &str) -> Self {
//         Bot {
//             users: HashMap::new(),
//             tonik: tonik::TeltonikaClient::new(host.to_string()),
//         }
//     }

//     pub fn handle_event(&mut self, event: events::Event) {
//         match event {
//             events::Event::DeviceLeft(user) => self.users.remove(&user.id),
//             events::Event::MessageReceived(message) => {
//                 if let Some(user) = self.users.get_mut(&message.sender) {
//                     user.handle_message(message);
//                 }
//             }
//             events::Event::DeviceArrived(_) => todo!(),
//         }
//     }

//     pub async fn listen(&mut self) {
//         let sms_listener = sms::SmsListener::new(&self.tonik);
//         loop {
//             if let Some(event) = self.tonik.receive_event() {
//                 self.handle_event(event);
//             }
//             if let Some(sms) = sms_listener.receive_sms() {
//                 self.handle_event(events::Event::SmsReceived(sms));
//             }
//         }
//     }
// }
