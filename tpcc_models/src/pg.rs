use diesel::prelude::*;
pub type DbConnection = diesel::pg::PgConnection;
pub type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<DbConnection>>;

/// Connect to database
pub fn connect(db_url: &str) -> ConnectionResult<DbConnection> {
    use diesel::prelude::Connection;

    DbConnection::establish(db_url)
}

/// Make database pool
pub fn pool(db_url: &str, connections: u32) -> Result<Pool, diesel::r2d2::PoolError> {
    let manager = diesel::r2d2::ConnectionManager::<DbConnection>::new(db_url);

    Pool::builder().max_size(connections).build(manager)
}

/// Run vacuum
pub fn vacuum(conn: &mut DbConnection) -> QueryResult<()> {
    use diesel::connection::SimpleConnection;
    conn.batch_execute("VACUUM;")?;
    Ok(())
}

/// Temporary return 0
pub fn database_size(_conn: &mut DbConnection) -> QueryResult<i64> {
    // SELECT pg_database_size('databaseName');
    Ok(0)
}
