use crate::SpawnTransaction;
use axum::extract;
use if_types::CustomersResponse;

/// for Debug
pub(crate) async fn customer_by_id(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Path((warehouse_id, district_id, customer_id)): extract::Path<(i32, i32, i32)>,
) -> Result<axum::response::Json<if_types::Customer>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;

    let t0 = std::time::Instant::now();
    let (resp, t1, t2) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let t1 = std::time::Instant::now();

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

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((axum::Json(customer), t1, t2))
        })
        .await?;

    let t3 = std::time::Instant::now();
    log::debug!(
        "customer_by_id() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
        (t1 - t0).as_secs_f32(),
        (t2 - t1).as_secs_f32(),
        (t3 - t2).as_secs_f32(),
        (t3 - t0).as_secs_f32(),
    );

    let elapsed = (t3 - t0).as_micros() as usize;
    state.statistics.customer_by_id_count.fetch_add(1, Relaxed);
    state
        .statistics
        .customer_by_id_us
        .fetch_add(elapsed, Relaxed);

    Ok(resp)
}

/// Customer by last name, used in Payment, Order-Status Transaction
/// TPC-C standard spec. 2.5, 2.6
pub(crate) async fn customer_by_lastname(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Query(params): extract::Query<if_types::CustomersByLastnameParams>,
) -> Result<axum::response::Json<CustomersResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;
    let t0 = std::time::Instant::now();
    let (resp, t1, t2) = state
        .pool
        .spawn_read_transaction(move |conn| {
            let t1 = std::time::Instant::now();
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

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((
                axum::Json(if_types::CustomersResponse { customers }),
                t1,
                t2,
            ))
        })
        .await?;

    let t3 = std::time::Instant::now();
    log::debug!(
        "customer_by_lastname() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
        (t1 - t0).as_secs_f32(),
        (t2 - t1).as_secs_f32(),
        (t3 - t2).as_secs_f32(),
        (t3 - t0).as_secs_f32(),
    );

    let elapsed = (t3 - t0).as_micros() as usize;
    state
        .statistics
        .customer_by_name_count
        .fetch_add(1, Relaxed);
    state
        .statistics
        .customer_by_name_us
        .fetch_add(elapsed, Relaxed);

    Ok(resp)
}
