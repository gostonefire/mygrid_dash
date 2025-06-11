// synchronized forecast: cloud
//
let cloud_options = {
    series: [],
    chart: {
        id: 'cloud',
        group: 'forecast',
        height: 200,
        type: 'area',
        toolbar: {
            show: false,
        },
        zoom: {
            enabled: false,
        },
    },
    colors: ["#FF4560"],
    stroke: {
        curve: 'smooth',
        width: 2,
    },
    fill: {
        type:'solid',
        opacity: 0.35,
    },
    dataLabels: {
        enabled: false,
    },
    yaxis: {
        min: 0,
        max: 1,
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
                return val;
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
            show: false
        },
        labels: {
            show: false,
        },
    },
    tooltip: {
        enabled: false,
    },
    title: {
        text: 'Cloud Factor',
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

let cloud = new ApexCharts(document.querySelector("#cloud-factor"), cloud_options);
cloud.render();

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
    colors: ["#FEB019", "#00E396"],
    stroke: {
        curve: 'smooth',
        width: 2,
    },
    fill: {
        type:'solid',
        opacity: 1,
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
