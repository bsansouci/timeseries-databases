extern crate sync;
extern crate postgres;
extern crate time;

use std::str::from_utf8;
use std::io::{Acceptor, Listener};
use std::io::{TcpListener, TcpStream};

use time::Timespec;

use postgres::{PostgresConnection, NoSsl};

struct SensorPacket {
  id: i32,
  value: f64,
  name: String,
  time_created: time::Timespec
}

fn handle_sensor(mut stream: TcpStream, db: Sender<SensorPacket>) {
  // let sensor_name: String = match stream.read_line() {
  //   Ok(s) => s,
  //   Err(e) => {
  //     println!("Error reading name from stream {}", e);
  //     return;
  //   },
  // };
  let mut tmp = [0u8, ..1024];
  stream.read(tmp);
  let sensor_name = from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_string();

  // let firstPacket: Vec<&str> = req.split_str(":").collect();
  // let sensor_name = String::from_str(firstPacket[0]);
  println!("New sensor with name {}", sensor_name)
  // let mut buffer = [0u8, ..1024];
  let mut id: i32 = 0;
  loop {
    let req = stream.read_be_f64();
    // let req = str::from_utf8(buffer).expect("unable to parse buffer to a utf8 string").to_owned();
    match req {
      Ok(x) => {
        println!("HANDLE_SENSOR got {}", x);

        db.send(SensorPacket {
          value: x,
          name: sensor_name.clone(),
          time_created: time::get_time(),
          id: id});
        id += 1;
      },
      Err(e) => {
        println!("Error reading sensor data {}", e);
        break;
      },
    }
    // let p: f64 = match from_str::from_str::<f64>(req.as_slice()) {
    //   Some(x) => x,
    //   None    => 0.0
    // };
  }
  drop(stream);
}

// fn handle_hook(mut stream: TcpStream, receiver Receiver<(int, f64)>, sender Sender<(id, &str)>) {
//   let mut tmp = [0u8, ..1024];

//   stream.read(tmp);

//   let req = str::from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_owned();
//   let v: Vec<&str> = req.as_slice().split_str(":").collect();
//   // let sensor_name = String::from_str(v[0]);
//   sender.send((TypeId::of::<int>(), v[0]));
//   let mut buffer = [0u8, ..1024];
//   while(true) {
//     // let s: Packet = mainChannel.recv();
//     // if s.sensor == sensor_name {
//     //   let v: &[u8] =  match s.value {
//     //     Some(x) => format!("{:s}", x.to_string()).as_bytes(),
//     //     None    => format!("{}", s.value).as_bytes(),
//     //   };
//     //   stream.write(v)
//     // }
//     let (id, data) = receiver.recv();
//     stream.write(format!("{:s}", data);
//   }

//   drop(stream)
// }

fn main() {
  // let mutex1 = Arc::new(RWLock::new(Vec::new()));
  // let mutex2 = mutex1.clone();
  // spawn(proc() {
  //   let hook_listener = TcpListener::bind("127.0.0.1", 8079);
  //   let mut hook = hook_listener.listen();
  //   // accept connections and process them, spawning a new tasks for each one
  //   for stream in hook.incoming() {
  //     println!("hey")
  //     match stream {
  //       Err(e) => { /* connection failed */ }
  //       Ok(stream) => {
  //         let mutex3 = mutex2.clone();
  //         spawn(proc() {
  //           let mut tmp = [0u8, ..1024];

  //           stream.read(tmp);

  //           let req = str::from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_owned();
  //           let v: Vec<&str> = req.as_slice().split_str(":").collect();

  //           let mut val = mutex3.write();
  //           // let sensor_name = String::from_str(v[0]);
  //           val.push(SensorPacket{stream: stream, sensor_name: String::from_str(v[0])});
  //           let val = val.downgrade();
  //           // let mut buffer = [0u8, ..1024];
  //           // while(true) {
  //           //   // let s: Packet = mainChannel.recv();
  //           //   // if s.sensor == sensor_name {
  //           //   //   let v: &[u8] =  match s.value {
  //           //   //     Some(x) => format!("{:s}", x.to_string()).as_bytes(),
  //           //   //     None    => format!("{}", s.value).as_bytes(),
  //           //   //   };
  //           //   //   stream.write(v)
  //           //   // }
  //           //   let (id, data) = receiver.recv();
  //           //   stream.write(format!("{:s}", data);
  //           // }

  //           drop(stream)
  //           // handle_hook(stream, sensorReceiver, hookSender.clone())
  //         })
  //       }
  //     }
  //   }

  //   drop(hook);
  // });

  // let value = mutex1.read();
  // println!("Length {}", value.len());
  // let (sensorSender, sensorReceiver): (Sender<(int, f64)>, Receiver<(int, f64)>) = channel();

  // spawn(proc() {
  //   let hook_listener = TcpListener::bind("127.0.0.1", 8079);
  //   let mut hook = hook_listener.listen();
  //   // accept connections and process them, spawning a new tasks for each one
  //   for stream in hook.incoming() {
  //     match stream {
  //       Err(e) => { /* connection failed */ }
  //       Ok(stream) => spawn(proc() {
  //         handle_hook(stream, sensorReceiver, hookSender.clone())
  //       })
  //     }
  //   }

  //   drop(hook);
  // });

  // let (sender, receiver): (Sender<SensorPacket>, Receiver<SensorPacket>) = channel();

  // let conn = PostgresConnection::connect("postgres://root:alphabeta@127.0.0.1:5432/local",&NoSsl).unwrap();
  // let stmt = conn.prepare("SELECT id, name, time_created, value FROM sensors")
  //         .unwrap();
  // for row in stmt.query([]).unwrap() {
  //     let data = SensorPacket {
  //         id: row.get(0),
  //         name: row.get(1),
  //         time_created: row.get(2),
  //         value: row.get(3)
  //     };
  //     println!("Found {}, {}, {}, {}", data.id, data.name, data.time_created, data.value);
  // }



  let (sender, receiver): (Sender<SensorPacket>, Receiver<SensorPacket>) = channel();

  spawn(proc(){
    let conn = PostgresConnection::connect("postgres://root:alphabeta@127.0.0.1:5432/local",&NoSsl).unwrap();
    // conn.execute("CREATE TABLE sensors (
    //                 id              SERIAL PRIMARY KEY,
    //                 name            VARCHAR NOT NULL,
    //                 time_created    TIMESTAMP NOT NULL,
    //                 value           DOUBLE PRECISION
    //               )", []).unwrap();
    loop {
      let data = receiver.recv();
      println!("Receiver got {} from {} at {}", data.value, data.name, data.time_created);
      conn.execute("INSERT INTO sensors (name, time_created, value)
                      VALUES ($1, $2, $3)",
                   &[&data.name, &data.time_created, &data.value]).unwrap();
    }
  });

  let sensor_listener = TcpListener::bind("127.0.0.1", 8000);
  let mut sensors = sensor_listener.listen();
  // accept connections and process them, spawning a new tasks for each one
  for stream in sensors.incoming() {
    match stream {
      Err(e) => {
        println!("Error connecting to a sensor {}", e);
        break;
      }
      Ok(stream) => {
        println!("New connection");
        let sender_copy = sender.clone();
        spawn(proc() {
          handle_sensor(stream, sender_copy)
        })
      }
    }
  }

  drop(sensors);
}