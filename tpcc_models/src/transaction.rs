use crate::DbConnection;

/// Transaction interface
///
/// Issue `BEGIN IMMEDIATE ... COMMIT`
/// to reduce write lock latency in SQLite
pub trait RwTransaction {
    fn read_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'b> FnOnce(&'b mut RdConnection<'b>) -> Result<T, E>,
        E: From<diesel::result::Error>;
    fn write_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'b> FnOnce(&'b mut WrConnection<'b>) -> Result<T, E>,
        E: From<diesel::result::Error>;
}

/// Database connection used only for read operation
pub struct RdConnection<'a>(&'a mut crate::DbConnection);
impl<'a> RdConnection<'a> {
    pub(crate) fn new<'b: 'a>(conn: &'b mut DbConnection) -> Self {
        Self(conn)
    }

    pub(crate) fn as_db(&mut self) -> &mut DbConnection {
        &mut self.0
    }
}

/// Database connection used both read and write operation
pub struct WrConnection<'a>(RdConnection<'a>);
impl<'a> WrConnection<'a> {
    pub(crate) fn new<'b: 'a>(conn: &'b mut DbConnection) -> Self {
        Self(RdConnection(conn))
    }

    pub(crate) fn as_db<'b: 'c, 'c>(&'b mut self) -> &'c mut DbConnection {
        &mut self.0 .0
    }

    pub(crate) fn transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'b> FnOnce(&'b mut WrConnection<'b>) -> Result<T, E>,
        E: From<diesel::result::Error>,
    {
        diesel::connection::Connection::transaction(self.0 .0, |db_conn| {
            f(&mut WrConnection::new(db_conn))
        })
    }
}

// Use WrConnection as RdConnection
impl<'a> std::ops::Deref for WrConnection<'a> {
    type Target = RdConnection<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for WrConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
