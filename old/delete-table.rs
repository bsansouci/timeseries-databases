extern crate postgres;

use std::os;

use postgres::{PostgresConnection, NoSsl};

fn main() {
  let args: Vec<String> = os::args();

  let dbname = args[1].clone();

  let conn = PostgresConnection::connect("postgres://root:alphabeta@127.0.0.1:5432/local",&NoSsl).unwrap();

  let query = "DROP TABLE ".to_string() + dbname;
  let stmt = conn.prepare(query.as_slice()).unwrap();
  stmt.query([]).unwrap();
}