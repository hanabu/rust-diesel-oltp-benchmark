use axum::extract;
use if_types::StockLevelResponse;
use tpcc_models::Connection;

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
pub(crate) async fn check_stocks(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Path((warehouse_id, district_id)): extract::Path<(i32, i32)>,
    extract::Query(params): extract::Query<if_types::StockLevelParams>,
) -> Result<axum::response::Json<StockLevelResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        conn.transaction(|conn| {
            let warehouse = tpcc_models::Warehouse::find(warehouse_id, conn)?;
            let district = warehouse.find_district(district_id, conn)?;

            let low_stocks = district.check_stock_level(params.stock_level, conn)?;

            Ok(axum::Json(StockLevelResponse {
                low_stocks: low_stocks as i32,
            }))
        })
    })
    .await?
}
