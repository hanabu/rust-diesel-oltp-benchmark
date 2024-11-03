use axum::extract;
use if_types::NewOrderRequest;

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn new_order(
    extract::State(_pool): extract::State<tpcc_models::Pool>,
    extract::Json(_params): extract::Json<NewOrderRequest>,
) -> axum::response::Json<serde_json::Value> {
    //let mut conn = pool.get().unwrap();
    todo!()
}
