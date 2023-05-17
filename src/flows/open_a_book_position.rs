use std::sync::Arc;

use my_nosql_contracts::{BidAskSnapshotNoSqlEntity, TradingInstrumentNoSqlEntity};
use yb_tcp_contracts::ExecutionReportModel;

use crate::{
    a_book_bridge_grpc::{ABookBridgeOpenPositionGrpcRequest, ABookBridgePositionSide},
    AppContext, LP_NAME,
};

pub async fn open_a_book_position(
    app: &Arc<AppContext>,
    request: &ABookBridgeOpenPositionGrpcRequest,
) -> Result<ExecutionReportModel, String> {
    let Some(ns_mapping) = app.mapping_ns_reader.get_entity("im", &LP_NAME).await else{
        println!("LP mapping not found");
        return Err("LP not found".to_string());
    };

    let Some((external_instrument, _)) = ns_mapping.map.iter().find(|(_, internal)| {
        internal.to_string() == request.instrument_id
    }) else{
        println!("Instrument mapping not found");
        return Err("Instrument not found".to_string());
    };

    let Some(target_instrument) = app
        .instruments_ns_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &request.instrument_id,
        )
        .await else{
            return Err("Instrument not found".to_string());
        };

    let side: ABookBridgePositionSide = ABookBridgePositionSide::from_i32(request.side).unwrap();

    let price = get_price(&app, "USD", target_instrument).await;

    let volume = request.invest_amount * request.leverage / price;

    let result = app
        .fix_socket
        .place_order(
            &request.position_id,
            external_instrument,
            side.into(),
            volume,
        )
        .await;

    return Ok(result.unwrap());
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
