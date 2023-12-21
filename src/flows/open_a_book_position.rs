use std::sync::Arc;

use my_nosql_contracts::{BidAskSnapshotNoSqlEntity, TradingInstrumentNoSqlEntity};
use yb_tcp_contracts::{ExecutionReportModel, ExecutionReportModelStatus};

use crate::{
    a_book_bridge_grpc::{ABookBridgeOpenPositionGrpcRequest, ABookBridgePositionSide},
    AppContext, OpenABookPositionError, LP_NAME,
};

pub async fn open_a_book_position(
    app: &Arc<AppContext>,
    request: &ABookBridgeOpenPositionGrpcRequest,
) -> Result<ExecutionReportModel, OpenABookPositionError> {
    let Some(ns_mapping) = app.mapping_ns_reader.get_entity("im", &LP_NAME).await else {
        println!("LP mapping not found");
        return Err(OpenABookPositionError::LpReject);
    };

    let Some((external_instrument, _)) = ns_mapping
        .map
        .iter()
        .find(|(_, internal)| internal.to_string() == request.instrument_id)
    else {
        println!("Instrument mapping not found");
        return Err(OpenABookPositionError::InstrumentNotFoundInMapping);
    };

    let Some(target_instrument) = app
        .instruments_ns_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &request.instrument_id,
        )
        .await
    else {
        return Err(OpenABookPositionError::TradingInstrumentNotFound);
    };

    let side: ABookBridgePositionSide = ABookBridgePositionSide::from_i32(request.side).unwrap();

    let price = get_price(&app, "USD", target_instrument).await;

    let volume = request.invest_amount * request.leverage / price;

    let result = app
        .a_book_tcp_processor
        .place_order(
            &request.position_id,
            external_instrument,
            side.into(),
            volume,
        )
        .await;

    match result {
        Ok(result) => {
            if let ExecutionReportModelStatus::Rejected = result.ord_status {
                return Err(OpenABookPositionError::LpReject);
            }

            return Ok(result);
        }
        Err(err) => return Err(err.into()),
    }
}

pub async fn get_price(
    app: &Arc<AppContext>,
    account_currency: &str,
    target_instrument: Arc<TradingInstrumentNoSqlEntity>,
) -> f64 {
    if account_currency == target_instrument.base {
        return 1.0;
    }

    let instruments = app
        .instruments_ns_reader
        .get_entities(TradingInstrumentNoSqlEntity::generate_partition_key())
        .get_as_vec()
        .await
        .unwrap();

    if let Some(base_coll_instrument) = instruments
        .iter()
        .find(|x| x.base == target_instrument.base && x.quote == account_currency)
    {
        let bid_ask = app
            .bid_ask_ns_reader
            .get_entity(
                BidAskSnapshotNoSqlEntity::generate_partition_key(),
                base_coll_instrument.get_id(),
            )
            .await
            .unwrap();

        return bid_ask.bid;
    }

    if let Some(coll_base_instrument) = instruments
        .iter()
        .find(|x| x.base == account_currency && x.quote == target_instrument.base)
    {
        let bid_ask = app
            .bid_ask_ns_reader
            .get_entity(
                BidAskSnapshotNoSqlEntity::generate_partition_key(),
                coll_base_instrument.get_id(),
            )
            .await
            .unwrap();

        return 1.0 / bid_ask.bid;
    }

    return 1.0;
}
