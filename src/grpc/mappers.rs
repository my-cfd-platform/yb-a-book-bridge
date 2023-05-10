use yb_tcp_contracts::PlaceOrderYbTcpContractSide;

use crate::a_book_bridge_grpc::ABookBridgePositionSide;

impl Into<PlaceOrderYbTcpContractSide> for ABookBridgePositionSide {
    fn into(self) -> PlaceOrderYbTcpContractSide {
        match self {
            ABookBridgePositionSide::Buy => PlaceOrderYbTcpContractSide::Buy,
            ABookBridgePositionSide::Sell => PlaceOrderYbTcpContractSide::Sell,
        }
    }
}
