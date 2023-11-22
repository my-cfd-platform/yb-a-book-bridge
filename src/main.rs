use std::sync::Arc;

use yb_a_book_bridge::{
    a_book_bridge_grpc::a_book_bridge_grpc_service_server::ABookBridgeGrpcServiceServer,
    AppContext, GrpcService, SettingsReader,
};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;

    let app_ctx = Arc::new(AppContext::new(&service_context, settings_reader).await);
    app_ctx.start().await;

    service_context.configure_grpc_server(|x| {
        x.add_grpc_service(ABookBridgeGrpcServiceServer::new(GrpcService::new(
            app_ctx.clone(),
        )));
    });

    service_context.start_application().await;
}

