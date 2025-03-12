use crate::SpawnTransaction;
use axum::extract;
use if_types::{NewOrderRequest, NewOrderResponse};

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn new_order(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Json(params): extract::Json<NewOrderRequest>,
) -> Result<axum::response::Json<NewOrderResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();

    let (contents, mut perflog) = state
        .pool
        .spawn_write_transaction(move |conn| {
            use tpcc_models::Warehouse;

            let mut perflog = perflog;
            perflog.begin();

            // Transaction described in TPC-C standard spec. 2.4.2
            let warehouse = Warehouse::find(params.warehouse_id, conn)?;
            let mut district = warehouse.find_district(params.district_id, conn)?;
            let customer = district.find_customer(params.customer_id, conn)?;

            // Find order items
            let order_items = params
                .items
                .iter()
                .map(|item| {
                    // ToDo : random select remote warehouse
                    let stocked_item =
                        tpcc_models::StockedItem::find(params.warehouse_id, item.item_id, conn)?;
                    Ok((stocked_item, item.quantity))
                })
                .collect::<Result<Vec<_>, crate::Error>>()?;

            // Insert into database
            let (order, lines) = district.insert_order(&customer, &order_items, conn)?;

            // Calc total amount including discount and tax
            let ol_amount = lines.iter().map(|ol| ol.amount()).sum::<f64>();
            let total_amount = ol_amount
                * (1.0 - customer.discount_rate())
                * (1.0 + warehouse.tax() + district.tax());

            let (warehouse_id, district_id, order_id) = order.id();
            let resp = if_types::NewOrderContents {
                warehouse_id,
                district_id,
                order_id,
                total_amount,
            };

            perflog.finish();
            Ok::<_, crate::Error>((resp, perflog))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "new_order() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.new_order_count.fetch_add(1, Relaxed);
    state
        .statistics
        .new_order_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(NewOrderResponse { contents, perf }))
}
