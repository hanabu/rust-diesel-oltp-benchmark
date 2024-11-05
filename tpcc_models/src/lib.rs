mod models;
mod schema;
mod sqlite;

// Re-export Diesel types for error handling
pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as QueryError;
// Re-export Diesel Connection for conn.transaction()
pub use diesel::Connection;

pub use models::{cleanup, prepare_data, prepare_schema};
pub use models::{Customer, District, Order, OrderLine, StockedItem, Warehouse};
pub use sqlite::{connect, pool, vacuum, DbConnection, Pool};
