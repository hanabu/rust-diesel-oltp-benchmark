fn main() {
    let mut conn = tpcc_models::connect("tpc_c.sqlite").unwrap();
    tpcc_models::prepare_schema(&mut conn).unwrap();
    tpcc_models::cleanup(&mut conn).unwrap();
    tpcc_models::prepare_data(1, &mut conn).unwrap();
    tpcc_models::vacuum(&mut conn).unwrap();
}
