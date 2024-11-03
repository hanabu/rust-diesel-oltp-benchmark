mod new_order;

pub type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

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
        .with_state(pool)
}

async fn status(
    axum::extract::State(pool): axum::extract::State<tpcc_models::Pool>,
) -> axum::response::Json<serde_json::Value> {
    let _conn = pool.get().unwrap();
    axum::response::Json(serde_json::json!({}))
}
