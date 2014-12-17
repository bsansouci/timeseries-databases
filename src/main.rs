extern crate time;
extern crate postgres;

use std::str;
use std::io::{Acceptor, Listener};
use std::io::{TcpListener, TcpStream};

use std::collections::HashMap;
use std::collections::RingBuf;
use std::sync::{Mutex, Arc};

use time::Timespec;
use std::io::timer;
use std::time::Duration;

use postgres::{Connection, SslMode};
use postgres::types::array::{ArrayBase};

use std::num::Float;
use std::f64::MAX_VALUE;
use std::f64::MIN_VALUE;

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

struct Hook {
  channel: TcpStream,
  is_connected: bool,
  state: RingBuf<f64>,
  command: String,
  window: uint
}

type Eric = Arc<Mutex<HashMap<String, RingBuf<Hook>>>>;

fn read_str(stream: &mut TcpStream) -> String {
  let length = stream.read_be_u32().unwrap();

  let mut t: Vec<u8> = Vec::from_elem(length.to_uint().unwrap(), 0);
  let tmp: &mut [u8] = t.as_mut_slice();

  stream.read(tmp).unwrap();

  return str::from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_string();
}

// fn create_namespace(ns: Sender<NamespacePacket>, namespace: &Vec<&str>) {
//   let mut not_first = false;
//   let mut prev: &str = "ERROR";
//   for n in namespace.iter() {
//     if not_first {
//       ns.send(NamespacePacket {
//         parent: prev.to_string(),
//         children: n.to_string(),
//         is_sensor: false
//       });
//     }
//     prev = *n;
//     not_first = true;
//   }

//   ns.send(NamespacePacket {
//     parent: "".to_string(),
//     children: namespace[namespace.len() - 1].to_string(),
//     is_sensor: true
//   });
// }

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
//         stream.lock(format!("{}:{}", x.name, x.value).as_bytes());
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
//     //   stream.lock(v)
//     // }
//     // stream.lock(format!("{:s}", mainChannel.recv().to_string()).as_bytes());
//   // }

//   drop(stream)
// }

fn add_sensor(all_hooks: &Eric, name: String) -> bool {
  let mut w = all_hooks.lock();
  if w.contains_key(&name) {
    println!("Sensor with name `{}` already exists.", name);
    return false;
  }

  w.insert(name, RingBuf::new());
  return true;
}

fn min(state: &mut RingBuf<f64>) -> f64 {
  let mut min = MAX_VALUE;
  for e in state.iter() {
    if *e < min {
      min = *e;
    }
  }
  return min;
}

fn max(state: &mut RingBuf<f64>) -> f64 {
  let mut max = MIN_VALUE;
  for e in state.iter() {
    if *e > max {
      max = *e;
    }
  }
  return max;
}

fn mean(state: &mut RingBuf<f64>) -> f64 {
  let mut sum = 0.0;
  for e in state.iter() {
    sum += *e;
  }

  let size: f64 = state.len() as f64;
  return sum / size;
}

// Make sure this doesn't overflow
// This can be done by pushing new nodes inside the state when hook.state[0]
// becomes too big. Then we return the sum of all the nodes inside the state
// vector.
fn count(state: &mut RingBuf<f64>) -> f64 {
  if state.len() == 0 {
    state.push_back(0.0);
    return state[0];
  }

  state[0] += 1.0;

  return state[0];
}

fn sum(state: &mut RingBuf<f64>) -> f64 {
  let mut total = 0.0;
  for e in state.iter() {
    total += *e;
  }
  return total;
}

// Standard Deviation through sampling (not population)
fn standard_deviation(state: &mut RingBuf<f64>) -> f64 {
  let mut sum = 0.0;
  for e in state.iter() {
    sum += *e;
  }

  let size: f64 = state.len() as f64;
  let mean = sum / size;

  sum = 0.0;
  for e in state.iter() {
    sum += (*e - mean).powi(2);
  }

  return (sum / (size - 1.0)).sqrt();
}

// Variance through sampling (not population)
fn variance(state: &mut RingBuf<f64>) -> f64 {
  let mut sum = 0.0;
  for e in state.iter() {
    sum += *e;
  }

  let size: f64 = state.len() as f64;
  let mean = sum / size;

  sum = 0.0;
  for e in state.iter() {
    let tmp = (*e - mean);
    sum += tmp * tmp;
  }

  return sum / (size - 1.0);
}

// Simple linear regression implementation that returns the slope
// for y = ax + b (returns a)
fn slope(state: &mut RingBuf<f64>) -> f64 {
  let mut sumXY = 0.0;
  let mut sumX = 0.0;
  let mut sumY = 0.0;
  let mut sum2X = 0.0;
  let mut x = 0.0;
  for y in state.iter() {
    sumX += x;
    sumY += *y;
    sum2X += x * x;
    sumXY += *y * x;
    x += 1.0;
  }

  let N = state.len() as f64;
  let slope = (N * sumXY - sumX * sumY) / (N * sum2X - sumX * sumX);
  return slope;
}

fn add_to_state(state: &mut RingBuf<f64>, x: f64, window: uint) {
  if state.len() < window {
    state.push_back(x);
  } else {
    state.pop_front();
    state.push_back(x);
  }
}

fn send_to_all_hooks_f64(all_hooks: &Eric, name: String, x: f64) {
  let mut w = all_hooks.lock();

  match w.get_mut(&name) {
    Some(buffer) => {
      for e in buffer.iter_mut() {
        if e.is_connected {
          // Not sure of what I'm doing anymore
          let ref mut state = e.state;

          // modify the state
          add_to_state(state, x, e.window);

          let value = match e.command.as_slice() {
            "new_value"          => x,
            "min"                => min(state),
            "max"                => max(state),
            "mean"               => mean(state),
            "sum"                => sum(state),
            "variance"           => variance(state),
            "standard_deviation" => standard_deviation(state),
            "count"              => count(state),
            "slope"              => slope(state),
            _ => x,
          };

          match e.channel.write(format!("{}", value).as_bytes()) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: {}", err);
              e.is_connected = false;
            }
          }
        }
      }
    }
    None => println!("Problem here.")
  }
}

fn send_to_all_hooks_str(all_hooks: &Eric, name: String, message: String) {
  let mut w = all_hooks.lock();
  let m = message.as_bytes();

  match w.get_mut(&name) {
    Some(buffer) => {
      for e in buffer.iter_mut() {
        if e.is_connected {
          match e.channel.write(m) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: {}", err);
              e.is_connected = false;
            }
          }
        }
      }
    }
    None => println!("Problem here.")
  }
}

fn handle_sensor(mut stream: TcpStream, db: Sender<SensorPacket>, ns: Sender<NamespacePacket>, all_hooks: Eric) {
  let namespace: String = read_str(&mut stream);

  // let str_split: Vec<&str> = namespace.as_slice().split_str(".").collect();
  // create_namespace(ns, &str_split);
  // let sensor_name: String = str_split[str_split.len() - 1].to_string();

  println!("WARNING: please comment out line 95 of main.rs now that you've created the namespace for that specific sensor. If you don't, you'll get an error next time because it will try to insert a new row with the same key as a previous one.");


  if !add_sensor(&all_hooks, namespace.clone()) {
    drop(stream);
    return;
  }

  println!("New sensor with name {}", namespace);

  loop {
    let req = stream.read_be_f64();
    match req {
      Ok(x) => {
        println!("HANDLE_SENSOR got {}", x);
        // Save the data in the DB
        // db.send(SensorPacket {
        //   value: x,
        //   name: sensor_name.clone(),
        //   time_created: time::get_time()
        // });

        // Send the data to all the hooks
        send_to_all_hooks_f64(&all_hooks, namespace.clone(), x);
      },
      Err(e) => {
        println!("Error reading sensor data: {}", e);
        send_to_all_hooks_str(&all_hooks, namespace.clone(), format!("closed"));

        let remove = |n: &String| {
          let mut unlocked_map = all_hooks.lock();
          unlocked_map.remove(n);
        };
        remove(&namespace);

        break;
      },
    }
  }
  drop(stream);
}

fn handle_hook(mut stream: TcpStream, all_hooks: &Eric) {
  let req: String = read_str(&mut stream);

  let v: Vec<&str> = req.as_slice().split_str(":").collect();

  let command_set: Vec<&str> = v[1].as_slice().split_str(" ").collect();
  let window: uint = match from_str(command_set[1]) {
    Some(x) => x,
    None => 0,
  };


  let namespaces: String = v[0].to_string();
  // let splitted_namespace: Vec<&str> = v[0].split_str(".").collect();
  // let wanted_sensor = namespaces[namespaces.len() - 1].to_string();

  let mut unlocked_map = all_hooks.lock();

  match unlocked_map.get_mut(&namespaces) {
    Some(list) => {
      println!("New hook for sensor `{}`", namespaces);
      list.push_back(Hook {
        channel: stream.clone(),
        is_connected: true,
        state: RingBuf::new(),
        command: command_set[2].to_string(),
        window: window
      });
    }
    None => println!("Sorry, the sensor `{}` isn't available.", namespaces)
  }
}

fn main() {
  let (sql_sensor_sender, sql_sensor_receiver): (Sender<SensorPacket>, Receiver<SensorPacket>) = channel();

  spawn(proc() {
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

  spawn(proc() {
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
  let all_hooks_hookside = Arc::new(Mutex::new(HashMap::new()));

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

  let all_hooks_cleanup = all_hooks_hookside.clone();
  spawn(proc() {
    loop {
      timer::sleep(Duration::seconds(10));
      let mut w = all_hooks_cleanup.lock();
      println!("Cleaning up the disconnected hooks...");
      for (_, e) in w.iter_mut() {
        let mut i = 0;
        let length = e.len();
        while i < length {
          if !e[i].is_connected {
            let end = e.len() - 1;
            e.swap(i, end);
            e.pop_back();
          }
          i += 1;
        }
      }
      println!("Done cleaning up");
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
      }
    }
  }
  drop(hook)
}