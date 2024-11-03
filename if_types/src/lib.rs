/// New-Order Transaction input,
/// TPC-C standard spec. 2.4
#[derive(serde::Deserialize)]
pub struct NewOrderRequest {
    pub terminal_id: i32, // benchmark runner ID
    pub warehouse_id: i32,
    pub district_id: i32,
    pub customer_id: i32,
    pub items: Vec<NewOrderRequestItem>,
}

#[derive(serde::Deserialize)]
pub struct NewOrderRequestItem {
    pub item_id: i32,
    pub quantity: i32,
}
