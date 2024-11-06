use axum::extract;
use if_types::StockLevelResponse;

#[derive(serde::Deserialize)]
pub(crate) struct StockLevelParams {
    stock_level: i32,
}

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
pub(crate) async fn check_stocks(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Path((warehouse_id, district_id)): extract::Path<(i32, i32)>,
    extract::Query(params): extract::Query<StockLevelParams>,
) -> Result<axum::response::Json<StockLevelResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;

        let warehouse = tpcc_models::Warehouse::find(warehouse_id, &mut conn)?;
        let district = warehouse.find_district(district_id, &mut conn)?;

        let low_stocks = district.check_stock_level(params.stock_level, &mut conn)?;

        Ok(axum::Json(StockLevelResponse {
            low_stocks: low_stocks as i32,
        }))
    })
    .await?
}
