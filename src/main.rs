extern crate time;
extern crate postgres;

use std::str;
use std::io::{Acceptor, Listener};
use std::io::{TcpListener, TcpStream};

use std::collections::HashMap;
use std::collections::RingBuf;
use std::sync::{RWLock, Arc};

use time::Timespec;

use postgres::{Connection, SslMode};
use postgres::types::array::{ArrayBase};

struct SensorPacket {
  value: f64,
  name: String,
  time_created: time::Timespec
}

struct NamespacePacket {
  parent: String,
  children: String,
  is_sensor: bool
}

fn read_str(stream: &mut TcpStream) -> String {
  let length = stream.read_be_u32().unwrap();

  let mut t: Vec<u8> = Vec::from_elem(length.to_uint().unwrap(), 0);
  let tmp: &mut [u8] = t.as_mut_slice();
  stream.read(tmp);
  return str::from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_string();
}

fn create_namespace(ns: Sender<NamespacePacket>, namespace: &Vec<&str>) {
  let mut not_first = false;
  let mut prev: &str = "ERROR";
  for n in namespace.iter() {
    if not_first {
      ns.send(NamespacePacket {
        parent: prev.to_string(),
        children: n.to_string(),
        is_sensor: false
      });
    }
    prev = *n;
    not_first = true;
  }

  ns.send(NamespacePacket {
    parent: "".to_string(),
    children: namespace[namespace.len() - 1].to_string(),
    is_sensor: true
  });
}

// fn send_data_to_db(db: &Sender<SensorPacket>, value: f64, name: &String) {
//   db.send(SensorPacket {
//     value: value,
//     name: name.clone(),
//     time_created: time::get_time()
//   });
// }


// fn handle_hook(mut stream: TcpStream, sensor: Queue<SensorPacket>) {
//   let req = read_str(&mut stream);

//   let v: Vec<&str> = req.as_slice().split_str(":").collect();
//   let split: Vec<&str> = v[0].split_str(".").collect();
//   let hook_name = split[split.len() - 1];
//   println!("New hook with name {}", hook_name);
//   loop {
//     let data: Option<SensorPacket> = sensor.pop();
//     match data {
//       Some(x) => {
//         println!("Hook-> {}: {}", x.name, x.value)
//         stream.write(format!("{}:{}", x.name, x.value).as_bytes());
//       },
//       None => continue
//     }
//   }

//   // while(true) {
//     // let s: Packet = mainChannel.recv();
//     // if s.sensor == sensorName {
//     //   let v: &[u8] =  match s.value {
//     //     Some(x) => format!("{:s}", x.to_string()).as_bytes(),
//     //     None    => format!("{}", s.value).as_bytes(),
//     //   };
//     //   stream.write(v)
//     // }
//     // stream.write(format!("{:s}", mainChannel.recv().to_string()).as_bytes());
//   // }

//   drop(stream)
// }

fn add_sensor(all_hooks: &Arc<RWLock<HashMap<String, RingBuf<TcpStream>>>>, name: String) {
  let mut w = all_hooks.write();
  w.insert(name, RingBuf::new());
}

fn send_to_all_hooks(all_hooks: &Arc<RWLock<HashMap<String, RingBuf<TcpStream>>>>, name: String, message: String) {
  let mut w = all_hooks.write();
  let m = message.as_bytes();
  match w.get_mut(&name) {
    Some(list) => {
      for e in list.iter_mut() {
        e.write(m);
      }
    }
    None => println!("Problem here.")
  }
}

fn handle_sensor(mut stream: TcpStream, db: Sender<SensorPacket>, ns: Sender<NamespacePacket>, all_hooks: Arc<RWLock<HashMap<String, RingBuf<TcpStream>>>>) {
  let namespace: String = read_str(&mut stream);
  let str_split: Vec<&str> = namespace.as_slice().split_str(".").collect();

  // create_namespace(ns, &str_split);
  println!("WARNING: please comment out line 95 of main.rs now that you've created the namespace for that specific sensor. If you don't, you'll get an error next time because it will try to insert a new row with the same key as a previous one.");




  let sensor_name: String = str_split[str_split.len() - 1].to_string();
  let tmp = all_hooks.clone();
  add_sensor(&all_hooks, sensor_name.clone());

  println!("New sensor with name {}", sensor_name);

  loop {
    let req = stream.read_be_f64();
    // let req = str::from_utf8(buffer).expect("unable to parse buffer to a utf8 string").to_owned();
    match req {
      Ok(x) => {
        println!("HANDLE_SENSOR got {}", x);
        // db.send(SensorPacket {
        //   value: x,
        //   name: sensor_name.clone(),
        //   time_created: time::get_time()
        // });
        send_to_all_hooks(&all_hooks, sensor_name.clone(), format!("{}:{}", sensor_name, x));
      },
      Err(e) => {
        println!("Error reading sensor data {}", e);
        send_to_all_hooks(&all_hooks, sensor_name.clone(), format!("{}:closed", sensor_name));
        break;
      },
    }
  }
  drop(stream);
}

fn handle_hook(mut stream: TcpStream, all_hooks: &Arc<RWLock<HashMap<String, RingBuf<TcpStream>>>>) {
  let req = read_str(&mut stream);

  let v: Vec<&str> = req.as_slice().split_str(":").collect();
  let request = v[1];
  let namespaces: Vec<&str> = v[0].split_str(".").collect();
  let wanted_sensor = namespaces[namespaces.len() - 1].to_string();

  let mut unlocked_map = all_hooks.write();

  match unlocked_map.get_mut(&wanted_sensor) {
    Some(list) => {
      println!("New hook for sensor `{}`", wanted_sensor);
      list.push_back(stream.clone());
    }
    None => println!("Sorry, the sensor `{}` isn't available.", wanted_sensor)
  }
}

fn main() {
  let (sql_sensor_sender, sql_sensor_receiver): (Sender<SensorPacket>, Receiver<SensorPacket>) = channel();

  spawn(proc(){
    let conn = Connection::connect("postgres://root:password@127.0.0.1:5432/local",&SslMode::None).unwrap();
    // conn.execute("CREATE TABLE tmp_table (
    //                 id              SERIAL PRIMARY KEY,
    //                 name            VARCHAR NOT NULL,
    //                 time_created    TIMESTAMP NOT NULL,
    //                 value           DOUBLE PRECISION
    //               )", []).unwrap();
    loop {
      let data = sql_sensor_receiver.recv();
      println!("Inserting into tmp_table: {} from {} at {}", data.value, data.name, data.time_created);
      conn.execute("INSERT INTO tmp_table (name, time_created, value) VALUES ($1, $2, $3)", &[&data.name, &data.time_created, &data.value]).unwrap();
    }
  });

  let (namespace_sender, namespace_receiver): (Sender<NamespacePacket>, Receiver<NamespacePacket>) = channel();

  spawn(proc(){
    let conn = Connection::connect("postgres://root:password@127.0.0.1:5432/local",&SslMode::None).unwrap();
    // conn.execute("CREATE TABLE active_sensors (
    //                 sensor_name      VARCHAR PRIMARY KEY,
    //                 is_active        BOOLEAN
    //               )", []).unwrap();
    loop {
      let data = namespace_receiver.recv();

      // We append this new sensor to the table of sensors (if the parent
      // doens't exist, we create it, if it does we add to its list of
      // children)
      if data.is_sensor {
        println!("Setting sensor {} available", data.children);
        conn.execute("UPDATE active_sensors SET is_active=true WHERE sensor_name=$1", &[&data.children]).unwrap();
        conn.execute("INSERT INTO active_sensors (sensor_name, is_active) VALUES ($1, true)", &[&data.children]).unwrap();
      } else {
        println!("Creating/Updating namespace {} with new value {}", data.parent, data.children);
        conn.execute("UPDATE namespace SET children = array_append(children, $2) WHERE parent=$1", &[&data.parent, &data.children]).unwrap();
        let options: Vec<Option<String>> = vec!(data.children).iter().map( |i| Some(i.clone()) ).collect();
        let arr: ArrayBase<Option<String>> = ArrayBase::from_vec(options, 0);
        conn.execute("INSERT INTO namespace (parent, children) VALUES ($1, $2)", &[&data.parent, &arr]).unwrap();
      }
    }
  });

  // let queue: Queue<SensorPacket> = Queue::with_capacity(10);
  // let queue_clone = queue.clone();
  let all_hooks_hookside = Arc::new(RWLock::new(HashMap::new()));

  let all_hooks_sensorside = all_hooks_hookside.clone();
  spawn(proc() {
    let sensor_listener = TcpListener::bind("127.0.0.1:8000");
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
          let sql_sensor_sender_clone = sql_sensor_sender.clone();
          let namespace_sender_clone = namespace_sender.clone();
          let all_hooks = all_hooks_sensorside.clone();
          spawn(proc() {
            handle_sensor(stream, sql_sensor_sender_clone, namespace_sender_clone, all_hooks);
          });
        }
      }
    }
  });

  let hook_listener = TcpListener::bind("127.0.0.1:8001");
  let mut hook = hook_listener.listen();
  // accept connections and process them, spawning a new tasks for each one
  for stream in hook.incoming() {
    match stream {
      Err(e) => {
        println!("Error connecting to a hook {}", e);
        break;
      }
      Ok(stream) => {
        handle_hook(stream, &all_hooks_hookside);
        // let q = queue.clone();
        // spawn(proc() {
        //   handle_hook(stream, q);
        // });
      }
    }
  }
  drop(hook)
}