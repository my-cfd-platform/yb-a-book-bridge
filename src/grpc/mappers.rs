use yb_tcp_contracts::PlaceOrderYbTcpContractSide;

use crate::{a_book_bridge_grpc::{ABookBridgePositionSide, OpenPositionGrpcResponseStatusCode}, OpenABookPositionError};

impl Into<PlaceOrderYbTcpContractSide> for ABookBridgePositionSide {
    fn into(self) -> PlaceOrderYbTcpContractSide {
        match self {
            ABookBridgePositionSide::Buy => PlaceOrderYbTcpContractSide::Buy,
            ABookBridgePositionSide::Sell => PlaceOrderYbTcpContractSide::Sell,
        }
    }
}


impl Into<OpenPositionGrpcResponseStatusCode> for OpenABookPositionError {
    fn into(self) -> OpenPositionGrpcResponseStatusCode {
        match self{
            OpenABookPositionError::LiquidityProviderNotFound => OpenPositionGrpcResponseStatusCode::LiquidityProviderNotFound,
            OpenABookPositionError::InstrumentNotFoundInMapping => OpenPositionGrpcResponseStatusCode::InstrumentNotFoundInLpMapping,
            OpenABookPositionError::TradingInstrumentNotFound => OpenPositionGrpcResponseStatusCode::TradingInstrumentNotFound,
            OpenABookPositionError::LpReject => OpenPositionGrpcResponseStatusCode::LpReject,
            OpenABookPositionError::Timeout => OpenPositionGrpcResponseStatusCode::Timeout,
            OpenABookPositionError::Disconnect => OpenPositionGrpcResponseStatusCode::Disconnect,
        }
    }
}
