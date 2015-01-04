timeseries-databases
====================

This won't run on the latest compiler because they introduced new concepts that broke one dependency of rust-postgres.

Research project

For this to work you'll need a postgreSQL database running on port 5432 with a db called "local", a user called "root" and a password "password".

Cargo isn't fully set up yet.

# PDEJGMOLI - Please Don't Explain Just Give Me a One-Liner Install
Simply run
```bash
curl -s https://static.rust-lang.org/rustup.sh | sudo sh &&
git clone https://github.com/bsansouci/timeseries-databases.git && 
cd timeseries-databases &&
cargo run
```

# How to Build/Run
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