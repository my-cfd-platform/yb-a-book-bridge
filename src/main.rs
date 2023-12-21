mod app;
mod flows;
mod grpc;
mod settings;
mod tcp;

pub mod a_book_bridge_grpc {
    tonic::include_proto!("a_book_bridge");
}

pub use app::*;
pub use flows::*;
pub use grpc::*;
use my_tcp_sockets::TcpClient;
pub use settings::*;
use yb_tcp_contracts::{FixLogonCredentials, YourBourseFixTcpSerializer};

use std::sync::Arc;
pub use tcp::*;

use crate::a_book_bridge_grpc::a_book_bridge_grpc_service_server::ABookBridgeGrpcServiceServer;

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd-platform").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;

    let app_ctx = Arc::new(AppContext::new(&service_context, settings_reader).await);

    /*
       let tcp_client = TcpClient::new(
           "yourbourse - fix-trading-client".to_string(),
           app_ctx.clone(),
       );

       let credentials = FixLogonCredentials {
           password: settings.password.clone(),
           sender: settings.sender.clone(),
           target: settings.target.clone(),
       };

       tcp_client
           .start(
               Arc::new(move || -> YourBourseFixTcpSerializer {
                   YourBourseFixTcpSerializer::new(credentials.clone())
               }),
               app_ctx.a_book_tcp_processor.clone(),
               service_sdk::my_logger::LOGGER.clone(),
           )
           .await;
    */
    service_context.configure_grpc_server(|x| {
        x.add_grpc_service(ABookBridgeGrpcServiceServer::new(GrpcService::new(
            app_ctx.clone(),
        )));
    });

    service_context.start_application().await;
}
