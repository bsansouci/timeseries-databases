extern crate time;
extern crate postgres;
use postgres::{PostgresConnection, NoSsl};
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
    let packet = SensorPacket {
        id: row.get("id"),
        name: row.get("name"),
        time_created: row.get("time_created"),
        value: row.get("value")
    };
    println!("Found sensor {}, {}, {}, {}", packet.id, packet.name, packet.time_created, packet.value);
  }
}