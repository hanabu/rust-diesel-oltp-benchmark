use axum::extract;

/// Order-Status Transaction
/// TPC-C standard spec. 2.6
pub(crate) async fn order_status(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Path((warehouse_id, district_id, customer_id)): extract::Path<(i32, i32, i32)>,
) -> Result<axum::response::Json<if_types::OrderStatusResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;

        // Search customer by ID
        let customer =
            tpcc_models::Customer::find(warehouse_id, district_id, customer_id, &mut conn)?;

        match customer.last_order(&mut conn) {
            Ok((db_order, db_lines)) => {
                // Re-share to response JSON type
                let lines = db_lines
                    .iter()
                    .map(|ol| if_types::OrderLine {
                        item_id: ol.item_id(),
                        supply_warehouse_id: ol.supply_warehouse_id(),
                        quantity: ol.quantity(),
                        amount: ol.amount(),
                        delivery_at: ol.delivery_at().map(|t| t.and_utc()),
                    })
                    .collect::<Vec<if_types::OrderLine>>();

                let (warehouse_id, district_id, order_id) = db_order.id();
                let order = if_types::Order {
                    warehouse_id,
                    district_id,
                    order_id,
                    entry_at: db_order.entry_at().and_utc(),
                    carrier_id: db_order.carrier_id(),
                    lines: lines,
                };

                Ok(axum::Json(if_types::OrderStatusResponse {
                    orders: vec![order],
                }))
            }
            Err(tpcc_models::QueryError::NotFound) => {
                Ok(axum::Json(if_types::OrderStatusResponse { orders: vec![] }))
            }
            Err(e) => Err(e)?,
        }
    })
    .await?
}
