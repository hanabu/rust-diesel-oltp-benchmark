mod schema;
mod tpcc_models;

fn main() {
    let mut conn = tpcc_models::connect("tpc_c.sqlite").unwrap();
    tpcc_models::prepare(1, &mut conn).unwrap();
}
