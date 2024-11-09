// @generated automatically by Diesel CLI.

diesel::table! {
    customers (c_id, c_d_id, c_w_id) {
        c_id -> Integer,
        c_d_id -> Integer,
        c_w_id -> Integer,
        c_first -> Text,
        c_middle -> Text,
        c_last -> Text,
        c_street_1 -> Text,
        c_street_2 -> Text,
        c_city -> Text,
        c_state -> Text,
        c_zip -> Text,
        c_phone -> Text,
        c_since -> Timestamp,
        c_credit -> Text,
        c_credit_lim -> Double,
        c_discount -> Double,
        c_balance -> Double,
        c_ytd_payment -> Double,
        c_payment_cnt -> Integer,
        c_delivery_cnt -> Integer,
        c_data -> Text,
    }
}

diesel::table! {
    districts (d_id, d_w_id) {
        d_id -> Integer,
        d_w_id -> Integer,
        d_name -> Text,
        d_street_1 -> Text,
        d_street_2 -> Text,
        d_city -> Text,
        d_state -> Text,
        d_zip -> Text,
        d_tax -> Double,
        d_ytd -> Double,
        d_next_o_id -> Integer,
    }
}

diesel::table! {
    histories (h_id) {
        h_id -> Integer,
        h_c_id -> Integer,
        h_c_d_id -> Integer,
        h_c_w_id -> Integer,
        h_d_id -> Integer,
        h_w_id -> Integer,
        h_date -> Timestamp,
        h_amount -> Double,
        h_data -> Text,
    }
}

diesel::table! {
    items (i_id) {
        i_id -> Integer,
        i_im_id -> Integer,
        i_name -> Text,
        i_price -> Double,
        i_data -> Text,
    }
}

diesel::table! {
    new_orders (no_o_id, no_d_id, no_w_id) {
        no_o_id -> Integer,
        no_d_id -> Integer,
        no_w_id -> Integer,
    }
}

diesel::table! {
    order_lines (ol_o_id, ol_d_id, ol_w_id, ol_number) {
        ol_o_id -> Integer,
        ol_d_id -> Integer,
        ol_w_id -> Integer,
        ol_number -> Integer,
        ol_i_id -> Integer,
        ol_supply_w_id -> Integer,
        ol_delivery_d -> Nullable<Timestamp>,
        ol_quantity -> Integer,
        ol_amount -> Double,
        ol_dist_info -> Text,
    }
}

diesel::table! {
    orders (o_id, o_d_id, o_w_id) {
        o_id -> Integer,
        o_d_id -> Integer,
        o_w_id -> Integer,
        o_c_id -> Integer,
        o_entry_d -> Timestamp,
        o_carrier_id -> Nullable<Integer>,
        o_ol_cnt -> Integer,
        o_all_local -> Integer,
    }
}

diesel::table! {
    stocks (s_i_id, s_w_id) {
        s_i_id -> Integer,
        s_w_id -> Integer,
        s_quantity -> Integer,
        s_dist_01 -> Text,
        s_dist_02 -> Text,
        s_dist_03 -> Text,
        s_dist_04 -> Text,
        s_dist_05 -> Text,
        s_dist_06 -> Text,
        s_dist_07 -> Text,
        s_dist_08 -> Text,
        s_dist_09 -> Text,
        s_dist_10 -> Text,
        s_ytd -> Integer,
        s_order_cnt -> Integer,
        s_remote_cnt -> Integer,
        s_data -> Text,
    }
}

diesel::table! {
    warehouses (w_id) {
        w_id -> Integer,
        w_name -> Text,
        w_street_1 -> Text,
        w_street_2 -> Text,
        w_city -> Text,
        w_state -> Text,
        w_zip -> Text,
        w_tax -> Double,
        w_ytd -> Double,
    }
}

diesel::joinable!(districts -> warehouses (d_w_id));
diesel::joinable!(stocks -> items (s_i_id));
diesel::joinable!(stocks -> warehouses (s_w_id));

diesel::allow_tables_to_appear_in_same_query!(
    customers,
    districts,
    histories,
    items,
    new_orders,
    order_lines,
    orders,
    stocks,
    warehouses,
);
