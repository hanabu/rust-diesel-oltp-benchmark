use crate::{schema, DbConnection};
use diesel::prelude::*;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");

/// Cleanup existing data
pub fn cleanup(conn: &mut DbConnection) -> diesel::migration::Result<()> {
    use diesel_migrations::MigrationHarness;

    // Run migration
    conn.revert_all_migrations(MIGRATIONS)?;

    Ok(())
}

/// Run database migration, prepare initial records
pub fn prepare(scale_factor: i32, conn: &mut DbConnection) -> diesel::migration::Result<()> {
    use diesel_migrations::MigrationHarness;

    // Run migration
    conn.run_pending_migrations(MIGRATIONS)?;

    // Prepare initial records
    let mut rand = tpcc_rand::TpcRandom::new();

    // TPC-C standard spec. 4.3.3, fixed 100_000 items
    Item::prepare(100_000, &mut rand, conn)?;

    for _i in 0..scale_factor {
        let warehouse = Warehouse::prepare(&mut rand, conn)?;
        // TPC-C standard spec. 4.3.3, each warehouse has
        //   100_000 stocks, 10 districts
        warehouse.prepare_stocks(100_000, &mut rand, conn)?;
        let districts = warehouse.prepare_districts(10, &mut rand, conn)?;
        for district in districts {
            // TPC-C standard spec. 4.3.3, each district has
            //   3_000 customers, 3_000 orders
            district.prepare_customers(3_000, &mut rand, conn)?;
            district.prepare_orders(3_000, &mut rand, conn)?;
        }
    }

    Ok(())
}

/// Sales item
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::items)]
pub struct Item {
    i_id: i32,
    i_im_id: i32,
    i_name: String,
    i_price: f64,
    i_data: String,
}

impl Item {
    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::items::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    // Prepare initial Items
    pub fn prepare(
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::items;
        let cur_id = items::table
            .select(diesel::dsl::max(items::i_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        let prepared_items = (0..num)
            .map(|i| {
                // TPC-C standard spec. 4.3.3
                Self {
                    i_id: cur_id + i + 1,
                    i_im_id: rand.i32_range(1..=10_000),
                    i_name: rand.alnum_string(14..=24),
                    i_price: rand.f64_range(1.00..=100.00),
                    i_data: rand.item_data(),
                }
            })
            .collect::<Vec<Self>>();

        diesel::insert_into(items::table)
            .values(&prepared_items)
            .execute(conn)?;
        Ok(prepared_items)
    }
}

/// Warehouse
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::warehouses)]
pub struct Warehouse {
    w_id: i32,
    w_name: String,
    w_street_1: String,
    w_street_2: String,
    w_city: String,
    w_state: String,
    w_zip: String,
    w_tax: f64,
    w_ytd: f64,
}

impl Warehouse {
    /// Get Warehouse by it's id
    pub fn find(id: i32, conn: &mut DbConnection) -> QueryResult<Self> {
        schema::warehouses::table.find(id).first(conn)
    }

    /// Get District
    pub fn find_district(
        &self,
        district_id: i32,
        conn: &mut DbConnection,
    ) -> QueryResult<District> {
        District::find(self.w_id, district_id, conn)
    }

    /// All districts under this warehouse
    pub fn all_districts(&self, conn: &mut DbConnection) -> QueryResult<Vec<District>> {
        District::all_by_warehouse(self.w_id, conn)
    }

    /// Get tax rate of the warehouse
    pub fn tax(&self) -> f64 {
        self.w_tax
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::warehouses::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    /// Prepare Warehose
    pub fn prepare(rand: &mut tpcc_rand::TpcRandom, conn: &mut DbConnection) -> QueryResult<Self> {
        use schema::warehouses;
        let cur_id = warehouses::table
            .select(diesel::dsl::max(warehouses::w_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        // TPC-C standard spec. 4.3.3
        let prepared_warehouse = Self {
            w_id: cur_id + 1,
            w_name: rand.alnum_string(6..=10),
            w_street_1: rand.alnum_string(10..=20),
            w_street_2: rand.alnum_string(10..=20),
            w_city: rand.alnum_string(10..=20),
            w_state: rand.alnum_string(2..=2),
            w_zip: rand.zip_code(),
            w_tax: rand.f64_range(0.0..=0.2),
            w_ytd: 300_000.0,
        };

        diesel::insert_into(warehouses::table)
            .values(&prepared_warehouse)
            .execute(conn)?;
        Ok(prepared_warehouse)
    }

    /// Insert stock in warehouse
    /// In TPC-C standard, each warehouse has 100_000 stocks.
    pub fn prepare_stocks(
        &self,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Stock>> {
        Stock::prepare(self.w_id, num, rand, conn)
    }

    /// Insert district in warehouse
    /// In TPC-C standard, each warehouse has 10 districts.
    pub fn prepare_districts(
        &self,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<District>> {
        District::prepare(self.w_id, num, rand, conn)
    }
}

/// Stock in Warehouse
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::stocks)]
pub struct Stock {
    s_i_id: i32,
    s_w_id: i32,
    s_quantity: i32,
    s_dist_01: String,
    s_dist_02: String,
    s_dist_03: String,
    s_dist_04: String,
    s_dist_05: String,
    s_dist_06: String,
    s_dist_07: String,
    s_dist_08: String,
    s_dist_09: String,
    s_dist_10: String,
    s_ytd: i32,
    s_order_cnt: i32,
    s_remote_cnt: i32,
    s_data: String,
}

impl Stock {
    /// TPC-C standard spec. 2.4.2.2
    /// Allocate stock by New Order transaction
    fn allocate(
        &self,
        quantity: i32,
        order_by_warehouse_id: i32,
        conn: &mut DbConnection,
    ) -> QueryResult<Self> {
        use schema::stocks;
        let new_qty = if self.s_quantity > quantity + 10 {
            self.s_quantity - quantity
        } else {
            self.s_quantity - quantity + 91
        };

        let remote_inc = if self.s_w_id == order_by_warehouse_id {
            0 // home order
        } else {
            1 // remote order
        };

        let row = stocks::table
            .filter(stocks::s_w_id.eq(self.s_w_id))
            .filter(stocks::s_i_id.eq(self.s_i_id));
        let updated_stock = diesel::update(row)
            .set((
                stocks::s_quantity.eq(new_qty),
                stocks::s_ytd.eq(stocks::s_ytd + quantity),
                stocks::s_order_cnt.eq(stocks::s_order_cnt + 1),
                stocks::s_remote_cnt.eq(stocks::s_remote_cnt + remote_inc),
            ))
            .get_result(conn)?;

        Ok(updated_stock)
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::stocks::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    /// Insert new stocks
    ///   public API: call warehouse.insert_stocks() instead.
    fn prepare(
        warehouse_id: i32,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::stocks;
        let cur_id = stocks::table
            .filter(stocks::s_w_id.eq(warehouse_id))
            .select(diesel::dsl::max(stocks::s_i_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        let prepared_stocks = (0..num)
            .map(|i| {
                // TPC-C standard spec. 4.3.3
                Self {
                    s_i_id: cur_id + i + 1,
                    s_w_id: warehouse_id,
                    s_quantity: rand.i32_range(10..=100),
                    s_dist_01: rand.alnum_string(24..=24),
                    s_dist_02: rand.alnum_string(24..=24),
                    s_dist_03: rand.alnum_string(24..=24),
                    s_dist_04: rand.alnum_string(24..=24),
                    s_dist_05: rand.alnum_string(24..=24),
                    s_dist_06: rand.alnum_string(24..=24),
                    s_dist_07: rand.alnum_string(24..=24),
                    s_dist_08: rand.alnum_string(24..=24),
                    s_dist_09: rand.alnum_string(24..=24),
                    s_dist_10: rand.alnum_string(24..=24),
                    s_ytd: 0,
                    s_order_cnt: 0,
                    s_remote_cnt: 0,
                    s_data: rand.item_data(),
                }
            })
            .collect::<Vec<Self>>();

        diesel::insert_into(stocks::table)
            .values(&prepared_stocks)
            .execute(conn)?;
        Ok(prepared_stocks)
    }
}

/// Interface type
pub struct StockedItem {
    item: Item,
    stock: Stock,
}

impl StockedItem {
    pub fn find(warehouse_id: i32, item_id: i32, conn: &mut DbConnection) -> QueryResult<Self> {
        use schema::{items, stocks};
        let item = items::table.find(item_id).first::<Item>(conn)?;
        let stock = stocks::table
            .filter(stocks::s_w_id.eq(warehouse_id))
            .filter(stocks::s_i_id.eq(item_id))
            .first::<Stock>(conn)?;

        Ok(Self { item, stock })
    }
}

/// District: belongs to Warehouse
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::districts)]
pub struct District {
    d_id: i32,
    d_w_id: i32,
    d_name: String,
    d_street_1: String,
    d_street_2: String,
    d_city: String,
    d_state: String,
    d_zip: String,
    d_tax: f64,
    d_ytd: f64,
    d_next_o_id: i32,
}

impl District {
    /// Get district by it's id
    ///   public API: call warehouse.find_district() instead.
    fn find(warehouse_id: i32, district_id: i32, conn: &mut DbConnection) -> QueryResult<Self> {
        use schema::districts;
        districts::table
            .filter(districts::d_w_id.eq(warehouse_id))
            .filter(districts::d_id.eq(district_id))
            .first(conn)
    }

    /// Get district by warehouse
    ///   public API: call warehouse.all_districts() instead.
    fn all_by_warehouse(warehouse_id: i32, conn: &mut DbConnection) -> QueryResult<Vec<Self>> {
        use schema::districts;
        districts::table
            .filter(districts::d_w_id.eq(warehouse_id))
            .load(conn)
    }

    /// Get tax rate of the district
    pub fn tax(&self) -> f64 {
        self.d_tax
    }

    /// Find customer
    pub fn find_customer(
        &self,
        customer_id: i32,
        conn: &mut DbConnection,
    ) -> QueryResult<Customer> {
        Customer::find(self.d_w_id, self.d_id, customer_id, conn)
    }

    /// Add new order
    /// TPC-C standard spec. 2.4.2
    pub fn insert_order(
        &mut self,
        customer: &Customer,
        items: &[(StockedItem, i32)], // (item, quantity)
        conn: &mut DbConnection,
    ) -> QueryResult<(Order, Vec<OrderLine>)> {
        use diesel::Connection;

        conn.transaction(|conn| {
            // Run transaction
            let order_id = self.issue_order_id(conn)?;
            let (order, lines) =
                Order::insert(self.d_w_id, self.d_id, order_id, customer, items, conn)?;

            // allocate stock
            for (item, qty) in items {
                item.stock.allocate(*qty, self.d_w_id, conn)?;
            }
            Ok((order, lines))
        })
    }

    /// Issue new order_id
    fn issue_order_id(&mut self, conn: &mut DbConnection) -> QueryResult<i32> {
        use schema::districts;

        // Increment d_next_o_id
        let row = districts::table
            .filter(districts::d_w_id.eq(self.d_w_id))
            .filter(districts::d_id.eq(self.d_id));
        let next_id = diesel::update(row)
            .set(districts::d_next_o_id.eq(districts::d_next_o_id + 1))
            .returning(districts::d_next_o_id)
            .get_result(conn)?;

        self.d_next_o_id = next_id;

        Ok(next_id - 1)
    }

    /// Delivery transaction
    /// TPC-C standard spec. 2.7.4
    pub fn delivery(&self, carrier_id: i32, conn: &mut DbConnection) -> QueryResult<usize> {
        use diesel::Connection;
        conn.transaction(|conn| {
            use schema::{customers, new_orders, orders};

            // Oldest 10 orders
            let order_ids = new_orders::table
                .filter(new_orders::no_w_id.eq(self.d_w_id))
                .filter(new_orders::no_d_id.eq(self.d_id))
                .order(new_orders::no_o_id)
                .select(new_orders::no_o_id)
                .limit(10)
                .load::<i32>(conn)?;
            // Remove new_orders to be delivered
            diesel::delete(
                new_orders::table
                    .filter(new_orders::no_w_id.eq(self.d_w_id))
                    .filter(new_orders::no_d_id.eq(self.d_id))
                    .filter(new_orders::no_o_id.eq_any(&order_ids)),
            )
            .execute(conn)?;

            let orders_to_deliver: Vec<Order> = diesel::update(
                orders::table
                    .filter(orders::o_w_id.eq(self.d_w_id))
                    .filter(orders::o_d_id.eq(self.d_id))
                    .filter(orders::o_id.eq_any(order_ids)),
            )
            .set(orders::o_carrier_id.eq(carrier_id))
            .get_results(conn)?;

            let tm = chrono::Utc::now().naive_utc();
            for order in &orders_to_deliver {
                let lines = order.record_lines_deliver_at(tm, conn)?;
                let total_amount = lines.iter().map(|ol| ol.amount()).sum::<f64>();

                // Update customer balance
                diesel::update(
                    customers::table
                        .filter(customers::c_w_id.eq(self.d_w_id))
                        .filter(customers::c_d_id.eq(self.d_id))
                        .filter(customers::c_id.eq(order.o_c_id)),
                )
                .set((
                    customers::c_balance.eq(customers::c_balance + total_amount),
                    customers::c_delivery_cnt.eq(customers::c_delivery_cnt + 1),
                ))
                .execute(conn)?;
            }
            Ok(orders_to_deliver.len())
        })
    }

    /// Stock-Level transaction
    /// TPC-C standard spec. 2.8.2
    pub fn check_stock_level(
        &self,
        stock_level: i32,
        conn: &mut DbConnection,
    ) -> QueryResult<usize> {
        use schema::{order_lines, stocks};

        let item_ids: Vec<i32> = order_lines::table
            .filter(order_lines::ol_w_id.eq(self.d_w_id))
            .filter(order_lines::ol_d_id.eq(self.d_id))
            .filter(order_lines::ol_o_id.ge(self.d_next_o_id - 20))
            .select(order_lines::ol_i_id)
            .load(conn)?;

        // Count items lower than stock level
        let low_stocks: i64 = stocks::table
            .filter(stocks::s_w_id.eq(self.d_w_id))
            .filter(stocks::s_i_id.eq_any(&item_ids))
            .filter(stocks::s_quantity.lt(stock_level))
            .select(diesel::dsl::count_distinct(stocks::s_i_id))
            .first(conn)?;

        Ok(low_stocks as usize)
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::districts::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    /// Insert new districts
    ///   public API: call warehouse.prepare_district() instead.
    fn prepare(
        warehouse_id: i32,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::districts;
        let cur_id = districts::table
            .filter(districts::d_w_id.eq(warehouse_id))
            .select(diesel::dsl::max(districts::d_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        let prepared_districts = (0..num)
            .map(|i| {
                // TPC-C standard spec. 4.3.3
                Self {
                    d_id: cur_id + i + 1,
                    d_w_id: warehouse_id,
                    d_name: rand.alnum_string(6..=10),
                    d_street_1: rand.alnum_string(10..=20),
                    d_street_2: rand.alnum_string(10..=20),
                    d_city: rand.alnum_string(10..=20),
                    d_state: rand.alnum_string(2..=2),
                    d_zip: rand.zip_code(),
                    d_tax: rand.f64_range(0.0000..=0.2000),
                    d_ytd: 30_000.00,
                    d_next_o_id: 3001,
                }
            })
            .collect::<Vec<Self>>();

        diesel::insert_into(districts::table)
            .values(&prepared_districts)
            .execute(conn)?;
        Ok(prepared_districts)
    }

    /// Insert customers in district
    /// In TPC-C standard, each district has 3_000 customers.
    pub fn prepare_customers(
        &self,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Customer>> {
        Customer::prepare(self.d_w_id, self.d_id, num, rand, conn)
    }

    /// Insert orders in district
    /// In TPC-C standard, each district has 3_000 orders.
    pub fn prepare_orders(
        &self,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Order>> {
        Order::prepare(self.d_w_id, self.d_id, num, rand, conn)
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::customers)]
pub struct Customer {
    c_id: i32,
    c_d_id: i32,
    c_w_id: i32,
    c_first: String,
    c_middle: String,
    c_last: String,
    c_street_1: String,
    c_street_2: String,
    c_city: String,
    c_state: String,
    c_zip: String,
    c_phone: String,
    c_since: chrono::NaiveDateTime,
    c_credit: String,
    c_credit_lim: f64,
    c_discount: f64,
    c_balance: f64,
    c_ytd_payment: f64,
    c_payment_cnt: i32,
    c_delivery_cnt: i32,
    c_data: String,
}

impl Customer {
    /// Get customer by it's id
    pub fn find(
        warehouse_id: i32,
        district_id: i32,
        customer_id: i32,
        conn: &mut DbConnection,
    ) -> QueryResult<Self> {
        use schema::customers;
        customers::table
            .filter(customers::c_w_id.eq(warehouse_id))
            .filter(customers::c_d_id.eq(district_id))
            .filter(customers::c_id.eq(customer_id))
            .first(conn)
    }

    /// Get customer by it's last name
    pub fn find_by_name(
        warehouse_id: i32,
        district_id: i32,
        lastname: &str,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::customers;
        customers::table
            .filter(customers::c_w_id.eq(warehouse_id))
            .filter(customers::c_d_id.eq(district_id))
            .filter(customers::c_last.eq(lastname))
            .order(customers::c_first)
            .load::<Self>(conn)
    }

    /// Payment Transaction
    /// TPC-C standard spec. 2.5
    pub fn pay(
        &self,
        district_at: &District,
        amount: f64,
        conn: &mut DbConnection,
    ) -> QueryResult<(Self, History, District, Warehouse)> {
        use diesel::Connection;
        use schema::{customers, districts, warehouses};

        conn.transaction(move |conn| {
            // Increment warehouse ytd
            let warehouse = diesel::update(warehouses::table.find(district_at.d_w_id))
                .set(warehouses::w_ytd.eq(warehouses::w_ytd + amount))
                .get_result::<Warehouse>(conn)?;
            // Increment district ytd
            let row = districts::table
                .filter(districts::d_w_id.eq(district_at.d_w_id))
                .filter(districts::d_id.eq(district_at.d_id));
            let district = diesel::update(row)
                .set(districts::d_ytd.eq(districts::d_ytd + amount))
                .get_result::<District>(conn)?;

            // Update customer column
            let row = customers::table
                .filter(customers::c_w_id.eq(self.c_w_id))
                .filter(customers::c_d_id.eq(self.c_d_id))
                .filter(customers::c_id.eq(self.c_id));
            let updated_customer = if self.c_credit == "BC" {
                // Update c_data field
                let new_c_data = format!(
                    "{:04}{:04}{:04}{:04}{:04}{:04.2}{}",
                    self.c_id,
                    self.c_d_id,
                    self.c_w_id,
                    district.d_w_id,
                    warehouse.w_id,
                    amount,
                    self.c_data
                );
                let new_c_data_trimed = &new_c_data[0..self.c_data.len()];

                diesel::update(row)
                    .set((
                        customers::c_balance.eq(customers::c_balance - amount),
                        customers::c_ytd_payment.eq(customers::c_ytd_payment + amount),
                        customers::c_data.eq(new_c_data_trimed),
                    ))
                    .get_result::<Self>(conn)?
            } else {
                diesel::update(row)
                    .set((
                        customers::c_balance.eq(customers::c_balance - amount),
                        customers::c_ytd_payment.eq(customers::c_ytd_payment + amount),
                    ))
                    .get_result::<Self>(conn)?
            };

            // Insert history
            let history = History::insert(&updated_customer, &warehouse, &district, amount, conn)?;

            Ok((updated_customer, history, district, warehouse))
        })
    }

    /// Returns last order
    /// Order-Status Transaction
    /// TPC-C standard spec. 2.6
    pub fn last_order(&self, conn: &mut DbConnection) -> QueryResult<(Order, Vec<OrderLine>)> {
        use schema::orders;

        let order = orders::table
            .filter(orders::o_w_id.eq(self.c_w_id))
            .filter(orders::o_d_id.eq(self.c_d_id))
            .filter(orders::o_c_id.eq(self.c_id))
            .order(orders::o_id.desc())
            .first::<Order>(conn)?;

        let lines = order.order_lines(conn)?;

        Ok((order, lines))
    }

    /// PK
    pub fn id(&self) -> (i32, i32, i32) {
        (self.c_w_id, self.c_d_id, self.c_id)
    }

    /// First name
    pub fn firstname<'a>(&'a self) -> &'a str {
        self.c_first.as_str()
    }

    /// Last name
    pub fn lastname<'a>(&'a self) -> &'a str {
        self.c_last.as_str()
    }

    /// Disacount rate for this customer
    pub fn discount_rate(&self) -> f64 {
        self.c_discount
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::customers::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    /// Insert new customers
    ///   public API: call district.prepare_customers() instead.
    fn prepare(
        warehouse_id: i32,
        district_id: i32,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::{customers, histories};

        let cur_c_id = customers::table
            .filter(customers::c_w_id.eq(warehouse_id))
            .filter(customers::c_d_id.eq(district_id))
            .select(diesel::dsl::max(customers::c_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        let prepared_customers = (0..num)
            .map(|i| {
                // TPC-C standard spec. 4.3.3
                let c_credit = if 0 == rand.i32_range(0..=9) {
                    "GC" // 10%
                } else {
                    "BC" // 90%
                }
                .to_string();
                let c_last = if i < 999 {
                    tpcc_rand::TpcRandom::last_name(i + 1) // spec. 4.3.2.3
                } else {
                    tpcc_rand::TpcRandom::last_name(rand.non_uniform_i32(255, 0..=999))
                };
                Self {
                    c_id: cur_c_id + i + 1,
                    c_d_id: district_id,
                    c_w_id: warehouse_id,
                    c_first: rand.alnum_string(8..=16),
                    c_middle: "OE".to_string(),
                    c_last,
                    c_street_1: rand.alnum_string(10..=20),
                    c_street_2: rand.alnum_string(10..=20),
                    c_city: rand.alnum_string(10..=20),
                    c_state: rand.alnum_string(2..=2),
                    c_zip: rand.zip_code(),
                    c_phone: rand.num_string(16),
                    c_since: chrono::Utc::now().naive_utc(),
                    c_credit,
                    c_credit_lim: 50_000.00,
                    c_discount: rand.f64_range(0.0..=0.5),
                    c_balance: -10.00,
                    c_ytd_payment: 10.00,
                    c_payment_cnt: 1,
                    c_delivery_cnt: 0,
                    c_data: rand.alnum_string(300..=500),
                }
            })
            .collect::<Vec<Self>>();
        diesel::insert_into(customers::table)
            .values(&prepared_customers)
            .execute(conn)?;

        // Also insert histories
        let cur_h_id = histories::table
            .select(diesel::dsl::max(histories::h_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);
        let prepared_histories = prepared_customers
            .iter()
            .enumerate()
            .map(|(i, customer)| {
                // TPC-C standard spec. 4.3.3
                History {
                    h_id: cur_h_id + 1 + i as i32,
                    h_c_id: customer.c_id,
                    h_c_d_id: customer.c_d_id,
                    h_c_w_id: customer.c_w_id,
                    h_d_id: customer.c_d_id,
                    h_w_id: customer.c_w_id,
                    h_date: chrono::Utc::now().naive_utc(),
                    h_amount: 10.0,
                    h_data: rand.alnum_string(12..=24),
                }
            })
            .collect::<Vec<History>>();
        diesel::insert_into(histories::table)
            .values(&prepared_histories)
            .execute(conn)?;

        Ok(prepared_customers)
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::histories)]
pub struct History {
    h_id: i32,
    h_c_id: i32,
    h_c_d_id: i32,
    h_c_w_id: i32,
    h_d_id: i32,
    h_w_id: i32,
    h_date: chrono::NaiveDateTime,
    h_amount: f64,
    h_data: String,
}

impl History {
    fn insert(
        customer: &Customer,
        warehouse_at: &Warehouse,
        district_at: &District,
        amount: f64,
        conn: &mut DbConnection,
    ) -> QueryResult<Self> {
        use schema::histories;

        // max history_id
        let cur_h_id = histories::table
            .select(diesel::dsl::max(histories::h_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        let history = Self {
            h_id: cur_h_id + 1,
            h_c_id: customer.c_id,
            h_c_d_id: customer.c_d_id,
            h_c_w_id: customer.c_w_id,
            h_d_id: district_at.d_id,
            h_w_id: warehouse_at.w_id,
            h_date: chrono::Utc::now().naive_utc(),
            h_amount: amount,
            h_data: format!("{}    {}", warehouse_at.w_name, district_at.d_name),
        };

        diesel::insert_into(histories::table)
            .values(&history)
            .execute(conn)?;

        Ok(history)
    }

    /// history timestamp
    pub fn timestamp(&self) -> chrono::NaiveDateTime {
        self.h_date
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::histories::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::orders)]
pub struct Order {
    o_id: i32,
    o_d_id: i32,
    o_w_id: i32,
    o_c_id: i32,
    o_entry_d: chrono::NaiveDateTime,
    o_carrier_id: Option<i32>,
    o_ol_cnt: i32,
    o_all_local: i32,
}

impl Order {
    /// Insert new orders
    ///   public API: call district.insert_order() instead.
    fn insert(
        warehouse_id: i32,
        district_id: i32,
        order_id: i32,
        customer: &Customer,
        items: &[(StockedItem, i32)],
        conn: &mut DbConnection,
    ) -> QueryResult<(Self, Vec<OrderLine>)> {
        use schema::{new_orders, order_lines, orders};

        // Order
        let insert_order = Self {
            o_id: order_id,
            o_d_id: district_id,
            o_w_id: warehouse_id,
            o_c_id: customer.c_id,
            o_entry_d: chrono::Utc::now().naive_utc(),
            o_carrier_id: None,
            o_ol_cnt: items.len() as i32,
            o_all_local: items.iter().all(|(s, _qty)| s.stock.s_w_id == warehouse_id) as i32,
        };
        diesel::insert_into(orders::table)
            .values(&insert_order)
            .execute(conn)?;

        // NewOrder
        let insert_new_order = NewOrder {
            no_o_id: order_id,
            no_d_id: district_id,
            no_w_id: warehouse_id,
        };
        diesel::insert_into(new_orders::table)
            .values(&insert_new_order)
            .execute(conn)?;

        // OrderLines
        let insert_order_lines = items
            .iter()
            .enumerate()
            .map(|(idx, (item, qty))| OrderLine::new(customer, item, order_id, idx as i32, *qty))
            .collect::<Vec<_>>();
        diesel::insert_into(order_lines::table)
            .values(&insert_order_lines)
            .execute(conn)?;

        Ok((insert_order, insert_order_lines))
    }

    /// OrderLines of this Order
    fn order_lines(&self, conn: &mut DbConnection) -> QueryResult<Vec<OrderLine>> {
        use schema::order_lines;

        order_lines::table
            .filter(order_lines::ol_w_id.eq(self.o_w_id))
            .filter(order_lines::ol_d_id.eq(self.o_d_id))
            .filter(order_lines::ol_o_id.eq(self.o_id))
            .order(order_lines::ol_number)
            .load::<OrderLine>(conn)
    }

    /// Record delivery timestamp to OrderLines
    fn record_lines_deliver_at(
        &self,
        tm: chrono::NaiveDateTime,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<OrderLine>> {
        use schema::order_lines;

        let updated_lines = diesel::update(
            order_lines::table
                .filter(order_lines::ol_w_id.eq(self.o_w_id))
                .filter(order_lines::ol_d_id.eq(self.o_d_id))
                .filter(order_lines::ol_o_id.eq(self.o_id)),
        )
        .set(order_lines::ol_delivery_d.eq(tm))
        .get_results::<OrderLine>(conn)?;

        Ok(updated_lines)
    }

    /// PK
    pub fn id(&self) -> (i32, i32, i32) {
        (self.o_w_id, self.o_d_id, self.o_id)
    }

    pub fn entry_at(&self) -> chrono::NaiveDateTime {
        self.o_entry_d
    }

    pub fn carrier_id(&self) -> Option<i32> {
        self.o_carrier_id
    }

    /// Count all rows
    pub fn count(conn: &mut DbConnection) -> QueryResult<i64> {
        schema::orders::table
            .select(diesel::dsl::count_star())
            .first(conn)
    }

    /// Insert new orders
    ///   public API: call district.prepare_orders() instead.
    fn prepare(
        warehouse_id: i32,
        district_id: i32,
        num: i32,
        rand: &mut tpcc_rand::TpcRandom,
        conn: &mut DbConnection,
    ) -> QueryResult<Vec<Self>> {
        use schema::{customers, new_orders, order_lines, orders};
        let cur_id = orders::table
            .filter(orders::o_w_id.eq(warehouse_id))
            .filter(orders::o_d_id.eq(district_id))
            .select(diesel::dsl::max(orders::o_id))
            .first::<Option<i32>>(conn)?
            .unwrap_or(0);

        // min-max customer ID
        let (min_c_id, max_c_id) = customers::table
            .filter(customers::c_w_id.eq(warehouse_id))
            .filter(customers::c_d_id.eq(district_id))
            .select((
                diesel::dsl::min(customers::c_id),
                diesel::dsl::max(customers::c_id),
            ))
            .first::<(Option<i32>, Option<i32>)>(conn)?;

        let min_c_id = min_c_id.unwrap_or(0);
        let max_c_id = max_c_id.unwrap_or(0);

        // Orders
        let prepared_orders = (0..num)
            .map(|i| {
                // TPC-C standard spec. 4.3.3
                let o_id = cur_id + i + 1;
                let o_carrier_id = if o_id <= 2100 {
                    Some(rand.i32_range(1..=10))
                } else {
                    None
                };

                Self {
                    o_id,
                    o_d_id: district_id,
                    o_w_id: warehouse_id,
                    o_c_id: rand.i32_range(min_c_id..=max_c_id),
                    o_entry_d: chrono::Utc::now().naive_utc(),
                    o_carrier_id,
                    o_ol_cnt: rand.i32_range(5..=15),
                    o_all_local: 1,
                }
            })
            .collect::<Vec<Self>>();
        diesel::insert_into(orders::table)
            .values(&prepared_orders)
            .execute(conn)?;

        // OrderLines
        let prepared_orderlines = prepared_orders
            .iter()
            .map(|order| {
                let ol_delivery_id = if order.o_id <= 2100 {
                    Some(order.o_entry_d)
                } else {
                    None
                };
                let ol_amount = if order.o_id <= 2100 {
                    0.0
                } else {
                    rand.f64_range(0.01..=9_999.99)
                };
                (0..order.o_ol_cnt)
                    .map(|i| OrderLine {
                        ol_o_id: order.o_id,
                        ol_d_id: order.o_d_id,
                        ol_w_id: order.o_w_id,
                        ol_number: i,
                        ol_i_id: rand.i32_range(1..=100_000),
                        ol_supply_w_id: order.o_w_id,
                        ol_delivery_d: ol_delivery_id,
                        ol_quantity: 5,
                        ol_amount,
                        ol_dist_info: rand.alnum_string(24..=24),
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<OrderLine>>();
        diesel::insert_into(order_lines::table)
            .values(&prepared_orderlines)
            .execute(conn)?;

        // NewOrders
        let prepared_new_orders = prepared_orders
            .iter()
            .filter_map(|order| {
                if order.o_carrier_id.is_none() {
                    Some(NewOrder {
                        no_o_id: order.o_id,
                        no_d_id: order.o_d_id,
                        no_w_id: order.o_w_id,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<NewOrder>>();
        diesel::insert_into(new_orders::table)
            .values(&prepared_new_orders)
            .execute(conn)?;

        Ok(prepared_orders)
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::order_lines)]
pub struct OrderLine {
    ol_o_id: i32,
    ol_d_id: i32,
    ol_w_id: i32,
    ol_number: i32,
    ol_i_id: i32,
    ol_supply_w_id: i32,
    ol_delivery_d: Option<chrono::NaiveDateTime>,
    ol_quantity: i32,
    ol_amount: f64,
    ol_dist_info: String,
}

impl OrderLine {
    /// NewOrder transaction
    ///
    fn new(
        customer: &Customer,
        item: &StockedItem,
        order_id: i32,
        index: i32,
        quantity: i32,
    ) -> Self {
        let dist_info = match customer.c_d_id {
            1 => item.stock.s_dist_01.as_str(),
            2 => item.stock.s_dist_02.as_str(),
            3 => item.stock.s_dist_03.as_str(),
            4 => item.stock.s_dist_04.as_str(),
            5 => item.stock.s_dist_05.as_str(),
            6 => item.stock.s_dist_06.as_str(),
            7 => item.stock.s_dist_07.as_str(),
            8 => item.stock.s_dist_08.as_str(),
            9 => item.stock.s_dist_09.as_str(),
            10 => item.stock.s_dist_10.as_str(),
            _ => "Invalid District ID",
        };

        Self {
            ol_o_id: order_id,
            ol_d_id: customer.c_d_id,
            ol_w_id: customer.c_w_id,
            ol_number: index,
            ol_i_id: item.item.i_id,
            ol_supply_w_id: item.stock.s_w_id,
            ol_delivery_d: None,
            ol_quantity: quantity,
            ol_amount: quantity as f64 * item.item.i_price,
            ol_dist_info: dist_info.to_string(),
        }
    }

    /// Item ID
    pub fn item_id(&self) -> i32 {
        self.ol_i_id
    }

    /// Supply warehouse
    pub fn supply_warehouse_id(&self) -> i32 {
        self.ol_supply_w_id
    }

    pub fn quantity(&self) -> i32 {
        self.ol_quantity
    }

    pub fn amount(&self) -> f64 {
        self.ol_amount
    }

    pub fn delivery_at(&self) -> Option<chrono::NaiveDateTime> {
        self.ol_delivery_d
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::new_orders)]
struct NewOrder {
    no_o_id: i32,
    no_d_id: i32,
    no_w_id: i32,
}
