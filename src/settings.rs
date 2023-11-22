use my_tcp_sockets::TcpClientSocketSettings;
use serde::{Deserialize, Serialize};
service_sdk::macros::use_settings!();

#[derive(
    service_sdk::my_settings_reader::SettingsModel,
    AutoGenerateSettingsTraits,
    SdkSettingsTraits,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
pub struct Settings {
    #[serde(rename = "NoSqlTcp")]
    pub my_no_sql_tcp_reader: String,
    #[serde(rename = "Seq")]
    pub seq_conn_string: String,
    #[serde(rename = "MyTelemetry")]
    pub my_telemetry: String,
    #[serde(rename = "YbPassword")]
    pub password: String,
    #[serde(rename = "YbTarget")]
    pub target: String,
    #[serde(rename = "YbSender")]
    pub sender: String,
    #[serde(rename = "YbUrl")]
    pub url: String,
}

#[async_trait]
impl TcpClientSocketSettings for SettingsReader {
    async fn get_host_port(&self) -> String {
        let settings = self.get_settings().await;
        return settings.url.clone();
    }
}
