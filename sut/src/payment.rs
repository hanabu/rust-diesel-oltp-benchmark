use crate::SpawnTransaction;
use axum::extract;
use if_types::{PaymentRequest, PaymentResponse};

/// Payment Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn payment(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Json(params): extract::Json<PaymentRequest>,
) -> Result<axum::response::Json<PaymentResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_write_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

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

            perflog.finish();
            Ok::<_, crate::Error>((
                if_types::PaymentContents {
                    amount: params.amount,
                    paied_at: history.timestamp().and_utc(),
                },
                perflog,
            ))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "payment() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.payment_count.fetch_add(1, Relaxed);
    state
        .statistics
        .payment_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(PaymentResponse { contents, perf }))
}
