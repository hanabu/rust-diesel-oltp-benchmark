mod customer;
mod delivery;
mod new_order;
mod order_status;
mod payment;
mod setup;
mod stock_level;

// pub type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

/// Initialize & build Axum route
pub async fn app(db_connectinos: u32) -> axum::Router {
    use axum::routing::{get, post};

    println!("{} database connections", db_connectinos);
    let db_url = std::env::var("DATABASE_URL").unwrap_or("tpc_c.sqlite".to_string());
    let pool = tpcc_models::pool(&db_url, db_connectinos)
        .expect(&format!("Can not open database {}", db_url));

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
        .with_state(pool)
}

/// Error type in request handler
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database schema setup error")]
    DbMigrationError(Box<dyn std::error::Error + Send + Sync>),
    #[error("database pool error")]
    DbPoolError(#[from] tpcc_models::PoolError),
    #[error("database query error")]
    DbQueryError(#[from] tpcc_models::QueryError),
    #[error("async runtime error")]
    TokioJoinError(#[from] tokio::task::JoinError),
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
