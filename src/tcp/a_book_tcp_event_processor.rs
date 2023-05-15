use std::{collections::HashMap, sync::Arc, time::Duration};

use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};
use yb_tcp_contracts::{
    ExecutionReportModel, FixIncomeMessage, FixMessage, FixOutcomeMessage, PlaceOrderYbTcpContract,
    PlaceOrderYbTcpContractSide, YourBourseFixTcpSerializer,
};

#[derive(Debug)]
pub enum PlaceOrderError {
    ConnectionNotFound,
    MaxIterationsReached,
}

pub struct ConnectionContainer {
    pub connection: RwLock<Option<Arc<SocketConnection<FixMessage, YourBourseFixTcpSerializer>>>>,
}

impl ConnectionContainer {
    pub fn new() -> Self {
        Self {
            connection: RwLock::new(None),
        }
    }

    pub async fn init_connection(
        &self,
        connection: Arc<SocketConnection<FixMessage, YourBourseFixTcpSerializer>>,
    ) {
        println!("Connection set");
        let mut connection_mut = self.connection.write().await;
        *connection_mut = Some(connection);
    }

    pub async fn remove_connection(&self) {
        println!("Connection removed");
        let mut connection_mut = self.connection.write().await;
        *connection_mut = None;
    }

    pub async fn get_connection(
        &self,
    ) -> Option<Arc<SocketConnection<FixMessage, YourBourseFixTcpSerializer>>> {
        let connection = self.connection.read().await;
        match &*connection {
            Some(connection) => Some(connection.clone()),
            None => None,
        }
    }
}

pub struct ABookTcpEventProcessor {
    active_connection: ConnectionContainer,
    created_orders: Mutex<HashMap<String, ExecutionReportModel>>,
}

impl ABookTcpEventProcessor {
    pub fn new() -> Self {
        Self {
            active_connection: ConnectionContainer::new(),
            created_orders: Mutex::new(HashMap::new()),
        }
    }

    async fn send_logon(&self) -> Result<(), PlaceOrderError> {
        let connection = self.get_connection().await?;
        connection
            .send(FixMessage::Outcome(FixOutcomeMessage::Logon))
            .await;

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
            .send(FixMessage::Outcome(FixOutcomeMessage::PlaceOrder(
                PlaceOrderYbTcpContract {
                    id: id.to_string(),
                    symbol: symbol.to_string(),
                    side,
                    qty,
                },
            )))
            .await;

        let mut count = 0;

        while count <= 10 {
            match self.created_orders.lock().await.remove(id) {
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

    async fn get_connection(
        &self,
    ) -> Result<Arc<SocketConnection<FixMessage, YourBourseFixTcpSerializer>>, PlaceOrderError>
    {
        let connection = self.active_connection.get_connection().await;

        if connection.is_none() {
            return Err(PlaceOrderError::ConnectionNotFound);
        }

        let connection = connection.as_ref().unwrap();

        return Ok(connection.clone());
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<FixMessage, YourBourseFixTcpSerializer> for ABookTcpEventProcessor {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<FixMessage, YourBourseFixTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                println!("Connected to FIX-Feed");
                self.active_connection.init_connection(connection).await;
                self.send_logon().await.unwrap();
            }
            ConnectionEvent::Disconnected(_) => {
                self.active_connection.remove_connection().await;
                println!("Disconnected to FIX-Feed");
            }
            ConnectionEvent::Payload {
                payload,
                connection,
            } => {
                if let FixMessage::Income(src) = &payload {
                    println!("Income: {:?}", src.to_string());
                }

                match payload {
                    FixMessage::Income(income_message) => match income_message {
                        FixIncomeMessage::Logon(_) => println!("Logon by FIX-Feed"),
                        FixIncomeMessage::Reject(_) => {
                            println!("Rejected by FIX-Feed");
                        }
                        FixIncomeMessage::Logout(_) => {
                            println!("Logged out from FIX-Feed");
                        }
                        FixIncomeMessage::MarketData(_) => panic!("We don't expect market data"),
                        FixIncomeMessage::MarketDataReject(_) => {
                            panic!("We don't expect market data reject")
                        }
                        FixIncomeMessage::ExecutionReport(report) => {
                            let report: ExecutionReportModel = report.into();
                            self.created_orders
                                .lock()
                                .await
                                .insert(report.internal_order_id.clone(), report);
                        }
                        FixIncomeMessage::Others(data) => {
                            println!("Others FIX-Feed: {:?}", data.to_string())
                        }
                        FixIncomeMessage::Pong => println!("Pong FIX-Feed"),
                        FixIncomeMessage::Ping => println!("Ping FIX-Feed"),
                    },
                    _ => {}
                }
            }
        }
    }
}
