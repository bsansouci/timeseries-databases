var net = require('net');

var HOST = '127.0.0.1';
var PORT = 8001;

var namespace = 'house1.firstfloor.*';
var command = 'new 5 mean';
var client = new net.Socket();

client.connect(PORT, HOST, function() {
  console.log('CONNECTED TO: ' + HOST + ':' + PORT);
  writeString(client, namespace + ':' + command);
});

client.on('data', middleware(function(time, namespace, value) {
  console.log("latency", Date.now() - time, "sensor:", namespace, "val", value);
}));

client.on('close', function() {
  console.log('Connection closed');
  client.destroy();
});

function middleware(f) {
  return function(data) {
    var str = data.toString();
    processData(data);
    var splitStr = str.split(":");
    return f(parseInt(splitStr[0]), splitStr[1], parseFloat(splitStr[2]));
  };
}

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
