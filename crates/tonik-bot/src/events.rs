use tonik::SmsMessage;

pub enum Event {
    MessageReceived(SmsMessage),
    DeviceArrived(String),
    DeviceLeft(String),
}
