/// Transaction interface
///
/// Issue `BEGIN IMMEDIATE ... COMMIT`
/// to reduce write lock latency in SQLite
pub trait RwTransaction {
    fn read_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
        E: From<diesel::result::Error>;
    fn write_transaction<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
        E: From<diesel::result::Error>;
}
