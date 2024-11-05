use diesel::prelude::*;
pub type DbConnection = diesel::sqlite::SqliteConnection;
pub type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<DbConnection>>;

/// Connect to database
pub fn connect(db_url: &str) -> ConnectionResult<DbConnection> {
    use diesel::prelude::Connection;

    let mut conn = diesel::SqliteConnection::establish(db_url)?;
    setup_conn(&mut conn).map_err(|e| ConnectionError::CouldntSetupConfiguration(e))?;
    Ok(conn)
}

/// Make database pool
pub fn pool(db_url: &str, connections: u32) -> Result<Pool, diesel::r2d2::PoolError> {
    let manager = diesel::r2d2::ConnectionManager::<DbConnection>::new(db_url);

    Pool::builder()
        .max_size(connections)
        .connection_customizer(Box::new(CustomOptions()))
        .build(manager)
}

/// Customize Sqlite options
#[derive(Debug)]
struct CustomOptions();
impl diesel::r2d2::CustomizeConnection<DbConnection, diesel::r2d2::Error> for CustomOptions {
    fn on_acquire(&self, conn: &mut DbConnection) -> Result<(), diesel::r2d2::Error> {
        setup_conn(conn).map_err(diesel::r2d2::Error::QueryError)
    }
}

/// SQLite setup in initial connection
fn setup_conn(conn: &mut DbConnection) -> QueryResult<()> {
    use diesel::connection::SimpleConnection;
    // SQLite on NFS can not use WAL, set journal_mode as default DELETE mode
    conn.batch_execute("PRAGMA journal_mode = DELETE;")?;
    // Force foreign key constraint
    conn.batch_execute("PRAGMA foreign_keys = ON;")?;
    // Timeout
    conn.batch_execute("PRAGMA busy_timeout = 1000;")?; // mili-sec
    Ok(())
}

/// SQLite setup in initial connection
pub fn vacuum(conn: &mut DbConnection) -> QueryResult<()> {
    use diesel::connection::SimpleConnection;
    conn.batch_execute("VACUUM;")?;
    Ok(())
}
