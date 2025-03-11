use axum::extract;
use if_types::{NewOrderRequest, NewOrderResponse};
use tpcc_models::RwTransaction;

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn new_order(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Json(params): extract::Json<NewOrderRequest>,
) -> Result<axum::response::Json<NewOrderResponse>, crate::Error> {
    use std::sync::atomic::Ordering::Relaxed;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()?;
        let t0 = std::time::Instant::now();
        let (resp, t1, t2) = conn.write_transaction(|conn| {
            use tpcc_models::Warehouse;

            let t1 = std::time::Instant::now();
            // Transaction described in TPC-C standard spec. 2.4.2
            let warehouse = Warehouse::find(params.warehouse_id, conn)?;
            let mut district = warehouse.find_district(params.district_id, conn)?;
            let customer = district.find_customer(params.customer_id, conn)?;

            // Find order items
            let order_items = params
                .items
                .iter()
                .map(|item| {
                    // ToDo : random select remote warehouse
                    let stocked_item =
                        tpcc_models::StockedItem::find(params.warehouse_id, item.item_id, conn)?;
                    Ok((stocked_item, item.quantity))
                })
                .collect::<Result<Vec<_>, crate::Error>>()?;

            // Insert into database
            let (order, lines) = district.insert_order(&customer, &order_items, conn)?;

            // Calc total amount including discount and tax
            let ol_amount = lines.iter().map(|ol| ol.amount()).sum::<f64>();
            let total_amount = ol_amount
                * (1.0 - customer.discount_rate())
                * (1.0 + warehouse.tax() + district.tax());

            let (warehouse_id, district_id, order_id) = order.id();
            let resp = NewOrderResponse {
                warehouse_id,
                district_id,
                order_id,
                total_amount,
            };

            let t2 = std::time::Instant::now();
            Ok::<_, crate::Error>((axum::Json(resp), t1, t2))
        })?;

        let t3 = std::time::Instant::now();
        log::debug!(
            "new_order() : Begin {:.03}s, Query {:.03}s, Commit {:03}s, Total {:03}s",
            (t1 - t0).as_secs_f32(),
            (t2 - t1).as_secs_f32(),
            (t3 - t2).as_secs_f32(),
            (t3 - t0).as_secs_f32(),
        );

        let elapsed = (t3 - t0).as_micros() as usize;
        state.statistics.new_order_count.fetch_add(1, Relaxed);
        state.statistics.new_order_us.fetch_add(elapsed, Relaxed);

        Ok(resp)
    })
    .await?
}
