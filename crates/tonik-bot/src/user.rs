pub struct User {
    name: String,
    device_mac: Option<String>,
}

impl User {
    pub fn new(name: String) -> Self {
        User { name, device_mac: None }
    }
}
