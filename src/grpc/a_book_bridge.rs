use tonic::Response;

use crate::{
    a_book_bridge_grpc::{
        a_book_bridge_grpc_service_server::ABookBridgeGrpcService,
        ABookBridgeOpenPositionGrpcRequest, ABookBridgeOpenPositionGrpcResponsePositionModel,
        ABookBridgeOpenPositionResponse, OpenPositionGrpcResponseStatusCode,
    },
    open_a_book_position, GrpcService,
};

#[tonic::async_trait]
impl ABookBridgeGrpcService for GrpcService {
    async fn open_position(
        &self,
        request: tonic::Request<ABookBridgeOpenPositionGrpcRequest>,
    ) -> Result<tonic::Response<ABookBridgeOpenPositionResponse>, tonic::Status> {
        let request = request.into_inner();
        let open_position_result = open_a_book_position(&self.app, &request).await;

        println!("Place order result: {:?}", open_position_result);

        let response = match open_position_result {
            Ok(result) => ABookBridgeOpenPositionResponse {
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
            },
            Err(err) => {
                let result: OpenPositionGrpcResponseStatusCode = err.into();
                ABookBridgeOpenPositionResponse {
                    status_code: result as i32,
                    position: None,
                }
            }
        };

        return Ok(Response::new(response));
    }

    async fn ping(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        return Ok(tonic::Response::new(()));
    }
}
