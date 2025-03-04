use axum::extract;
use if_types::StockLevelResponse;
use tpcc_models::Connection;

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
pub(crate) async fn check_stocks(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Path((warehouse_id, district_id)): extract::Path<(i32, i32)>,
    extract::Query(params): extract::Query<if_types::StockLevelParams>,
) -> Result<axum::response::Json<StockLevelResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()?;
        let t0 = std::time::Instant::now();
        let (resp, t1, t2) = conn.transaction(|conn| {
            let t1 = std::time::Instant::now();
            let warehouse = tpcc_models::Warehouse::find(warehouse_id, conn)?;
            let district = warehouse.find_district(district_id, conn)?;

            let low_stocks = district.check_stock_level(params.stock_level, conn)?;

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((
                axum::Json(StockLevelResponse {
                    low_stocks: low_stocks as i32,
                }),
                t1,
                t2,
            ))
        })?;
        let t3 = std::time::Instant::now();
        log::debug!(
            "check_stocks() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
            (t1 - t0).as_secs_f32(),
            (t2 - t1).as_secs_f32(),
            (t3 - t2).as_secs_f32(),
            (t3 - t0).as_secs_f32(),
        );

        let elapsed = (t3 - t0).as_micros() as usize;
        state.statistics.stock_level_count.fetch_add(1, Relaxed);
        state.statistics.stock_level_us.fetch_add(elapsed, Relaxed);

        Ok(resp)
    })
    .await?
}
