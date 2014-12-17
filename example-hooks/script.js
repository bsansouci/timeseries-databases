var chart = c3.generate({
  bindto: '#chart',
  data: {
    columns: []
  }
});

var aggregated = {};
var socket = io.connect('http://localhost:8080');
socket.on('d', function (data) {
  if(!aggregated[data.sensorName]) {
    aggregated[data.sensorName] = [];
  }

  aggregated[data.sensorName].push(data.value);
  if(aggregated[data.sensorName].length > 10) {
    aggregated[data.sensorName].shift();
  }

  var arr = [];
  for (var i in aggregated) {
    arr.push([i].concat(aggregated[i]));
  }

  var min = 100000;
  for (var i = arr.length - 1; i >= 0; i--) {
    if(arr[i].length < min) {
      min = arr[i].length;
    }
  }

  // Hardcore truncating baby
  for (var i = arr.length - 1; i >= 0; i--) {
    arr[i].length = min;
  }

  chart.load({
    columns: arr
  });
});