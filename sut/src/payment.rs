use axum::extract;
use if_types::PaymentRequest;

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn payment(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<PaymentRequest>,
) -> Result<axum::response::Json<serde_json::Value>, crate::Error> {
    todo!()
}
