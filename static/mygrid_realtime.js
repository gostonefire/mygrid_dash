// combined realtime values for production and load
//
let realtime_options = {
    series: [],
    chart: {
        height: 350,
        type: 'bar',
        toolbar: {
            show: false,
        },
        zoom: {
            enabled: false,
        },
    },
    legend: {
        show: false,
    },
    colors: ["#00E396", "#FF4560"],
    stroke: {
        show: true,
        width: 2,
    },
    fill: {
        type: 'solid',
        opacity: 0.7,
    },
    plotOptions: {
        bar: {
            columnWidth: '50%',
            distributed: true,
            dataLabels: {
                position: 'top',
            }
        }
    },
    dataLabels: {
        enabled: true,
        formatter: function(value) {
            return value + " kW";
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
            formatter: function (val) {
                return val + " kW";
            }
        }
    },
    xaxis: {
        position: 'bottom',
        type: 'category',
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: false
        },
        labels: {
            show: true,
        },
    },
    tooltip: {
        enabled: false,
    },
    title: {
        text: 'Current Production & Load',
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


let realtime = new ApexCharts(document.querySelector("#realtime"), realtime_options);
realtime.render();
