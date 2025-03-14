/// New-Order Transaction input,
/// TPC-C standard spec. 2.4
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewOrderRequest {
    pub terminal_id: i32, // benchmark runner ID
    pub warehouse_id: i32,
    pub district_id: i32,
    pub customer_id: i32,
    pub items: Vec<NewOrderRequestItem>,
    pub inject_rollback: bool,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewOrderRequestItem {
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewOrderResponse {
    pub contents: NewOrderContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewOrderContents {
    pub warehouse_id: i32,
    pub district_id: i32,
    pub order_id: i32,
    pub total_amount: f64,
}

/// Payment Transaction input,
/// TPC-C standard spec. 2.5
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PaymentRequest {
    pub terminal_id: i32, // benchmark runner ID
    pub warehouse_id: i32,
    pub district_id: i32,
    pub customer_warehouse_id: i32,
    pub customer_district_id: i32,
    pub customer_id: i32,
    pub amount: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PaymentResponse {
    pub contents: PaymentContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PaymentContents {
    pub amount: f64,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub paied_at: chrono::DateTime<chrono::Utc>,
}

/// Order-Status Transaction output,
/// TPC-C standard spec. 2.6
#[derive(serde::Deserialize, serde::Serialize)]
pub struct OrderStatusResponse {
    pub contents: OrderStatusContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct OrderStatusContents {
    pub orders: Vec<Order>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Order {
    pub warehouse_id: i32,
    pub district_id: i32,
    pub order_id: i32,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub entry_at: chrono::DateTime<chrono::Utc>,
    pub carrier_id: Option<i32>,
    pub lines: Vec<OrderLine>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct OrderLine {
    pub item_id: i32,
    pub supply_warehouse_id: i32,
    pub quantity: i32,
    pub amount: f64,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub delivery_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Query parameter of customers_by_lastname
#[derive(serde::Deserialize, serde::Serialize)]
pub struct CustomersByLastnameParams {
    pub warehouse_id: i32,
    pub district_id: i32,
    pub lastname: String,
}

/// Response of customers_by_lastname
/// TPC-C standard spec. 2.5, 2.6
#[derive(serde::Deserialize, serde::Serialize)]
pub struct CustomersResponse {
    pub contents: CustomersContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct CustomersContents {
    pub customers: Vec<Customer>,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Customer {
    pub warehouse_id: i32,
    pub district_id: i32,
    pub customer_id: i32,
    pub firstname: String,
    pub lastname: String,
}

/// Delivery Transaction input,
/// TPC-C standard spec. 2.7
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DeliveryRequest {
    pub warehouse_id: i32,
    pub carrier_id: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DeliveryResponse {
    pub contents: DeliveryContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DeliveryContents {
    pub deliverd_orders: i32,
}

/// Stock-Level Transaction input,
/// TPC-C standard spec. 2.8
#[derive(serde::Deserialize, serde::Serialize)]
pub struct StockLevelParams {
    pub stock_level: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct StockLevelResponse {
    pub contents: StockLevelContents,
    pub perf: PerformanceMetrics,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct StockLevelContents {
    pub low_stocks: i32,
}

/// Setup initial database
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PrepareDbRequest {
    pub scale_factor: i32,
}

/// Performance metrics
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PerformanceMetrics {
    /// Time to take BEGIN TRANSACTION
    pub begin: f64,
    /// Query (SELECT, INSERT, UPDATE, DELETE)
    pub query: f64,
    /// Commit transaction
    pub commit: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DbStatusResponse {
    pub warehouse_count: i64,
    pub district_count: i64,
    pub customer_count: i64,
    pub order_count: i64,
    pub database_bytes: i64,
    pub statistics: Statistics,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Statistics {
    pub new_order_count: i64,
    pub new_order_secs: f64,
    pub payment_count: i64,
    pub payment_secs: f64,
    pub order_status_count: i64,
    pub order_status_secs: f64,
    pub delivery_count: i64,
    pub delivery_secs: f64,
    pub stock_level_count: i64,
    pub stock_level_secs: f64,
    pub customer_by_id_count: i64,
    pub customer_by_id_secs: f64,
    pub customer_by_name_count: i64,
    pub customer_by_name_secs: f64,
}
