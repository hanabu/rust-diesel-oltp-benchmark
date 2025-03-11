mod customer;
mod delivery;
mod new_order;
mod order_status;
mod payment;
mod setup;
mod spawn_transaction;
mod stock_level;

use spawn_transaction::SpawnTransaction;

/// Initialize & build Axum route
pub async fn app(db_connectinos: u32) -> axum::Router {
    use axum::routing::{get, post};

    println!("{} database connections", db_connectinos);
    let db_url = std::env::var("DATABASE_URL").unwrap_or("tpc_c.sqlite".to_string());
    let pool = tpcc_models::pool(&db_url, db_connectinos)
        .expect(&format!("Can not open database {}", db_url));
    let app_state = std::sync::Arc::new(AppState {
        pool,
        statistics: Statistics::default(),
    });

    axum::Router::new()
        .route("/orders", post(new_order::new_order))
        .route("/payment", post(payment::payment))
        .route(
            "/customers/:warehouse_id/:district_id/:customer_id/orders",
            get(order_status::order_status),
        )
        .route(
            "/customers/:warehouse_id/:district_id/:customer_id",
            get(customer::customer_by_id),
        )
        .route("/customers", get(customer::customer_by_lastname))
        .route("/delivery", post(delivery::delivery))
        .route(
            "/districts/:warehouse_id/:district_id/check_stocks",
            get(stock_level::check_stocks),
        )
        .route("/prepare_db", post(setup::prepare_db))
        .route("/", get(setup::status))
        .with_state(app_state)
}

struct AppState {
    pool: tpcc_models::Pool,
    statistics: Statistics,
}

/// Benchmark performance statistics
#[derive(Default)]
struct Statistics {
    new_order_count: std::sync::atomic::AtomicUsize,
    new_order_us: std::sync::atomic::AtomicUsize,
    payment_count: std::sync::atomic::AtomicUsize,
    payment_us: std::sync::atomic::AtomicUsize,
    order_status_count: std::sync::atomic::AtomicUsize,
    order_status_us: std::sync::atomic::AtomicUsize,
    delivery_count: std::sync::atomic::AtomicUsize,
    delivery_us: std::sync::atomic::AtomicUsize,
    stock_level_count: std::sync::atomic::AtomicUsize,
    stock_level_us: std::sync::atomic::AtomicUsize,
    customer_by_id_count: std::sync::atomic::AtomicUsize,
    customer_by_id_us: std::sync::atomic::AtomicUsize,
    customer_by_name_count: std::sync::atomic::AtomicUsize,
    customer_by_name_us: std::sync::atomic::AtomicUsize,
}

impl Statistics {
    fn to_iftype(&self) -> if_types::Statistics {
        use std::sync::atomic::Ordering::Relaxed;

        if_types::Statistics {
            new_order_count: self.new_order_count.load(Relaxed) as i64,
            new_order_secs: 0.000001 * self.new_order_us.load(Relaxed) as f64,
            payment_count: self.payment_count.load(Relaxed) as i64,
            payment_secs: 0.000001 * self.payment_us.load(Relaxed) as f64,
            order_status_count: self.order_status_count.load(Relaxed) as i64,
            order_status_secs: 0.000001 * self.order_status_us.load(Relaxed) as f64,
            delivery_count: self.delivery_count.load(Relaxed) as i64,
            delivery_secs: 0.000001 * self.delivery_us.load(Relaxed) as f64,
            stock_level_count: self.stock_level_count.load(Relaxed) as i64,
            stock_level_secs: 0.000001 * self.stock_level_us.load(Relaxed) as f64,
            customer_by_id_count: self.customer_by_id_count.load(Relaxed) as i64,
            customer_by_id_secs: 0.000001 * self.customer_by_id_us.load(Relaxed) as f64,
            customer_by_name_count: self.customer_by_name_count.load(Relaxed) as i64,
            customer_by_name_secs: 0.000001 * self.customer_by_name_us.load(Relaxed) as f64,
        }
    }
}

/// Error type in request handler
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database pool error")]
    DbPoolError(#[from] tpcc_models::PoolError),
    #[error("database query error")]
    DbQueryError(#[from] tpcc_models::QueryError),
    #[error("async runtime error")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("database schema setup error")]
    DbMigrationError(Box<dyn std::error::Error + Send + Sync>),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use tpcc_models::QueryError;
        match self {
            Error::DbQueryError(e) => match e {
                QueryError::NotFound => StatusCode::NOT_FOUND.into_response(),
                _ => {
                    log::error!("{:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
            _ => {
                log::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl Error {
    fn migration_error(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::DbMigrationError(e)
    }
}
