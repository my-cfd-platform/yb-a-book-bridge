use std::sync::Arc;

use service_sdk::ServiceContext;
use yb_a_book_bridge::{
    a_book_bridge_grpc::a_book_bridge_grpc_service_server::ABookBridgeGrpcServiceServer,
    AppContext, GrpcService, SettingsReader,
};

#[tokio::main]
async fn main() {
    let settings_reader = Arc::new(SettingsReader::new(".my-cfd").await);
    let settings = Arc::new(settings_reader.get_settings().await);

    let mut service_context = ServiceContext::new(settings_reader);
    let app_ctx = Arc::new(AppContext::new(&service_context, settings).await);
    app_ctx.start().await;
    service_context.add_grpc_service(ABookBridgeGrpcServiceServer::new(GrpcService::new(app_ctx)));

    service_context.start_application().await;
}
