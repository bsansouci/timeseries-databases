timeseries-databases
====================

Research project

For this to work you'll need a postgreSQL database running on port 5432 with a db called "local", a user called "root" and a password "password".

Cargo isn't fully set up yet.

# How to build
First start by making a subfolder called target
```
mkdir target
```

Now compile the main server and the fake sensor
```
rustc -L libs src/main.rs -o target/main && rustc -L libs src/sensor.rs -o target/sensor
```

Now run the server in one terminal
```
target/main
```

And run the fake sensor in another terminal
```
target/sensor "some.namespacing.goes.here.sensor1"
```