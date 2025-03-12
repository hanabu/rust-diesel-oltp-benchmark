use crate::SpawnTransaction;
use axum::extract;
use if_types::{StockLevelParams, StockLevelResponse};

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
pub(crate) async fn check_stocks(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Path((warehouse_id, district_id)): extract::Path<(i32, i32)>,
    extract::Query(params): extract::Query<StockLevelParams>,
) -> Result<axum::response::Json<StockLevelResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

            let warehouse = tpcc_models::Warehouse::find(warehouse_id, conn)?;
            let district = warehouse.find_district(district_id, conn)?;

            let low_stocks = district.check_stock_level(params.stock_level, conn)?;

            perflog.finish();
            Ok::<_, crate::Error>((
                if_types::StockLevelContents {
                    low_stocks: low_stocks as i32,
                },
                perflog,
            ))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "check_stocks() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.stock_level_count.fetch_add(1, Relaxed);
    state
        .statistics
        .stock_level_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(StockLevelResponse { contents, perf }))
}
