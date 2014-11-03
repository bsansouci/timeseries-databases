use std::io::TcpStream;
use std::io::IoResult;
use std::rand;
use std::rand::Rng;
use std::io::Timer;
use std::time::duration::Duration;

fn main() {
  let mut socket: TcpStream = TcpStream::connect("127.0.0.1", 8000).unwrap();
  let interval = Duration::milliseconds(1000);
  let mut timer = Timer::new().unwrap();
  let mut rng = rand::task_rng();
  let two_pi = 6.28318530718;
  let namespace: &str = "house1.first-floor.sensor1";

  send_str(namespace, &mut socket).unwrap();

  // Iterate 100000 times (long enough to test out things)
  for i in range(0f64, 10000f64) {
    // Start a one second timer
    let oneshot: Receiver<()> = timer.oneshot(interval);
    println!("Wait {} ms...", interval.num_milliseconds());

    // Block the task until notification arrives (aka the timer is done)
    oneshot.recv();

    // Generate a random number (alternates between -40 and 40 slowly)
    let mid: f64 = (i / two_pi).sin();
    let num: f64 = rng.gen_range(mid * 40.0 - 1.0, mid * 40.0 + 1.0);

    // Send the random number through sockets to the server
    match socket.write_be_f64(num) {
      Ok(x) => println!("{}", x),
      Err(x) => println!("{}", x),
    }
    println!("sent {}", num)
  }

  drop(socket)
}

fn send_str(string: &str, socket: &mut TcpStream) -> IoResult<()> {
  try!(socket.write_be_uint(string.len()));
  socket.write(string.as_bytes())
}