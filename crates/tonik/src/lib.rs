use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

pub struct TeltonikaClient {
    host: String,
    reqwest: reqwest::Client,
    auth: Option<LoginData>,
}

impl TeltonikaClient {
    pub fn new(host: String) -> Self {
        TeltonikaClient {
            host,
            reqwest: reqwest::Client::builder().gzip(true).build().unwrap(),
            auth: None,
        }
    }

    pub async fn authenticate(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<Response<LoginData>, reqwest::Error> {
        let response = self.login(username, password).await?;
        self.auth = response.data.clone();
        Ok(response)
    }

    /// Send a POST request to the router.
    pub async fn post<R, T>(
        &self,
        path: &str,
        body: Option<R>,
    ) -> Result<Response<T>, reqwest::Error>
    where
        R: Serialize,
        T: DeserializeOwned,
    {
        let mut request = self
            .reqwest
            .post(format!("http://{}/api{}", self.host, path).as_str());

        if let Some(auth) = self.auth.as_ref() {
            request = request.bearer_auth(auth.token.as_str());
        }

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?.json::<Response<T>>().await?;

        Ok(response)
    }

    /// Send a GET request to the router.
    pub async fn get<T>(&self, path: &str) -> Result<Response<T>, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let mut request = self
            .reqwest
            .get(format!("http://{}/api{}", self.host, path).as_str());

        if let Some(auth) = self.auth.as_ref() {
            request = request.bearer_auth(auth.token.as_str());
        }

        let response = request.send().await?.json::<Response<T>>().await?;

        Ok(response)
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Response<LoginData>, reqwest::Error> {
        self.post(
            "/login",
            Some(&json!({
                "username": username,
                "password": password,
            })),
        )
        .await
    }

    pub async fn dhcp_leases_ipv4_status(
        &self,
    ) -> Result<Response<Vec<DhcpLease>>, reqwest::Error> {
        self.get("/dhcp/leases/ipv4/status").await
    }

    pub async fn firmware_device_status(
        &self,
    ) -> Result<Response<FirmwareDeviceStatus>, reqwest::Error> {
        self.get("/firmware/device/status").await
    }

    pub async fn firmware_actions_fota_download(&self) -> Result<Response<()>, reqwest::Error> {
        self.post("/firmware/actions/fota_download", None::<()>)
            .await
    }

    pub async fn gps_position_status(&self) -> Result<Response<GpsPositionStatus>, reqwest::Error> {
        self.get("/gps/position/status").await
    }

    pub async fn wireless_devices_status(
        &self,
    ) -> Result<Response<Vec<WirelessDeviceStatus>>, reqwest::Error> {
        self.get("/wireless/devices/status").await
    }

    pub async fn wireless_interfaces_status(
        &self,
    ) -> Result<Response<Vec<InterfaceStatus>>, reqwest::Error> {
        self.get("/wireless/interfaces/status").await
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceStatus {
    pub ifname: String,
    pub disabled: bool,
    pub op_class: i64,
    pub status: String,
    pub quality: i64,
    pub noise: i64,
    pub up: bool,
    pub device: InterfaceStatusDevice,
    pub txpoweroff: i64,
    // rrm
    pub bitrate: i64,
    pub name: String,
    // airtime
    // ...
    pub ssid: String,
    pub assoclist: HashMap<String, InterfaceStatusAssoc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceStatusAssoc {
    pub signal: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceStatusDevice {
    device: String,
    pending: bool,
    name: String,
    up: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WirelessDeviceStatus {
    pub id: String,
    pub quality_max: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GpsPositionStatus {
    accuracy: String,
    fix_status: String,
    altitude: String,
    timestamp: String,
    satellites: String,
    longitude: String,
    latitude: String,
    angle: String,
    utc_timestamp: String,
}

impl Display for GpsPositionStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Accuracy: {}\nFix status: {}\nAltitude: {}\nTimestamp: {}\nSatellites: {}\nLongitude: {}\nLatitude: {}\nAngle: {}\nUTC timestamp: {}",
            self.accuracy, self.fix_status, self.altitude, self.timestamp, self.satellites, self.longitude, self.latitude, self.angle, self.utc_timestamp
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FirmwareDeviceStatus {
    pub kernel_version: String,
    pub version: String,
    pub build_date: String,
}

impl Display for FirmwareDeviceStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Kernel version: {}\nVersion: {}\nBuild date: {}",
            self.kernel_version, self.version, self.build_date
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DhcpLease {
    pub expires: i64,
    pub macaddr: String,
    pub ipaddr: String,
    pub hostname: Option<String>,
}

impl Display for DhcpLease {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "MAC address: {}\nIP address: {}\nHostname: {}\nExpires: {}",
            self.macaddr,
            self.ipaddr,
            self.hostname.as_deref().unwrap_or(""),
            self.expires
        )?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginData {
    pub username: String,
    pub token: String,
    pub expires: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub success: bool,
    pub data: Option<T>,
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub code: i32,
    pub error: String,
    pub source: String,
    pub section: Option<String>,
}

#[cfg(test)]
mod tests {

    use std::env;

    use super::*;

    fn create_client() -> TeltonikaClient {
        TeltonikaClient::new(env::var("TELTONIKA_HOST").expect("TELTONIKA_HOST is not set"))
    }

    async fn create_authenticated_client() -> TeltonikaClient {
        let mut client = create_client();
        let response = client
            .authenticate(
                env::var("TELTONIKA_USERNAME")
                    .expect("TELTONIKA_USERNAME is not set")
                    .as_str(),
                env::var("TELTONIKA_PASSWORD")
                    .expect("TELTONIKA_PASSWORD is not set")
                    .as_str(),
            )
            .await
            .unwrap();

        assert!(response.success);
        assert!(response.data.is_some());

        client
    }

    #[tokio::test]
    async fn test_login() {
        create_authenticated_client().await;
    }

    #[tokio::test]
    async fn test_dhcp_leases_ipv4_status() {
        let client = create_authenticated_client().await;
        let response = client.dhcp_leases_ipv4_status().await.unwrap();

        assert!(response.success);
        assert!(response.data.is_some());
    }

    #[tokio::test]
    async fn test_firmware_device_status() {
        let client = create_authenticated_client().await;
        let response = client.firmware_device_status().await.unwrap();

        assert!(response.success);
        assert!(response.data.is_some());
    }
}
