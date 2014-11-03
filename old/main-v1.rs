use std::str::from_utf8;
use std::io::{Acceptor, Listener};
use std::io::net::ip::SocketAddr;
use std::from_str::from_str;
use std::io;
use std::io::{TcpListener, TcpStream};
use std::sync::Arc;

struct Hook {
  stream: TcpStream,
  interest: String,
  function: String,
  id: String
}

fn handle_sensor(mut stream: TcpStream, all_hooks: Vec<Hook>) {
  let mut buffer = [0u8, ..1024];
  while(true) {
    stream.read(buffer);
    let req = from_utf8(buffer).expect("unable to parse buffer to a utf8 string").to_owned();

    let v: Vec<&str> = req.as_slice().split_str(":").collect();

    println!("{:s}", v[0]);
    // stream.write(format!("{:s}", req).as_bytes())
  }
  drop(stream);
}

fn handle_hook(mut stream: TcpStream, mut all_hooks: Vec<Hook>) {
  let mut tmp = [0u8, ..1024];
  stream.read(tmp);
  let req = from_utf8(tmp).expect("unable to parse buffer to a utf8 string").to_owned();
  let v: Vec<&str> = req.as_slice().split_str(":").collect();
  let h = Hook {stream: stream.clone(), id: String::from_str(v[0]), interest: String::from_str(v[1]), function: String::from_str(v[2])};

  all_hooks.push(h);

  let mut buffer = [0u8, ..1024];
  while(true) {
    stream.read(buffer);
    let req = from_utf8(buffer).expect("unable to parse buffer to a utf8 string").to_owned();

    println!("{:s}", req);
    // stream.write(format!("{:s}", req).as_bytes())
  }

  drop(stream)
}

fn hooks_loop(all_hooks: Vec<Hook>) {
  let hook_listener = TcpListener::bind("127.0.0.1", 8079);
  let mut hook = hook_listener.listen();
  // accept connections and process them, spawning a new tasks for each one
  for stream in hook.incoming() {
    match stream {
      Err(e) => { /* connection failed */ }
      Ok(stream) => spawn(proc() {
        // connection succeeded
        handle_hook(stream, all_hooks)
      })
    }
  }

  drop(hook)
}

fn sensors_loop(all_hooks: Vec<Hook>) {
  let sensor_listener = TcpListener::bind("127.0.0.1", 8080);
  let mut sensors = sensor_listener.listen();

  // accept connections and process them, spawning a new tasks for each one
  for stream in sensors.incoming() {
    match stream {
      Err(e) => { /* connection failed */ }
      Ok(stream) => spawn(proc() {
        // connection succeeded
        handle_sensor(stream, all_hooks)
      })
    }
  }

  drop(sensors);
}

fn main() {
  let mut all_hooks: Vec<Hook> = Vec::new();

  spawn(proc() {
    hooks_loop(all_hooks.clone());
  });
  sensors_loop(all_hooks.clone());
}