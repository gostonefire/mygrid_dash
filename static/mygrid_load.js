// combined estimated and history load
//
let load_options = {
    series: [],
    chart: {
        id: 'load',
        group: 'mygrid',
        height: 200,
        type: 'line',
        toolbar: {
            show: false,
        },
        zoom: {
            enabled: false,
        },
    },
    colors: ["#008FFB", "#FF4560"],
    stroke: {
        curve: 'smooth',
        width: [2,2],
    },
    fill: {
        type:'solid',
        opacity: [0.35, 1],
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
                return val + " kW";
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
        text: 'Power Load',
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

let load = new ApexCharts(document.querySelector("#load"), load_options);
load.render();
