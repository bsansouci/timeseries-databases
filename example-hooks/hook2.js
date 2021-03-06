var net = require('net');

var HOST = '104.131.55.110';
var PORT = 8001;

var express = require("express");
var app = express();
var server = require('http').createServer(app);
var io = require('socket.io').listen(server);

var sensorName = '*';
var namespace = 'house.floor1.room1.' + sensorName;
var command = 'new 5 mean';
var client = new net.Socket();

server.listen(8080);
console.log(__dirname);
app.use(express.static(__dirname));
app.get('/', function (req, res) {
  res.sendFile('index.html', {root: __dirname});
});

io.on('connection', function (socket) {
  client.connect(PORT, HOST, function() {
    console.log('CONNECTED TO: ' + HOST + ':' + PORT);
    writeString(client, namespace + ':' + command);
  });

  client.on('data', function(data) {
    var str = data.toString();
    var splitStr = str.split(":");
    console.log("latency", Date.now() - parseInt(splitStr[0]), "sensor:", splitStr[1], "val", parseFloat(splitStr[2]));
    socket.emit('d', { timeStamp: parseInt(splitStr[0]), sensorName: splitStr[1], value: parseFloat(splitStr[2]) });
    // console.log('DATA:', data.toString());
    processData(data.toString());
  });

  client.on('close', function() {
    console.log('Connection closed');
    client.destroy();
  });
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
