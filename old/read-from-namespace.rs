extern crate time;
extern crate postgres;
use postgres::{PostgresConnection, NoSsl};
use postgres::types::array::{ArrayBase};
use time::Timespec;

struct SensorPacket {
  id: i32,
  value: f64,
  name: String,
  time_created: time::Timespec
}

fn main() {
  let conn = PostgresConnection::connect("postgres://root:alphabeta@127.0.0.1:5432/local",&NoSsl).unwrap();

  let query = "SELECT * FROM tmp_table";
  let stmt = conn.prepare(query).unwrap();
  for row in stmt.query([]).unwrap() {
    let p: String = row.get("parent");
    let c: ArrayBase<Option<String>> = row.get("children");
    let mut children: String = "".to_string();
    for s in c.iter() {
      children += try!(s);
    }
    println!("Found sensor {}, {}", p, c);
  }
}