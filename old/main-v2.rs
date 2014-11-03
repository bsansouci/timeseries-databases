use std::comm;
use std::str::from_utf8;
use std::from_str::from_str;
use std::io::{Acceptor, Listener};
use std::io::{TcpListener, TcpStream};

#[deriving(Clone,Show)]
struct Packet {
  sensor: String,
  value: Option<f64>
}

fn handle_sensor(mut stream: TcpStream, mainChannel: Sender<f64>) {
  let mut buffer = [0u8, ..1024];
  while(true) {
    stream.read(buffer);
    let req = from_utf8(buffer).expect("unable to parse buffer to a utf8 string").to_owned();

    let v: Vec<&str> = req.as_slice().split_str(":").collect();

    // let p = Packet {sensor: String::from_str(v[0]), value: from_str::<f64>(v[1])};
    // mainChannel.send(p);
    let p: f64 = match from_str::<f64>(v[1]) {
      Some(x) => x,
      None    => 0.0
    };
    mainChannel.send(p);
    println!("{:s}", v[1]);
  }
  drop(stream);
}

fn handle_hook(mut stream: TcpStream, mainChannel: Receiver<f64>) {
  let mut tmp = [0u8, ..1024];

  stream.read(tmp);

  let req = from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_owned();
  let v: Vec<&str> = req.as_slice().split_str(":").collect();
  let sensorName = String::from_str(v[0]);

  let mut buffer = [0u8, ..1024];
  while(true) {
    // let s: Packet = mainChannel.recv();
    // if s.sensor == sensorName {
    //   let v: &[u8] =  match s.value {
    //     Some(x) => format!("{:s}", x.to_string()).as_bytes(),
    //     None    => format!("{}", s.value).as_bytes(),
    //   };
    //   stream.write(v)
    // }
    stream.write(format!("{:s}", mainChannel.recv().to_string()).as_bytes());
  }

  drop(stream)
}

fn hooks_loop(mainChannel: Receiver<f64>) {
  let hook_listener = TcpListener::bind("127.0.0.1", 8079);
  let mut hook = hook_listener.listen();
  // accept connections and process them, spawning a new tasks for each one
  for stream in hook.incoming() {
    match stream {
      Err(e) => { /* connection failed */ }
      Ok(stream) => spawn(proc() {
        handle_hook(stream, mainChannel.clone())
      })
    }
  }

  drop(hook)
}

fn sensors_loop(mainChannel: Sender<f64>) {
  let sensor_listener = TcpListener::bind("127.0.0.1", 8080);
  let mut sensors = sensor_listener.listen();

  // accept connections and process them, spawning a new tasks for each one
  for stream in sensors.incoming() {
    match stream {
      Err(e) => { /* connection failed */ }
      Ok(stream) => spawn(proc() {
        // connection succeeded
        handle_sensor(stream, mainChannel)
      })
    }
  }

  drop(sensors);
}

fn main() {
  let (tx, rx): (Sender<f64>, Receiver<f64>) = channel();

  spawn(proc() {
    hooks_loop(rx);
  });
  sensors_loop(tx);
}