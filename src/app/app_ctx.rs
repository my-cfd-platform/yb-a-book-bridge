use std::sync::Arc;

use my_nosql_contracts::{
    BidAskSnapshotNoSqlEntity, InstrumentMappingEntity, TradingInstrumentNoSqlEntity,
};
use my_tcp_sockets::TcpClient;
use service_sdk::{
    my_no_sql_sdk::reader::{MyNoSqlDataReader, MyNoSqlDataReaderTcp},
    ServiceContext,
};
use yb_tcp_contracts::{FixLogonCredentials, YourBourseFixTcpSerializer};

use crate::{ABookTcpEventProcessor, Settings, SettingsReader};

pub const LP_NAME: &str = "Yourbourse";

pub struct AppContext {
    pub fix_socket: Arc<ABookTcpEventProcessor>,
    pub mapping_ns_reader:
        Arc<dyn MyNoSqlDataReader<InstrumentMappingEntity> + Send + Sync + 'static>,
    pub bid_ask_ns_reader:
        Arc<dyn MyNoSqlDataReader<BidAskSnapshotNoSqlEntity> + Send + Sync + 'static>,
    pub instruments_ns_reader:
        Arc<dyn MyNoSqlDataReader<TradingInstrumentNoSqlEntity> + Send + Sync + 'static>,
    pub settings: Arc<SettingsReader>,
    pub tcp_client: TcpClient,
}

impl AppContext {
    pub async fn new(ctx: &ServiceContext, settings: Arc<SettingsReader>) -> Self {
        let tcp_client = TcpClient::new(
            "yourbourse - fix-trading-client".to_string(),
            settings.clone(),
        );

        Self {
            fix_socket: Arc::new(ABookTcpEventProcessor::new()),
            mapping_ns_reader: ctx.get_ns_reader().await,
            bid_ask_ns_reader: ctx.get_ns_reader().await,
            settings,
            tcp_client,
            instruments_ns_reader: ctx.get_ns_reader().await,
        }
    }

    pub async fn start(&self) {
        let settings = self.settings.get_settings().await;

        let credentials = FixLogonCredentials {
            password: settings.password.clone(),
            sender: settings.sender.clone(),
            target: settings.target.clone(),
        };

        self.tcp_client
            .start(
                Arc::new(move || -> YourBourseFixTcpSerializer {
                    YourBourseFixTcpSerializer::new(credentials.clone())
                }),
                self.fix_socket.clone(),
                service_sdk::my_logger::LOGGER.clone(),
            )
            .await;
    }
}
