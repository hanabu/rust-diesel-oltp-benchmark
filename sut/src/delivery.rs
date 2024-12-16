use axum::extract;
use if_types::{DeliveryRequest, DeliveryResponse};
use tpcc_models::Connection;

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn delivery(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<DeliveryRequest>,
) -> Result<axum::response::Json<DeliveryResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        let t0 = std::time::Instant::now();
        let (resp, t1, t2) = conn.transaction(|conn| {
            let t1 = std::time::Instant::now();
            let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, conn)?;
            let districts = warehouse.all_districts(conn)?;

            let mut total_delivered = 0;
            for district in &districts {
                let delivered_orders = district.delivery(params.carrier_id, conn)?;
                total_delivered += delivered_orders;
            }

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((
                axum::Json(DeliveryResponse {
                    deliverd_orders: total_delivered as i32,
                }),
                t1,
                t2,
            ))
        })?;

        let t3 = std::time::Instant::now();
        log::debug!(
            "delivery() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
            (t1 - t0).as_secs_f32(),
            (t2 - t1).as_secs_f32(),
            (t3 - t2).as_secs_f32(),
            (t3 - t0).as_secs_f32(),
        );

        Ok(resp)
    })
    .await?
}
