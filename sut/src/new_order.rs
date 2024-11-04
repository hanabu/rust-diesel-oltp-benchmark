use axum::extract;
use if_types::NewOrderRequest;

/// New-Order Transaction
/// TPC-C standard spec. 2.4
pub(crate) async fn new_order(
    extract::State(pool): extract::State<tpcc_models::Pool>,
    extract::Json(params): extract::Json<NewOrderRequest>,
) -> Result<axum::response::Json<serde_json::Value>, crate::Error> {
    tokio::task::spawn_blocking(move || {
        use tpcc_models::Warehouse;
        let mut conn = pool.get()?;

        // Transaction described in TPC-C standard spec. 2.4.2
        let warehouse = Warehouse::find(params.warehouse_id, &mut conn)?;
        let mut district = warehouse.find_district(params.district_id, &mut conn)?;
        let customer = district.find_customer(params.customer_id, &mut conn)?;

        // Find order items
        let order_items = params
            .items
            .iter()
            .map(|item| {
                // ToDo : random select remote warehouse
                let stocked_item =
                    tpcc_models::StockedItem::find(params.warehouse_id, item.item_id, &mut conn)?;
                Ok((stocked_item, item.quantity))
            })
            .collect::<Result<Vec<_>, crate::Error>>()?;

        let _order = district.insert_order(&customer, &order_items, &mut conn)?;

        todo!()
    })
    .await?
}
