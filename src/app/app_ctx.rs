use std::sync::Arc;

use my_no_sql_tcp_reader::MyNoSqlDataReader;
use my_nosql_contracts::{BidAskSnapshotNoSqlEntity, InstrumentMappingEntity, TradingInstrumentNoSqlEntity};
use my_tcp_sockets::TcpClient;
use service_sdk::ServiceContext;
use yb_tcp_contracts::{YourBourseFixTcpSerializer, FixLogonCredentials};

use crate::{ABookTcpEventProcessor, Settings};

pub const LP_NAME: &str = "Yourbourse";

pub struct AppContext {
    pub fix_socket: Arc<ABookTcpEventProcessor>,
    pub mapping_ns_reader: Arc<MyNoSqlDataReader<InstrumentMappingEntity>>,
    pub bid_ask_ns_reader: Arc<MyNoSqlDataReader<BidAskSnapshotNoSqlEntity>>,
    pub instruments_ns_reader: Arc<MyNoSqlDataReader<TradingInstrumentNoSqlEntity>>,
    pub settings: Arc<Settings>,
    pub tcp_client: TcpClient,
}

impl AppContext {
    pub async fn new(ctx: &ServiceContext, settings: Arc<Settings>) -> Self {
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
        let credentials = FixLogonCredentials {
            password: self.settings.password.clone(),
            sender: self.settings.sender.clone(),
            target: self.settings.target.clone(),
        };

        self.tcp_client
            .start(
                Arc::new(move || -> YourBourseFixTcpSerializer {
                    YourBourseFixTcpSerializer::new(credentials.clone())
                }),
                self.fix_socket.clone(),
                my_logger::LOGGER.clone(),
            )
            .await;
    }
}
