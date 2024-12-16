use axum::extract;
use tpcc_models::Connection;

/// Order-Status Transaction
/// TPC-C standard spec. 2.6
pub(crate) async fn order_status(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Path((warehouse_id, district_id, customer_id)): extract::Path<(i32, i32, i32)>,
) -> Result<axum::response::Json<if_types::OrderStatusResponse>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        let t0 = std::time::Instant::now();
        let (resp, t1, t2) = conn.transaction(|conn| {
            let t1 = std::time::Instant::now();
            // Search customer by ID
            let customer =
                tpcc_models::Customer::find(warehouse_id, district_id, customer_id, conn)?;

            match customer.last_order(conn) {
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

                    let t2 = std::time::Instant::now();
                    Ok::<_, crate::Error>((
                        axum::Json(if_types::OrderStatusResponse {
                            orders: vec![order],
                        }),
                        t1,
                        t2,
                    ))
                }
                Err(tpcc_models::QueryError::NotFound) => {
                    let t2 = std::time::Instant::now();
                    Ok((
                        axum::Json(if_types::OrderStatusResponse { orders: vec![] }),
                        t1,
                        t2,
                    ))
                }
                Err(e) => Err(e)?,
            }
        })?;

        let t3 = std::time::Instant::now();
        log::debug!(
            "order_status() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
            (t1 - t0).as_secs_f32(),
            (t2 - t1).as_secs_f32(),
            (t3 - t2).as_secs_f32(),
            (t3 - t0).as_secs_f32(),
        );

        Ok(resp)
    })
    .await?
}
