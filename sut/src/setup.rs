use axum::extract;
use if_types::{DbStatusResponse, PrepareDbRequest};

pub(crate) async fn status(
    extract::State(pool): extract::State<tpcc_models::Pool>,
) -> Result<axum::response::Json<DbStatusResponse>, crate::Error> {
    let mut conn = pool.get()?;

    let stat = DbStatusResponse {
        warehouse_count: tpcc_models::Warehouse::count(&mut conn)?,
        district_count: tpcc_models::District::count(&mut conn)?,
        customer_count: tpcc_models::Customer::count(&mut conn)?,
        order_count: tpcc_models::Order::count(&mut conn)?,
        database_bytes: tpcc_models::database_size(&mut conn)?,
    };

    Ok(axum::response::Json(stat))
}

/// Setup initial database
pub(crate) async fn prepare_db(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<PrepareDbRequest>,
) -> Result<axum::response::Json<serde_json::Value>, crate::Error> {
    let mut conn = pool.get()?;

    // Clean up database
    tpcc_models::cleanup(&mut conn).map_err(|e| crate::Error::migration_error(e))?;
    // Setup schema (create table) and initial data
    tpcc_models::prepare(params.scale_factor, &mut conn)
        .map_err(|e| crate::Error::migration_error(e))?;
    tpcc_models::vacuum(&mut conn)?;

    Ok(axum::response::Json(serde_json::json!({})))
}
