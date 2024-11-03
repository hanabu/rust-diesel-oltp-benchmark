use diesel::prelude::*;
pub type Connection = diesel::sqlite::SqliteConnection;

/// Connect to database
pub fn connect(db_url: &str) -> ConnectionResult<Connection> {
    use diesel::prelude::Connection;

    let mut conn = diesel::SqliteConnection::establish(db_url)?;

    // SQLite on NFS can not use WAL,
    // set journal_mode as default DELETE mode
    diesel::sql_query("PRAGMA journal_mode = DELETE")
        .execute(&mut conn)
        .map_err(|e| ConnectionError::CouldntSetupConfiguration(e))?;

    Ok(conn)
}
