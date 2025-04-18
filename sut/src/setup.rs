use crate::SpawnTransaction;
use axum::extract;
use if_types::{DbStatusResponse, PrepareDbRequest};

pub(crate) async fn status(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
) -> Result<axum::response::Json<DbStatusResponse>, crate::Error> {
    let statistics = state.statistics.to_iftype();
    state
        .pool
        .spawn_read_transaction(move |conn| {
            let stat = DbStatusResponse {
                warehouse_count: tpcc_models::Warehouse::count(conn)?,
                district_count: tpcc_models::District::count(conn)?,
                customer_count: tpcc_models::Customer::count(conn)?,
                order_count: tpcc_models::Order::count(conn)?,
                database_bytes: tpcc_models::database_size(conn)?,
                statistics,
            };

            Ok::<_, crate::Error>(axum::response::Json(stat))
        })
        .await
}

/// Setup initial database
pub(crate) async fn prepare_db(
    extract::State(state): extract::State<std::sync::Arc<super::AppState>>,
    extract::Json(params): extract::Json<PrepareDbRequest>,
) -> Result<axum::response::Json<DbStatusResponse>, crate::Error> {
    use tpcc_models::RwTransaction;

    tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()?;
        conn.write_transaction(|conn| -> Result<(), crate::Error> {
            // Clean up database
            tpcc_models::cleanup(conn).map_err(|e| crate::Error::migration_error(e))?;
            // Setup schema (create table) and initial data
            tpcc_models::prepare(params.scale_factor, conn)
                .map_err(|e| crate::Error::migration_error(e))?;
            Ok(())
        })?;

        // Can not vacuum in transaction
        tpcc_models::vacuum(&mut conn)?;

        conn.read_transaction(|conn| {
            let stat = DbStatusResponse {
                warehouse_count: tpcc_models::Warehouse::count(conn)?,
                district_count: tpcc_models::District::count(conn)?,
                customer_count: tpcc_models::Customer::count(conn)?,
                order_count: tpcc_models::Order::count(conn)?,
                database_bytes: tpcc_models::database_size(conn)?,
                statistics: state.statistics.to_iftype(),
            };

            Ok(axum::response::Json(stat))
        })
    })
    .await?
}
