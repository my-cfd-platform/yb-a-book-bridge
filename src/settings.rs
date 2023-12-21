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
    pub my_no_sql_tcp_reader: String,
    pub seq_conn_string: String,
    pub my_telemetry: String,
}
