use crate::SpawnTransaction;
use axum::extract;

/// for Debug
pub(crate) async fn customer_by_id(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Path((warehouse_id, district_id, customer_id)): extract::Path<(i32, i32, i32)>,
) -> Result<axum::response::Json<if_types::CustomersResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

            // Search customer by ID
            let db_customer =
                tpcc_models::Customer::find(warehouse_id, district_id, customer_id, conn)?;

            // Re-share to response JSON type
            let (warehouse_id, district_id, customer_id) = db_customer.id();
            let customer = if_types::Customer {
                warehouse_id,
                district_id,
                customer_id,
                firstname: db_customer.firstname().to_string(),
                lastname: db_customer.lastname().to_string(),
            };

            perflog.finish();
            Ok::<_, crate::Error>((
                if_types::CustomersContents {
                    customers: vec![customer],
                },
                perflog,
            ))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();

    log::debug!(
        "customer_by_id() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit
    );

    state.statistics.customer_by_id_count.fetch_add(1, Relaxed);
    state
        .statistics
        .customer_by_id_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(if_types::CustomersResponse { contents, perf }))
}

/// Customer by last name, used in Payment, Order-Status Transaction
/// TPC-C standard spec. 2.5, 2.6
pub(crate) async fn customer_by_lastname(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Query(params): extract::Query<if_types::CustomersByLastnameParams>,
) -> Result<axum::response::Json<if_types::CustomersResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let perflog = crate::PerformanceLog::new();
    let (contents, mut perflog) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let mut perflog = perflog;
            perflog.begin();

            // Search customer by lastname
            let db_customers = tpcc_models::Customer::find_by_name(
                params.warehouse_id,
                params.district_id,
                &params.lastname,
                conn,
            )?;

            // Re-share to response JSON type
            let customers = db_customers
                .iter()
                .map(|c| {
                    let (warehouse_id, district_id, customer_id) = c.id();
                    if_types::Customer {
                        warehouse_id,
                        district_id,
                        customer_id,
                        firstname: c.firstname().to_string(),
                        lastname: c.lastname().to_string(),
                    }
                })
                .collect::<Vec<_>>();

            perflog.finish();
            Ok::<_, crate::Error>((if_types::CustomersContents { customers }, perflog))
        })
        .await?;

    perflog.commit();
    let perf = perflog.to_performance_metric();
    log::debug!(
        "customer_by_lastname() : Begin {:.03}s, Query {:.03}s, Commit {:03}s",
        perf.begin,
        perf.query,
        perf.commit,
    );

    state
        .statistics
        .customer_by_name_count
        .fetch_add(1, Relaxed);
    state
        .statistics
        .customer_by_name_us
        .fetch_add(perflog.total_us(), Relaxed);

    Ok(axum::Json(if_types::CustomersResponse { contents, perf }))
}
