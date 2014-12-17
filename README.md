timeseries-databases
====================

Research project

For this to work you'll need a postgreSQL database running on port 5432 with a db called "local", a user called "root" and a password "password".

Cargo isn't fully set up yet.

# Easy install - please don't explain just give me one-liner install
Simply run
```bash
git clone https://github.com/bsansouci/timeseries-databases.git && 
curl -s https://static.rust-lang.org/rustup.sh | sudo sh &&
cargo run
```

# How to build
If you already have the Rust compiler installed, you can build and run the main server simply by running
```bash
cargo run
```


# Sensors
To build the fake sensor run
```
mkdir sensors

rustc src/sensor.rs -o sensors/sensor
```

And run the fake sensor in another terminal
```
sensors/sensor "house1.firstfloor.sensor1"
```


# Hooks
To run a hook simply do
```bash
node example-hooks/hook1.js
```