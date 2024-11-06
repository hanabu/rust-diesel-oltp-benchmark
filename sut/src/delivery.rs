use axum::extract;
use if_types::{DeliveryRequest, DeliveryResponse};

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn new_order(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<DeliveryRequest>,
) -> Result<axum::response::Json<DeliveryResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;

        let warehouse = tpcc_models::Warehouse::find(params.warehouse_id, &mut conn)?;
        let districts = warehouse.all_districts(&mut conn)?;

        let mut total_delivered = 0;
        for district in &districts {
            let delivered_orders = district.delivery(params.carrier_id, &mut conn)?;
            total_delivered += delivered_orders;
        }

        Ok(axum::Json(DeliveryResponse {
            deliverd_orders: total_delivered as i32,
        }))
    })
    .await?
}
