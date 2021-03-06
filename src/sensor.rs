use std::old_io::TcpStream;
use std::old_io::IoResult;
use std::rand;
use std::rand::Rng;
use std::old_io::Timer;
use std::time::duration::Duration;
use std::os;
use std::num::Float;

fn main() {
  let mut dbname = "house.floor1.room1.sensor1".to_string();

  let args: Vec<String> = os::args();
  if args.len() >= 2 {
    dbname = args[1].clone();
  }

  let namespace: &str = dbname.as_slice();

  let interval = Duration::milliseconds(1000);
  let mut timer = Timer::new().unwrap();
  let mut rng = rand::thread_rng();
  let two_pi = 6.28318530718;

  let mut socket: TcpStream = TcpStream::connect("localhost:8000").unwrap();
  send_str(namespace, &mut socket).unwrap();

  let v: Vec<f64> = rng.gen_iter::<f64>().take(250).collect();

  for i in v.iter() {
    // Start a one second timer
    let oneshot = timer.oneshot(interval);
    println!("Wait {} ms...", interval.num_milliseconds());

    // Block the task until notification arrives (aka the timer is done)
    oneshot.recv().unwrap();

    // Generate a random number (alternates between -40 and 40 slowly)
    let mid: f64 = (*i / two_pi).sin();
    let num: f64 = rng.gen_range(mid * 40.0 - 1.0, mid * 40.0 + 1.0);

    // Send the random number through sockets to the server
    match socket.write_be_f64(num) {
      Ok(x) => x,
      Err(x) => {
        println!("{}", x);
        break;
      },
    }
    println!("sent {}", num)
  }

  drop(socket)
}

fn send_str(string: &str, socket: &mut TcpStream) -> IoResult<()> {
  let u32_length = string.len() as u32;
  try!(socket.write_be_u32(u32_length));
  socket.write(string.as_bytes())
}