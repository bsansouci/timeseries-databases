extern crate time;
extern crate postgres;

use std::str;
use std::thread::Thread;
use std::num::Float;
use std::str::FromStr;

use std::f64::MAX_VALUE;
use std::f64::MIN_VALUE;

use std::old_io::{Acceptor, Listener};
use std::old_io::{TcpListener, TcpStream};

use std::collections::HashMap;
use std::collections::RingBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver, channel};

use time::Timespec;
use std::old_io::timer;
use std::time::Duration;


use postgres::{Connection, SslMode};


type Eric = Arc<Mutex<HashMap<String, (RingBuf<Hook<TcpStream>>, RingBuf<Hook<Sender<f64>>>)>>>;

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

struct Hook<T> {
  channel: T,
  is_connected: bool,
  state: RingBuf<f64>,
  command: String,
  window: usize,
  namespace: String
}

fn read_str(stream: &mut TcpStream) -> String {
  let length = stream.read_be_u32().unwrap();
  let usize_length = length as usize;

  let mut buff: Vec<u8> = stream.read_exact(usize_length).unwrap();
  return str::from_utf8(buff.as_mut_slice()).unwrap().to_string();
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
//     // let s: Packet = mainChannel.recv().unwrap();
//     // if s.sensor == sensorName {
//     //   let v: &[u8] =  match s.value {
//     //     Some(x) => format!("{:s}", x.to_string()).as_bytes(),
//     //     None    => format!("{}", s.value).as_bytes(),
//     //   };
//     //   stream.lock(v)
//     // }
//     // stream.lock(format!("{:s}", mainChannel.recv().unwrap().to_string()).as_bytes());
//   // }

//   drop(stream)
// }

fn add_sensor(all_hooks: &Eric, name: String) -> bool {
  let mut w = all_hooks.lock().unwrap();
  if w.contains_key(&name) {
    println!("Sensor with name `{}` already exists.", name);
    return false;
  }

  w.insert(name, (RingBuf::new(), RingBuf::new()));
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
    let tmp = *e - mean;
    sum += tmp * tmp;
  }

  return sum / (size - 1.0);
}

// Simple linear regression implementation that returns the slope
// for y = ax + b (returns a)
fn slope(state: &mut RingBuf<f64>) -> f64 {
  let mut sum_xy = 0.0;
  let mut sum_x = 0.0;
  let mut sum_y = 0.0;
  let mut sum_2x = 0.0;
  let mut x = 0.0;
  for y in state.iter() {
    sum_x += x;
    sum_y += *y;
    sum_2x += x * x;
    sum_xy += *y * x;
    x += 1.0;
  }

  let n = state.len() as f64;
  let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_2x - sum_x * sum_x);
  return slope;
}

fn add_to_state(state: &mut RingBuf<f64>, x: f64, window: usize) {
  if state.len() < window {
    state.push_back(x);
  } else {
    state.pop_front();
    state.push_back(x);
  }
}

fn send_to_all_hooks_f64(all_hooks: &Eric, name: String, x: f64, time_stamp: i64) {
  let mut w = all_hooks.lock().unwrap();

  match w.get_mut(&name) {
    Some(buffer) => {
      // Sending to all connected hooks
      for e in buffer.0.iter_mut() {
        if e.is_connected {
          // Not sure of what I'm doing anymore
          let ref mut state = e.state;

          // modify the state
          add_to_state(state, x, e.window);

          let value = match e.command.as_slice() {
            "identity"          => x,
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

          match e.channel.write(format!("{}:{}", time_stamp, value).as_bytes()) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: [TEMPORARILY REMOVED SORRY]");
              e.is_connected = false;
            }
          }
        }
      }

      // Sending to all intermediaries
      for e in buffer.1.iter_mut() {
        if e.is_connected {
          // Not sure of what I'm doing anymore
          let ref mut state = e.state;

          // modify the state
          add_to_state(state, x, e.window);

          let value = match e.command.as_slice() {
            "identity"          => x,
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

          match e.channel.send(value) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: [TEMPORARILY REMOVED SORRY]");
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
  let mut w = all_hooks.lock().unwrap();
  let m = message.as_bytes();

  match w.get_mut(&name) {
    Some(buffer) => {
      for e in buffer.0.iter_mut() {
        if e.is_connected {
          match e.channel.write(m) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: [TEMPORARILY REMOVED SORRY]");
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

  // println!("WARNING: please comment out line 95 of main.rs now that you've created the namespace for that specific sensor. If you don't, you'll get an error next time because it will try to insert a new row with the same key as a previous one.");


  if !add_sensor(&all_hooks, namespace.clone()) {
    drop(stream);
    return;
  }

  println!("New sensor with name {}", namespace);

  loop {
    let req = stream.read_be_f64();
    let t = time::get_time();
    let time_stamp = t.sec * 1000 + t.nsec as i64 / 1000000;
    match req {
      Ok(x) => {
        println!("{}:{}:{}", time_stamp, namespace, x);
        // Save the data in the DB
        // db.send(SensorPacket {
        //   value: x,
        //   name: sensor_name.clone(),
        //   time_created: time::get_time()
        // });

        // Send the data to all the hooks
        send_to_all_hooks_f64(&all_hooks, namespace.clone(), x, time_stamp);
      },
      Err(e) => {
        println!("Error reading sensor data: {}", e);
        send_to_all_hooks_str(&all_hooks, namespace.clone(), format!("closed"));

        let remove = |&:n: &String| {
          let mut unlocked_map = all_hooks.lock().unwrap();
          unlocked_map.remove(n);
        };
        remove(&namespace);

        break;
      },
    }
  }
  drop(stream);
}


fn handle_intermediary(mut all_receivers: Vec<Hook<Receiver<f64>>>, mut hook: TcpStream) {
  let mut is_connected = true;
  loop {
    if !is_connected {
      println!("Destroying intermediary");
      return;
    }
    // timer::sleep(Duration::seconds(1));
    for r in all_receivers.iter_mut() {
      match r.channel.try_recv() {
        Ok(x) => {
          let t = time::get_time();
          let time_stamp = t.sec * 1000 + t.nsec as i64 / 1000000;
          match hook.write(format!("{}:{}:{}", time_stamp, r.namespace.clone(), x).as_bytes()) {
            Ok(_) => (),
            Err(err) => {
              println!("Hook probably disconnected, error: [TEMPORARILY REMOVED SORRY]");
              is_connected = false;
            }
          }
        }
        Err(_) => ()
      }
    }
  }
}

fn handle_hook(mut stream: TcpStream, all_hooks: &Eric) {
  let req: String = read_str(&mut stream);

  let v: Vec<&str> = req.as_slice().split_str(":").collect();

  let command_set: Vec<&str> = v[1].as_slice().split_str(" ").collect();
  let window: usize = match FromStr::from_str(command_set[1]) {
    Ok(x) => x,
    Err(e) => 0,
  };


  let namespace: String = v[0].to_string();
  let splitted_namespace: Vec<&str> = v[0].split_str(".").collect();
  let wanted_sensor = splitted_namespace[splitted_namespace.len() - 1].to_string();

  let mut unlocked_map = all_hooks.lock().unwrap();
  if wanted_sensor == "*" {
    println!("Wants all inside {}", namespace);
    let mut namespace_without_star = splitted_namespace.clone();
    namespace_without_star.pop();

    let mut list: Vec<Hook<Receiver<f64>>> = Vec::new();
    for (key, sensor) in unlocked_map.iter_mut() {
      let names: Vec<&str> = key.as_slice().split_str(".").collect();
      let mut i = 0;
      let mut all_good = true;

      for n in namespace_without_star.iter() {
        if *n != names[i] || i >= names.len() {
          all_good = false;
          break;
        }
        i += 1;
      }

      if all_good {
        let (sender, receiver): (Sender<f64>, Receiver<f64>) = channel();
        list.push(Hook {
          channel: receiver,
          is_connected: true,
          state: RingBuf::new(),
          command: command_set[2].to_string(),
          window: window,
          namespace: key.clone()
        });

        sensor.1.push_back(Hook {
          channel: sender,
          is_connected: true,
          state: RingBuf::new(),
          command: command_set[2].to_string(),
          window: window,
          namespace: key.clone()
        });
      }
    }

    Thread::spawn(move || {
      handle_intermediary(list, stream.clone());
    });
  } else {
    match unlocked_map.get_mut(&namespace) {
      Some(tuple) => {
        println!("New hook for sensor `{}`", namespace);
        tuple.0.push_back(Hook {
          channel: stream.clone(),
          is_connected: true,
          state: RingBuf::new(),
          command: command_set[2].to_string(),
          window: window,
          namespace: namespace.clone()
        });
      }
      None => println!("Sorry, the sensor `{}` isn't available.", namespace)
    }
  }
}

fn main() {
  let (sql_sensor_sender, sql_sensor_receiver): (Sender<SensorPacket>, Receiver<SensorPacket>) = channel();

  // Thread::spawn(move || {
  //   let conn = Connection::connect("postgres://root:password@127.0.0.1:5432/local",&SslMode::None).unwrap();
  //   // conn.execute("CREATE TABLE tmp_table (
  //   //                 id              SERIAL PRIMARY KEY,
  //   //                 name            VARCHAR NOT NULL,
  //   //                 time_created    TIMESTAMP NOT NULL,
  //   //                 value           DOUBLE PRECISION
  //   //               )", []).unwrap();
  //   loop {
  //     let data = sql_sensor_receiver.recv().unwrap();
  //     println!("Inserting into tmp_table: {} from {} at {}", data.value, data.name, data.time_created);
  //     // conn.execute("INSERT INTO tmp_table (name, time_created, value) VALUES ($1, $2, $3)", &[&data.name, &data.time_created, &data.value]).unwrap();
  //   }
  // });

  let (namespace_sender, namespace_receiver): (Sender<NamespacePacket>, Receiver<NamespacePacket>) = channel();

  // Thread::spawn(move || {
  //   let conn = Connection::connect("postgres://root:password@127.0.0.1:5432/local",&SslMode::None).unwrap();
  //   // conn.execute("CREATE TABLE active_sensors (
  //   //                 sensor_name      VARCHAR PRIMARY KEY,
  //   //                 is_active        BOOLEAN
  //   //               )", []).unwrap();
  //   loop {
  //     let data = namespace_receiver.recv().unwrap();

  //     // We append this new sensor to the table of sensors (if the parent
  //     // doens't exist, we create it, if it does we add to its list of
  //     // children)
  //     if data.is_sensor {
  //       println!("Setting sensor {} available", data.children);
  //       // conn.execute("UPDATE active_sensors SET is_active=true WHERE sensor_name=$1", &[&data.children]).unwrap();
  //       // conn.execute("INSERT INTO active_sensors (sensor_name, is_active) VALUES ($1, true)", &[&data.children]).unwrap();
  //     } else {
  //       println!("Creating/Updating namespace {} with new value {}", data.parent, data.children);
  //       // conn.execute("UPDATE namespace SET children = array_append(children, $2) WHERE parent=$1", &[&data.parent, &data.children]).unwrap();
  //       // let options: Vec<Option<String>> = vec!(data.children).iter().map( |i| Some(i.clone()) ).collect();
  //       // let arr: ArrayBase<Option<String>> = ArrayBase::from_vec(options, 0);
  //       // conn.execute("INSERT INTO namespace (parent, children) VALUES ($1, $2)", &[&data.parent, &arr]).unwrap();
  //     }
  //   }
  // });

  // let queue: Queue<SensorPacket> = Queue::with_capacity(10);
  // let queue_clone = queue.clone();
  let all_hooks_hookside = Arc::new(Mutex::new(HashMap::new()));

  let all_hooks_sensorside = all_hooks_hookside.clone();
  Thread::spawn(move || {
    let sensor_listener = TcpListener::bind("localhost:8000");
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
          Thread::spawn(move || {
            handle_sensor(stream, sql_sensor_sender_clone, namespace_sender_clone, all_hooks);
          });
        }
      }
    }
  });

  // Garbage collector thread
  let all_hooks_cleanup = all_hooks_hookside.clone();
  Thread::spawn(move || {
    loop {
      timer::sleep(Duration::seconds(10));
      let mut w = all_hooks_cleanup.lock().unwrap();
      println!("Cleaning up the disconnected hooks...");
      for (_, tuple) in w.iter_mut() {
        let mut i = 0;

        let ref mut e = tuple.0;
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

  let hook_listener = TcpListener::bind("localhost:8001");
  let mut hook = hook_listener.listen();
  // accept connections and process them, spawning a new tasks for each one
  for stream in hook.incoming() {
    match stream {
      Err(e) => {
        println!("Error connecting to a hook {}", e);
        break;
      }
      Ok(stream) => {
        println!("Connected");
        handle_hook(stream, &all_hooks_hookside);
      }
    }
  }
  drop(hook);
}