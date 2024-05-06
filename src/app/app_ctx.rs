use std::sync::Arc;

use my_nosql_contracts::{
    BidAskSnapshotNoSqlEntity, InstrumentMappingEntity, ProductSettings,
    TradingInstrumentNoSqlEntity, YbABookSettings,
};
use my_tcp_sockets::TcpClientSocketSettings;
use service_sdk::{my_no_sql_sdk::reader::MyNoSqlDataReaderTcp, ServiceContext};

use crate::{ABookTcpEventProcessor, SettingsReader};

pub const LP_NAME: &str = "Yourbourse";

pub struct AppContext {
    pub mapping_ns_reader: Arc<MyNoSqlDataReaderTcp<InstrumentMappingEntity>>,
    pub bid_ask_ns_reader: Arc<MyNoSqlDataReaderTcp<BidAskSnapshotNoSqlEntity>>,
    pub instruments_ns_reader: Arc<MyNoSqlDataReaderTcp<TradingInstrumentNoSqlEntity>>,
    pub product_settings: Arc<MyNoSqlDataReaderTcp<ProductSettings>>,
    pub settings: Arc<SettingsReader>,
    pub a_book_tcp_processor: Arc<ABookTcpEventProcessor>,
}

impl AppContext {
    pub async fn new(ctx: &ServiceContext, settings: Arc<SettingsReader>) -> Self {
        Self {
            mapping_ns_reader: ctx.get_ns_reader().await,
            bid_ask_ns_reader: ctx.get_ns_reader().await,
            settings,

            instruments_ns_reader: ctx.get_ns_reader().await,
            product_settings: ctx.get_ns_reader().await,
            a_book_tcp_processor: Arc::new(ABookTcpEventProcessor::new()),
        }
    }

    pub async fn get_yb_settings(&self) -> Option<YbABookSettings> {
        self.product_settings.get_enum_case_model().await
    }
}

#[async_trait::async_trait]
impl TcpClientSocketSettings for AppContext {
    async fn get_host_port(&self) -> Option<String> {
        let yb_settings = self.get_yb_settings().await?;

        return Some(yb_settings.url.clone());
    }
}
