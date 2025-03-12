use crate::SpawnTransaction;
use axum::extract;
use if_types::OrderStatusResponse;

/// Order-Status Transaction
/// TPC-C standard spec. 2.6
pub(crate) async fn order_status(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Path((warehouse_id, district_id, customer_id)): extract::Path<(i32, i32, i32)>,
) -> Result<axum::response::Json<OrderStatusResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

            // Search customer by ID
            let customer =
                tpcc_models::Customer::find(warehouse_id, district_id, customer_id, conn)?;

            match customer.last_order(conn) {
                Ok((db_order, db_lines)) => {
                    // Re-share to response JSON type
                    let lines = db_lines
                        .iter()
                        .map(|ol| if_types::OrderLine {
                            item_id: ol.item_id(),
                            supply_warehouse_id: ol.supply_warehouse_id(),
                            quantity: ol.quantity(),
                            amount: ol.amount(),
                            delivery_at: ol.delivery_at().map(|t| t.and_utc()),
                        })
                        .collect::<Vec<if_types::OrderLine>>();

                    let (warehouse_id, district_id, order_id) = db_order.id();
                    let order = if_types::Order {
                        warehouse_id,
                        district_id,
                        order_id,
                        entry_at: db_order.entry_at().and_utc(),
                        carrier_id: db_order.carrier_id(),
                        lines: lines,
                    };

                    perflog.finish();
                    Ok::<_, crate::Error>((
                        if_types::OrderStatusContents {
                            orders: vec![order],
                        },
                        perflog,
                    ))
                }
                Err(tpcc_models::QueryError::NotFound) => {
                    perflog.finish();
                    Ok((if_types::OrderStatusContents { orders: vec![] }, perflog))
                }
                Err(e) => Err(e)?,
            }
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "order_status() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.order_status_count.fetch_add(1, Relaxed);
    state
        .statistics
        .order_status_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(OrderStatusResponse { contents, perf }))
}
