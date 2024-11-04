-- TPC Benchmark C, rev. 5.11

CREATE TABLE warehouses (
  w_id       INTEGER          NOT NULL,
  w_name     TEXT             NOT NULL,
  w_street_1 TEXT             NOT NULL,
  w_street_2 TEXT             NOT NULL,
  w_city     TEXT             NOT NULL,
  w_state    TEXT             NOT NULL,
  w_zip      TEXT             NOT NULL,
  w_tax      DOUBLE PRECISION NOT NULL,
  w_ytd      DOUBLE PRECISION NOT NULL,
  PRIMARY KEY (w_id)
);

CREATE TABLE districts (
  d_id        INTEGER          NOT NULL,
  d_w_id      INTEGER          NOT NULL,
  d_name      TEXT             NOT NULL,
  d_street_1  TEXT             NOT NULL,
  d_street_2  TEXT             NOT NULL,
  d_city      TEXT             NOT NULL,
  d_state     TEXT             NOT NULL,
  d_zip       TEXT             NOT NULL,
  d_tax       DOUBLE PRECISION NOT NULL,
  d_ytd       DOUBLE PRECISION NOT NULL,
  d_next_o_id INTEGER          NOT NULL,
  PRIMARY KEY (d_w_id, d_id),
  FOREIGN KEY (d_w_id) REFERENCES warehouses (w_id)
);

CREATE TABLE customers (
  c_id           INTEGER          NOT NULL,
  c_d_id         INTEGER          NOT NULL,
  c_w_id         INTEGER          NOT NULL,
  c_first        TEXT             NOT NULL,
  c_middle       TEXT             NOT NULL,
  c_last         TEXT             NOT NULL,
  c_street_1     TEXT             NOT NULL,
  c_street_2     TEXT             NOT NULL,
  c_city         TEXT             NOT NULL,
  c_state        TEXT             NOT NULL,
  c_zip          TEXT             NOT NULL,
  c_phone        TEXT             NOT NULL,
  c_since        DATETIME         NOT NULL,
  c_credit       TEXT             NOT NULL,
  c_credit_lim   DOUBLE PRECISION NOT NULL,
  c_discount     DOUBLE PRECISION NOT NULL,
  c_balance      DOUBLE PRECISION NOT NULL,
  c_ytd_payment  DOUBLE PRECISION NOT NULL,
  c_payment_cnt  INTEGER          NOT NULL,
  c_delivery_cnt INTEGER          NOT NULL,
  c_data         TEXT             NOT NULL,
  PRIMARY KEY (c_w_id, c_d_id, c_id),
  FOREIGN KEY (c_w_id,c_d_id) REFERENCES districts (d_w_id,d_id)
);

CREATE TABLE histories (
  h_id     INTEGER          NOT NULL, -- h_id is not required in TPC-C
  h_c_id   INTEGER          NOT NULL,
  h_c_d_id INTEGER          NOT NULL,
  h_c_w_id INTEGER          NOT NULL,
  h_d_id   INTEGER          NOT NULL,
  h_w_id   INTEGER          NOT NULL,
  h_date   DATETIME         NOT NULL,
  h_amount DOUBLE PRECISION NOT NULL,
  h_data   TEXT             NOT NULL,
  PRIMARY KEY (h_id),
  FOREIGN KEY (h_c_w_id, h_c_d_id, h_c_id) REFERENCES customers (c_w_id, c_d_id, c_id),
  FOREIGN KEY (h_w_id, h_d_id)             REFERENCES districts (d_w_id, d_id)
);

CREATE TABLE items (
  i_id     INTEGER          NOT NULL,
  i_im_id  INTEGER          NOT NULL,
  i_name   TEXT             NOT NULL,
  i_price  DOUBLE PRECISION NOT NULL,
  i_data   TEXT             NOT NULL,
  PRIMARY KEY(i_id)
);

CREATE TABLE stocks (
  s_i_id        INTEGER  NOT NULL,
  s_w_id        INTEGER  NOT NULL,
  s_quantity    INTEGER  NOT NULL,
  s_dist_01     TEXT     NOT NULL,
  s_dist_02     TEXT     NOT NULL,
  s_dist_03     TEXT     NOT NULL,
  s_dist_04     TEXT     NOT NULL,
  s_dist_05     TEXT     NOT NULL,
  s_dist_06     TEXT     NOT NULL,
  s_dist_07     TEXT     NOT NULL,
  s_dist_08     TEXT     NOT NULL,
  s_dist_09     TEXT     NOT NULL,
  s_dist_10     TEXT     NOT NULL,
  s_ytd         INTEGER  NOT NULL,
  s_order_cnt   INTEGER  NOT NULL,
  s_remote_cnt  INTEGER  NOT NULL,
  s_data        TEXT     NOT NULL,
  PRIMARY KEY (s_w_id, s_i_id),
  FOREIGN KEY (s_w_id) REFERENCES warehouses (w_id),
  FOREIGN KEY (s_i_id) REFERENCES items (i_id)
);

CREATE TABLE orders (
  o_id         INTEGER  NOT NULL,
  o_d_id       INTEGER  NOT NULL,
  o_w_id       INTEGER  NOT NULL,
  o_c_id       INTEGER  NOT NULL,
  o_entry_d    DATETIME NOT NULL,
  o_carrier_id INTEGER,
  o_ol_cnt     INTEGER  NOT NULL,
  o_all_local  INTEGER  NOT NULL,
  PRIMARY KEY (o_w_id, o_d_id, o_id),
  FOREIGN KEY (o_w_id, o_d_id, o_c_id) REFERENCES customers (c_w_id, c_d_id, c_id)
);

CREATE TABLE new_orders (
  no_o_id  INTEGER  NOT NULL,
  no_d_id  INTEGER  NOT NULL,
  no_w_id  INTEGER  NOT NULL,
  PRIMARY KEY (no_w_id, no_d_id, no_o_id),
  FOREIGN KEY (no_w_id, no_d_id, no_o_id)  REFERENCES orders(o_w_id, o_d_id, o_id)
);

CREATE TABLE order_lines (
  ol_o_id        INTEGER          NOT NULL,
  ol_d_id        INTEGER          NOT NULL,
  ol_w_id        INTEGER          NOT NULL,
  ol_number      INTEGER          NOT NULL,
  ol_i_id        INTEGER          NOT NULL,
  ol_supply_w_id INTEGER          NOT NULL,
  ol_delivery_d  DATETIME,
  ol_quantity    INTEGER          NOT NULL,
  ol_amount      DOUBLE PRECISION NOT NULL,
  ol_dist_info   TEXT             NOT NULL,
  PRIMARY KEY (ol_w_id, ol_d_id, ol_o_id, ol_number),
  FOREIGN KEY (ol_w_id, ol_d_id, ol_o_id) REFERENCES orders(o_w_id, o_d_id, o_id),
  FOREIGN KEY (ol_supply_w_id, ol_i_id) REFERENCES stocks(s_w_id, s_i_id)
);

