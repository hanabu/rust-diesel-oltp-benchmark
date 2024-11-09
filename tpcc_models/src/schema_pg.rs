// @generated automatically by Diesel CLI.

diesel::table! {
    customers (c_w_id, c_d_id, c_id) {
        c_id -> Int4,
        c_d_id -> Int4,
        c_w_id -> Int4,
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
        c_credit_lim -> Float8,
        c_discount -> Float8,
        c_balance -> Float8,
        c_ytd_payment -> Float8,
        c_payment_cnt -> Int4,
        c_delivery_cnt -> Int4,
        c_data -> Text,
    }
}

diesel::table! {
    districts (d_w_id, d_id) {
        d_id -> Int4,
        d_w_id -> Int4,
        d_name -> Text,
        d_street_1 -> Text,
        d_street_2 -> Text,
        d_city -> Text,
        d_state -> Text,
        d_zip -> Text,
        d_tax -> Float8,
        d_ytd -> Float8,
        d_next_o_id -> Int4,
    }
}

diesel::table! {
    histories (h_id) {
        h_id -> Int4,
        h_c_id -> Int4,
        h_c_d_id -> Int4,
        h_c_w_id -> Int4,
        h_d_id -> Int4,
        h_w_id -> Int4,
        h_date -> Timestamp,
        h_amount -> Float8,
        h_data -> Text,
    }
}

diesel::table! {
    items (i_id) {
        i_id -> Int4,
        i_im_id -> Int4,
        i_name -> Text,
        i_price -> Float8,
        i_data -> Text,
    }
}

diesel::table! {
    new_orders (no_w_id, no_d_id, no_o_id) {
        no_o_id -> Int4,
        no_d_id -> Int4,
        no_w_id -> Int4,
    }
}

diesel::table! {
    order_lines (ol_w_id, ol_d_id, ol_o_id, ol_number) {
        ol_o_id -> Int4,
        ol_d_id -> Int4,
        ol_w_id -> Int4,
        ol_number -> Int4,
        ol_i_id -> Int4,
        ol_supply_w_id -> Int4,
        ol_delivery_d -> Nullable<Timestamp>,
        ol_quantity -> Int4,
        ol_amount -> Float8,
        ol_dist_info -> Text,
    }
}

diesel::table! {
    orders (o_w_id, o_d_id, o_id) {
        o_id -> Int4,
        o_d_id -> Int4,
        o_w_id -> Int4,
        o_c_id -> Int4,
        o_entry_d -> Timestamp,
        o_carrier_id -> Nullable<Int4>,
        o_ol_cnt -> Int4,
        o_all_local -> Int4,
    }
}

diesel::table! {
    stocks (s_w_id, s_i_id) {
        s_i_id -> Int4,
        s_w_id -> Int4,
        s_quantity -> Int4,
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
        s_ytd -> Int4,
        s_order_cnt -> Int4,
        s_remote_cnt -> Int4,
        s_data -> Text,
    }
}

diesel::table! {
    warehouses (w_id) {
        w_id -> Int4,
        w_name -> Text,
        w_street_1 -> Text,
        w_street_2 -> Text,
        w_city -> Text,
        w_state -> Text,
        w_zip -> Text,
        w_tax -> Float8,
        w_ytd -> Float8,
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
