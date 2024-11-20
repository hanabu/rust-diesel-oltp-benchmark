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
        conn.transaction(|conn| {
            let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, conn)?;
            let districts = warehouse.all_districts(conn)?;

            let mut total_delivered = 0;
            for district in &districts {
                let delivered_orders = district.delivery(params.carrier_id, conn)?;
                total_delivered += delivered_orders;
            }

            Ok(axum::Json(DeliveryResponse {
                deliverd_orders: total_delivered as i32,
            }))
        })
    })
    .await?
}
