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
pub fn database_size(_conn: &mut crate::RdConnection) -> QueryResult<i64> {
    // SELECT pg_database_size('databaseName');
    Ok(0)
}

impl crate::RwTransaction for DbConnection {
    /// BEGIN TRANSACTION
    /// for Postgres, read_transaction() and write_transaction have no difference
    fn read_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        for<'b> F: FnOnce(&'b mut crate::RdConnection<'b>) -> Result<T, E>,
        E: From<diesel::result::Error>,
    {
        diesel::connection::Connection::transaction(self, |conn| {
            f(&mut crate::RdConnection::new(conn))
        })
    }

    /// BEGIN TRANSACTION
    /// for Postgres, read_transaction() and write_transaction have no difference
    fn write_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        for<'b> F: FnOnce(&'b mut crate::WrConnection<'b>) -> Result<T, E>,
        E: From<diesel::result::Error>,
    {
        diesel::connection::Connection::transaction(self, |conn| {
            f(&mut crate::WrConnection::new(conn))
        })
    }
}
