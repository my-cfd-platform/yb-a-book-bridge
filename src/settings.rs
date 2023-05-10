use async_trait::async_trait;
use my_no_sql_tcp_reader::MyNoSqlTcpConnectionSettings;
use my_settings_reader::SettingsModel;
use my_tcp_sockets::TcpClientSocketSettings;
use serde::{Deserialize, Serialize};
use service_sdk::ServiceInfo;

pub struct PositionsConnectionString(String);
pub struct BalanceHistoryConnectionString(String);

#[derive(SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(rename = "NoSqlTcp")]
    pub no_sql_tcp: String,
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


#[async_trait::async_trait]
impl my_seq_logger::SeqSettings for SettingsReader {
    async fn get_conn_string(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.seq_conn_string.clone()
    }
}


#[async_trait::async_trait]
impl MyNoSqlTcpConnectionSettings for SettingsReader {
    async fn get_host_port(&self) -> String{
        let read_access = self.settings.read().await;
        read_access.no_sql_tcp.clone()
    }
}

#[async_trait::async_trait]
impl my_telemetry_writer::MyTelemetrySettings for SettingsReader {
    async fn get_telemetry_url(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.my_telemetry.clone()
    }
}


#[async_trait::async_trait]
impl ServiceInfo for SettingsReader {
    fn get_service_name(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }
    fn get_service_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

#[async_trait]
impl TcpClientSocketSettings for Settings {
    async fn get_host_port(&self) -> String {
        return self.url.clone();
    }
}