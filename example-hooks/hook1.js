var net = require('net');

var HOST = '127.0.0.1';
var PORT = 8001;

var client = new net.Socket();
var sensorName = '*';

var namespace = 'house.floor1.room1.' + sensorName;
var command = 'new 5 new_value';
client.connect(PORT, HOST, function() {
  console.log('CONNECTED TO: ' + HOST + ':' + PORT);
  writeString(client, namespace + ':' + command);
});

client.on('data', function(data) {
  console.log('DATA: ', data.toString());
  processData(data.toString());
});

client.on('close', function() {
  console.log('Connection closed');
  client.destroy();
});

function processData(data) {
  if(data === "closed") {
    console.log("Closing connection because sensor is down");
    client.destroy();
  }
}

function writeString(client, str) {
  var buf = new Buffer(4);
  buf.writeInt32BE(str.length, 0);
  client.write(buf);
  client.write(str);
}
