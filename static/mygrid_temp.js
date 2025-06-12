// synchronized forecast: temperature
//
let temp_options = {
    series: [],
    chart: {
        id: 'temp',
        group: 'forecast',
        height: 200,
        type: 'line',
        toolbar: {
            show: false,
        },
        zoom: {
            enabled: false,
        },
    },
    colors: ["#FEB019", "#00E396", "#FF4560"],
    stroke: {
        curve: 'smooth',
        width: 2,
    },
    fill: {
        type:'solid',
        opacity: 1,
    },
    dataLabels: {
        enabled: true,
        enabledOnSeries: [2],
        formatter: function (val, opts) {
            return Math.round(val * 10) / 10 + ' ℃'
        },
    },
    yaxis: {
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: false,
        },
        labels: {
            show: true,
            minWidth: 30,
            formatter: function (val) {
                return val + " ℃";
            }
        }
    },
    xaxis: {
        position: 'bottom',
        type: 'datetime',
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: true
        },
        labels: {
            show: true,
        },
    },
    tooltip: {
        enabled: false,
    },
    title: {
        text: 'Temperature',
        floating: true,
        offsetY: 0,
        align: 'center',
    },
    noData: {
        text: 'Loading...'
    },
    theme: {
        mode: 'dark',
        palette: 'palette1',
        monochrome: {
            enabled: false,
            color: '#255aee',
            shadeTo: 'light',
            shadeIntensity: 0.65
        },
    }
};

let temp = new ApexCharts(document.querySelector("#temperature"), temp_options);
temp.render();
