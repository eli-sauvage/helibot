const ChartJSImage = require('chart.js-image');
setInterval(()=>{
    const line_chart = ChartJSImage()
        .chart({
        type: 'bar',
        data: { labels: ['Hello world', 'Foo bar'], datasets: [{ label: 'Foo', data: [1, (new Date()).getSeconds()] }] },
        }) // Line chart
  .backgroundColor('white')
  .width(500) // 500px
  .height(300); // 300px
    line_chart.toFile(__dirname  + '/chart.png')
}, 1000);