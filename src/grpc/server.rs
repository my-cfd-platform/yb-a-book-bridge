use crate::a_book_bridge_grpc::a_book_bridge_grpc_service_server::{ABookBridgeGrpcServiceServer};
use crate::app::AppContext;

use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

#[derive(Clone)]

pub struct GrpcService {
    pub app: Arc<AppContext>,
}

impl GrpcService {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

pub fn start(app: Arc<AppContext>, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let service = GrpcService::new(app);

    println!("Listening to {:?} as grpc endpoint", addr);

    tokio::spawn(async move {
        Server::builder()
            .add_service(ABookBridgeGrpcServiceServer::new(service.clone()))
            .serve(addr)
            .await
    });
}
