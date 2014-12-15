timeseries-databases
====================

Research project

For this to work you'll need a postgreSQL database running on port 5432 with a db called "local", a user called "root" and a password "password".

Cargo isn't fully set up yet.

# How to build
To build and run the main server simply run
```bash
cargo build
```

To build the fake sensor run
```
mkdir sensors

rustc src/sensor.rs -o sensors/sensor
```

And run the fake sensor in another terminal
```
sensors/sensor "house1.firstfloor.sensor1"
```