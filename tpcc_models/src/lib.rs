mod models;
#[cfg(feature = "postgres")]
mod pg;
#[cfg(feature = "postgres")]
mod schema_pg;
#[cfg(not(any(feature = "postgres")))]
mod schema_sqlite;
#[cfg(not(any(feature = "postgres")))]
mod sqlite;

// Re-export Diesel types for error handling
pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as QueryError;
pub use diesel_migrations::MigrationError;
// Re-export Diesel Connection for conn.transaction()
pub use diesel::Connection;

pub use models::{cleanup, prepare};
pub use models::{Customer, District, Order, OrderLine, StockedItem, Warehouse};

#[cfg(feature = "postgres")]
pub use pg::{connect, database_size, pool, vacuum, DbConnection, Pool};
#[cfg(feature = "postgres")]
use schema_pg as schema;

#[cfg(not(any(feature = "postgres")))]
use schema_sqlite as schema;
#[cfg(not(any(feature = "postgres")))]
pub use sqlite::{connect, database_size, pool, vacuum, DbConnection, Pool};
