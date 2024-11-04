mod customer;
mod new_order;
mod order_status;
mod payment;

// pub type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

/// Initialize & build Axum route
pub async fn app() -> axum::Router {
    use axum::routing::{get, post};

    // If you need S3 client, databae connection etc. initialize here.
    //let sdk_cfg = aws_config::load_from_env().await;
    //let s3_client = aws_sdk_s3::Client::new(&sdk_cfg);
    let db_url = std::env::var("DATABASE_URL").unwrap_or("tpc_c.sqlite".to_string());
    let pool = tpcc_models::pool(&db_url, 1).expect(&format!("Can not open database {}", db_url));

    axum::Router::new()
        .route("/", get(status))
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
        .with_state(pool)
}

async fn status(
    axum::extract::State(pool): axum::extract::State<tpcc_models::Pool>,
) -> Result<axum::response::Json<serde_json::Value>, Error> {
    let _conn = pool.get()?;
    Ok(axum::response::Json(serde_json::json!({})))
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
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use tpcc_models::QueryError;
        match self {
            Error::DbQueryError(e) => match e {
                QueryError::NotFound => StatusCode::NOT_FOUND.into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
