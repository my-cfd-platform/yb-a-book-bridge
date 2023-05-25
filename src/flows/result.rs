use crate::PlaceOrderError;

#[derive(Debug)]
pub enum OpenABookPositionError{
    LiquidityProviderNotFound,
    InstrumentNotFoundInMapping,
    TradingInstrumentNotFound,
    LpReject,
    Timeout,
    Disconnect
}

impl Into<OpenABookPositionError> for PlaceOrderError {
    fn into(self) -> OpenABookPositionError {
        match self{
            PlaceOrderError::ConnectionNotFound => OpenABookPositionError::Disconnect,
            PlaceOrderError::MaxIterationsReached => OpenABookPositionError::Timeout,
        }
    }
}