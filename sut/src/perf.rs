use if_types::PerformanceMetrics;

pub(crate) struct PerformanceLog {
    tm: [std::time::Instant; 4],
}

impl PerformanceLog {
    /// Call before BEGIN TRANSACTION
    pub fn new() -> Self {
        let now = std::time::Instant::now();

        Self {
            tm: [now, now, now, now],
        }
    }

    /// Call after BEGIN TRANSACTION
    pub fn begin(&mut self) {
        self.tm[1] = std::time::Instant::now();
    }

    /// Call before COMMIT
    pub fn finish(&mut self) {
        self.tm[2] = std::time::Instant::now();
    }

    /// Call after COMMIT
    pub fn commit(&mut self) {
        self.tm[3] = std::time::Instant::now();
    }

    /// Total time
    pub fn total_us(&self) -> usize {
        (self.tm[3] - self.tm[0]).as_micros() as usize
    }

    /// to PerformanceMetric
    pub fn to_performance_metric(&self) -> if_types::PerformanceMetrics {
        PerformanceMetrics {
            begin: (self.tm[1] - self.tm[0]).as_secs_f64(),
            query: (self.tm[2] - self.tm[1]).as_secs_f64(),
            commit: (self.tm[3] - self.tm[2]).as_secs_f64(),
        }
    }
}

/// Benchmark performance statistics
#[derive(Default)]
pub(crate) struct Statistics {
    pub(crate) new_order_count: std::sync::atomic::AtomicUsize,
    pub(crate) new_order_us: std::sync::atomic::AtomicUsize,
    pub(crate) payment_count: std::sync::atomic::AtomicUsize,
    pub(crate) payment_us: std::sync::atomic::AtomicUsize,
    pub(crate) order_status_count: std::sync::atomic::AtomicUsize,
    pub(crate) order_status_us: std::sync::atomic::AtomicUsize,
    pub(crate) delivery_count: std::sync::atomic::AtomicUsize,
    pub(crate) delivery_us: std::sync::atomic::AtomicUsize,
    pub(crate) stock_level_count: std::sync::atomic::AtomicUsize,
    pub(crate) stock_level_us: std::sync::atomic::AtomicUsize,
    pub(crate) customer_by_id_count: std::sync::atomic::AtomicUsize,
    pub(crate) customer_by_id_us: std::sync::atomic::AtomicUsize,
    pub(crate) customer_by_name_count: std::sync::atomic::AtomicUsize,
    pub(crate) customer_by_name_us: std::sync::atomic::AtomicUsize,
}

impl Statistics {
    pub fn to_iftype(&self) -> if_types::Statistics {
        use std::sync::atomic::Ordering::Relaxed;

        if_types::Statistics {
            new_order_count: self.new_order_count.load(Relaxed) as i64,
            new_order_secs: 0.000001 * self.new_order_us.load(Relaxed) as f64,
            payment_count: self.payment_count.load(Relaxed) as i64,
            payment_secs: 0.000001 * self.payment_us.load(Relaxed) as f64,
            order_status_count: self.order_status_count.load(Relaxed) as i64,
            order_status_secs: 0.000001 * self.order_status_us.load(Relaxed) as f64,
            delivery_count: self.delivery_count.load(Relaxed) as i64,
            delivery_secs: 0.000001 * self.delivery_us.load(Relaxed) as f64,
            stock_level_count: self.stock_level_count.load(Relaxed) as i64,
            stock_level_secs: 0.000001 * self.stock_level_us.load(Relaxed) as f64,
            customer_by_id_count: self.customer_by_id_count.load(Relaxed) as i64,
            customer_by_id_secs: 0.000001 * self.customer_by_id_us.load(Relaxed) as f64,
            customer_by_name_count: self.customer_by_name_count.load(Relaxed) as i64,
            customer_by_name_secs: 0.000001 * self.customer_by_name_us.load(Relaxed) as f64,
        }
    }
}
