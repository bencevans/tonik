use std::fmt::{self, Display, Formatter};

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
        let response = self
            .reqwest
            .get("http://192.168.7.1/api/dhcp/leases/ipv4/status")
            .bearer_auth(self.auth.as_ref().unwrap().token.as_str())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn firmware_device_status(
        &self,
    ) -> Result<Response<FirmwareDeviceStatus>, reqwest::Error> {
        let response = self
            .reqwest
            .get("http://192.168.7.1/api/firmware/device/status")
            .bearer_auth(self.auth.as_ref().unwrap().token.as_str())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn firmware_actions_fota_download(&self) -> Result<Response<()>, reqwest::Error> {
        let response = self
            .reqwest
            .post("http://192.168.7.1/api/firmware/actions/fota_download")
            .bearer_auth(self.auth.as_ref().unwrap().token.as_str())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
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
