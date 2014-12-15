var net = require('net');

var HOST = '127.0.0.1';
var PORT = 8001;

var client = new net.Socket();
var sensorName = 'sensor1';
var namespace = 'house1.firstfloor.' + sensorName;
var command = 'new_value';
client.connect(PORT, HOST, function() {
  console.log('CONNECTED TO: ' + HOST + ':' + PORT);
  writeString(client, namespace + ':' + command);
});

client.on('data', function(data) {
  console.log('DATA: ', data.toString());
  data.toString()
      .split(sensorName + ":")
      .slice(1)
      .map(processData);
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
