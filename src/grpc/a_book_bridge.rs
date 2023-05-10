use tonic::Response;

use crate::{
    a_book_bridge_grpc::{
        a_book_bridge_grpc_service_server::ABookBridgeGrpcService,
        ABookBridgeOpenPositionGrpcRequest, ABookBridgeOpenPositionGrpcResponsePositionModel,
        ABookBridgeOpenPositionResponse, ABookBridgePositionSide,
    },
    GrpcService, LP_NAME,
};

#[tonic::async_trait]
impl ABookBridgeGrpcService for GrpcService {
    async fn open_position(
        &self,
        request: tonic::Request<ABookBridgeOpenPositionGrpcRequest>,
    ) -> Result<tonic::Response<ABookBridgeOpenPositionResponse>, tonic::Status> {
        let request = request.into_inner();

        let Some(ns_mapping) = self.app.mapping_ns_reader.get_entity("im", &LP_NAME).await else{
            println!("LP mapping not found");
            return Err(tonic::Status::new(tonic::Code::NotFound, "LP not found"));
        };

        let Some((external_instrument, _)) = ns_mapping.map.iter().find(|(_, internal)| {
            internal.to_string() == request.instrument_id
        }) else{
            println!("Instrument mapping not found");
            return Err(tonic::Status::new(tonic::Code::NotFound, "Instrument not found"));
        };

        let side: ABookBridgePositionSide =
            ABookBridgePositionSide::from_i32(request.side).unwrap();

        let result = self
            .app
            .fix_socket
            .place_order(
                &request.position_id,
                external_instrument,
                side.into(),
                request.invest_amount * request.leverage,
            )
            .await;

        println!("Place order result: {:?}", result);

        let result = result.unwrap();

        let response = ABookBridgeOpenPositionResponse {
            status_code: 0,
            position: Some(ABookBridgeOpenPositionGrpcResponsePositionModel {
                account_id: request.account_id,
                internal_id: request.position_id,
                external_id: result.external_order_id,
                leverage: request.leverage,
                invest_amount: request.invest_amount,
                side: request.side,
                trade_date: result.trade_date.unwrap().parse().unwrap(),
                price: result.avg_price,
            }),
        };

        return Ok(Response::new(response));
    }
}
