use axum::extract;
use if_types::{PaymentRequest, PaymentResponse};
use tpcc_models::Connection;

/// Payment Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn payment(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<PaymentRequest>,
) -> Result<axum::response::Json<PaymentResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        let t0 = std::time::Instant::now();
        let (resp, t1, t2) = conn.transaction(|conn| {
            let t1 = std::time::Instant::now();
            // Search district, customer by ID
            let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, conn)?;
            let district = warehouse.find_district(params.district_id, conn)?;
            let customer = tpcc_models::Customer::find(
                params.customer_warehouse_id,
                params.customer_district_id,
                params.customer_id,
                conn,
            )?;

            // Payment transaction
            let (_updated_customer, history, _updated_district, _updated_warehouse) =
                customer.pay(&district, params.amount, conn)?;

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((
                axum::Json(if_types::PaymentResponse {
                    amount: params.amount,
                    paied_at: history.timestamp().and_utc(),
                }),
                t1,
                t2,
            ))
        })?;

        let t3 = std::time::Instant::now();
        log::debug!(
            "payment() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
            (t1 - t0).as_secs_f32(),
            (t2 - t1).as_secs_f32(),
            (t3 - t2).as_secs_f32(),
            (t3 - t0).as_secs_f32(),
        );

        Ok(resp)
    })
    .await?
}
