var net = require('net');

var HOST = '127.0.0.1';
var PORT = 8001;

var client = new net.Socket();
client.connect(PORT, HOST, function() {
  console.log('CONNECTED TO: ' + HOST + ':' + PORT);
  writeString(client, 'house1.firstfloor.sensor1:new_value');
});

client.on('data', function(data) {
  console.log('DATA: ', data.toString());
  if(data.toString().split(":")[1] === "closed") {
    console.log("Closing connection because sensor is down");
    client.destroy();
  }
});

client.on('close', function() {
  console.log('Connection closed');
  client.destroy();
});

function writeString(client, str) {
  var buf = new Buffer(4);
  buf.writeInt32BE(str.length, 0);
  client.write(buf);
  client.write(str);
}
