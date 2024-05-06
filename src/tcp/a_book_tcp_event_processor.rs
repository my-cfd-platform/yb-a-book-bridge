use std::{collections::HashMap, sync::Arc, time::Duration};

use my_tcp_sockets::{tcp_connection::TcpSocketConnection, SocketEventCallback};
use service_sdk::async_trait;
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};
use yb_tcp_contracts::{tcp_serializer::YourBourseFixTcpSerializer, *};

#[derive(Debug)]
pub enum PlaceOrderError {
    ConnectionNotFound,
    MaxIterationsReached,
}

pub struct ConnectionContainer {
    pub connection: RwLock<Option<Arc<YbTcpSocketConnection>>>,
}

impl ConnectionContainer {
    pub fn new() -> Self {
        Self {
            connection: RwLock::new(None),
        }
    }

    pub async fn init_connection(&self, connection: Arc<YbTcpSocketConnection>) {
        println!("Connection set");
        let mut connection_mut = self.connection.write().await;
        *connection_mut = Some(connection);
    }

    pub async fn remove_connection(&self) {
        println!("Connection removed");
        let mut connection_mut = self.connection.write().await;
        *connection_mut = None;
    }

    pub async fn get_connection(&self) -> Option<Arc<YbTcpSocketConnection>> {
        let connection = self.connection.read().await;
        match &*connection {
            Some(connection) => Some(connection.clone()),
            None => None,
        }
    }
}

pub struct ABookTcpEventProcessor {
    active_connection: ConnectionContainer,
    success_orders: Mutex<HashMap<String, ExecutionReportModel>>,
    failed_orders: Mutex<HashMap<String, ExecutionReportModel>>,
}

impl ABookTcpEventProcessor {
    pub fn new() -> Self {
        Self {
            active_connection: ConnectionContainer::new(),
            success_orders: Mutex::new(HashMap::new()),
            failed_orders: Mutex::new(HashMap::new()),
        }
    }

    async fn send_logon(&self) -> Result<(), PlaceOrderError> {
        let connection = self.get_connection().await?;
        connection.send(&FixMessage::Logon).await;

        return Ok(());
    }

    pub async fn place_order(
        &self,
        id: &str,
        symbol: &str,
        side: PlaceOrderYbTcpContractSide,
        qty: f64,
    ) -> Result<ExecutionReportModel, PlaceOrderError> {
        let connection = self.get_connection().await?;

        connection
            .send(&FixMessage::PlaceOrder(PlaceOrderYbTcpContract {
                id: id.to_string(),
                symbol: symbol.to_string(),
                side,
                qty,
            }))
            .await;

        let mut count = 0;

        while count <= 10 {
            if let Some(order_report) = self.success_orders.lock().await.remove(id) {
                return Ok(order_report);
            }

            match self.failed_orders.lock().await.remove(id) {
                Some(order_report) => {
                    return Ok(order_report);
                }
                None => {
                    sleep(Duration::from_millis(300)).await;
                    count += 1;
                }
            }
        }

        return Err(PlaceOrderError::MaxIterationsReached);
    }

    async fn get_connection(&self) -> Result<Arc<YbTcpSocketConnection>, PlaceOrderError> {
        let connection = self.active_connection.get_connection().await;

        if connection.is_none() {
            return Err(PlaceOrderError::ConnectionNotFound);
        }

        let connection = connection.as_ref().unwrap();

        return Ok(connection.clone());
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<FixMessage, YourBourseFixTcpSerializer, YbTcpSate>
    for ABookTcpEventProcessor
{
    async fn connected(&self, connection: Arc<YbTcpSocketConnection>) {
        println!("Connected to FIX-Feed");
        self.active_connection.init_connection(connection).await;
        self.send_logon().await.unwrap();
    }

    async fn disconnected(
        &self,
        _connection: Arc<TcpSocketConnection<FixMessage, YourBourseFixTcpSerializer, YbTcpSate>>,
    ) {
        self.active_connection.remove_connection().await;
    }
    async fn payload(
        &self,
        _connection: &Arc<TcpSocketConnection<FixMessage, YourBourseFixTcpSerializer, YbTcpSate>>,
        contract: FixMessage,
    ) {
        match contract {
            FixMessage::Logon => println!("Logon by FIX-Feed"),
            FixMessage::Reject => {
                println!("Rejected by FIX-Feed");
            }
            FixMessage::Logout => {
                println!("Logged out from FIX-Feed");
            }
            FixMessage::MarketData(_) => panic!("We don't expect market data"),
            FixMessage::MarketDataReject(_) => {
                panic!("We don't expect market data reject")
            }
            FixMessage::ExecutionReport(report) => {
                let report: ExecutionReportModel = report.into();

                match report.ord_status {
                    yb_tcp_contracts::ExecutionReportModelStatus::PendingNew => {}
                    yb_tcp_contracts::ExecutionReportModelStatus::PartiallyFilled => {}
                    yb_tcp_contracts::ExecutionReportModelStatus::Filled => {
                        self.success_orders
                            .lock()
                            .await
                            .insert(report.internal_order_id.clone(), report);
                    }
                    yb_tcp_contracts::ExecutionReportModelStatus::Canceled => {
                        self.failed_orders
                            .lock()
                            .await
                            .insert(report.internal_order_id.clone(), report);
                    }
                    yb_tcp_contracts::ExecutionReportModelStatus::Rejected => {
                        self.failed_orders
                            .lock()
                            .await
                            .insert(report.internal_order_id.clone(), report);
                    }
                }
            }
            FixMessage::Others(data) => {
                println!("Others FIX-Feed: {:?}", data.to_string())
            }
            FixMessage::Pong => println!("Pong FIX-Feed"),
            FixMessage::Ping => println!("Ping FIX-Feed"),
            _ => {}
        }
    }
}
