use axum::extract;
use if_types::{PaymentRequest, PaymentResponse};

/// Payment Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn payment(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<PaymentRequest>,
) -> Result<axum::response::Json<PaymentResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;

        // Search district, customer by ID
        let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, &mut conn)?;
        let district = warehouse.find_district(params.district_id, &mut conn)?;
        let customer = tpcc_models::Customer::find(
            params.customer_warehouse_id,
            params.customer_district_id,
            params.customer_id,
            &mut conn,
        )?;

        // Payment transaction
        let (_updated_customer, history, _updated_district, _updated_warehouse) =
            customer.pay(&district, params.amount, &mut conn)?;

        Ok(axum::Json(if_types::PaymentResponse {
            amount: params.amount,
            paied_at: history.timestamp().and_utc(),
        }))
    })
    .await?
}
