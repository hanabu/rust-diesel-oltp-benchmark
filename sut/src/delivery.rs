use crate::SpawnTransaction;
use axum::extract;
use if_types::{DeliveryRequest, DeliveryResponse};

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn delivery(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Json(params): extract::Json<DeliveryRequest>,
) -> Result<axum::response::Json<DeliveryResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_write_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

            let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, conn)?;
            let districts = warehouse.all_districts(conn)?;

            let mut total_delivered = 0;
            for district in &districts {
                let delivered_orders = district.delivery(params.carrier_id, conn)?;
                total_delivered += delivered_orders;
            }

            perflog.finish();
            Ok::<_, crate::Error>((
                if_types::DeliveryContents {
                    deliverd_orders: total_delivered as i32,
                },
                perflog,
            ))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "delivery() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.delivery_count.fetch_add(1, Relaxed);
    state
        .statistics
        .delivery_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(DeliveryResponse { contents, perf }))
}
