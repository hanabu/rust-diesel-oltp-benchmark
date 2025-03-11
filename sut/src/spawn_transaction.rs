use tpcc_models::RwTransaction;

/// Run transaction in dedicated thread
pub(crate) trait SpawnTransaction {
    /// Spawn thread and run read transaction
    async fn spawn_read_transaction<T, E, F>(&self, f: F) -> Result<T, crate::Error>
    where
        T: Send + 'static,
        F: for<'a> FnOnce(&'a mut tpcc_models::RdConnection<'a>) -> Result<T, E> + Send + 'static,
        E: From<tpcc_models::QueryError> + Send,
        crate::Error: From<E>;

    /// Spawn thread and run write transaction
    async fn spawn_write_transaction<T, E, F>(&self, f: F) -> Result<T, crate::Error>
    where
        T: Send + 'static,
        F: for<'a> FnOnce(&'a mut tpcc_models::WrConnection<'a>) -> Result<T, E> + Send + 'static,
        E: From<tpcc_models::QueryError> + Send,
        crate::Error: From<E>;
}

impl SpawnTransaction for tpcc_models::Pool {
    /// Run read transaction in
    async fn spawn_read_transaction<T, E, F>(&self, f: F) -> Result<T, crate::Error>
    where
        T: Send + 'static,
        F: for<'a> FnOnce(&'a mut tpcc_models::RdConnection<'a>) -> Result<T, E> + Send + 'static,
        E: From<tpcc_models::QueryError> + Send,
        crate::Error: From<E>,
    {
        let pool = self.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<T, crate::Error> {
            let mut conn = pool.get()?;
            let t = conn.read_transaction(f)?;
            Ok(t)
        })
        .await?;

        result
    }

    async fn spawn_write_transaction<T, E, F>(&self, f: F) -> Result<T, crate::Error>
    where
        T: Send + 'static,
        F: for<'a> FnOnce(&'a mut tpcc_models::WrConnection<'a>) -> Result<T, E> + Send + 'static,
        E: From<tpcc_models::QueryError> + Send,
        crate::Error: From<E>,
    {
        let pool = self.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<T, crate::Error> {
            let mut conn = pool.get()?;
            let t = conn.write_transaction(f)?;
            Ok(t)
        })
        .await?;

        result
    }
}
