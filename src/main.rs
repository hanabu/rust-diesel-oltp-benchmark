mod schema;

fn main() {
    let mut conn = tpcc_models::connect("tpc_c.sqlite").unwrap();
    tpcc_models::prepare(2, &mut conn).unwrap();
}
