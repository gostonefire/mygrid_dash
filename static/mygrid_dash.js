const locale = {
    "name": "se",
    "options": {
        "months": [
            "Januari",
            "Februari",
            "Mars",
            "April",
            "Maj",
            "Juni",
            "Juli",
            "Augusti",
            "September",
            "Oktober",
            "November",
            "December"
        ],
        "shortMonths": [
            "Jan",
            "Feb",
            "Mar",
            "Apr",
            "Maj",
            "Juni",
            "Juli",
            "Aug",
            "Sep",
            "Okt",
            "Nov",
            "Dec"
        ],
        "days": [
            "Söndag",
            "Måndag",
            "Tisdag",
            "Onsdag",
            "Torsdag",
            "Fredag",
            "Lördag"
        ],
        "shortDays": ["Sön", "Mån", "Tis", "Ons", "Tor", "Fre", "Lör"],
        "toolbar": {
            "exportToSVG": "Ladda SVG",
            "exportToPNG": "Ladda PNG",
            "exportToCSV": "Ladda CSV",
            "menu": "Meny",
            "selection": "Selektion",
            "selectionZoom": "Val av zoom",
            "zoomIn": "Zooma in",
            "zoomOut": "Zooma ut",
            "pan": "Panorering",
            "reset": "Återställ zoomning"
        }
    }
};

Apex.chart = {
    locales: [locale],
    defaultLocale: "se"
}

// combined realtime values for production, load and SoC (State of Charge)
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
            show: false,
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

// Realtime SoC (State of Charge)
//
let soc_options = {
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
    colors: ["#FEB019"],
    stroke: {
        show: true,
        width: 2,
    },
    fill: {
        type:'solid',
        opacity: 0.7,
    },
    plotOptions: {
        bar: {
            columnWidth: '40%',
            dataLabels: {
                position: 'top',
            }
        }
    },
    dataLabels: {
        enabled: true,
        formatter: function(value) {
            return value + "%";
        },
    },
    yaxis: {
        min: 0,
        max: 100,
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: false,
        },
        labels: {
            show: true,
            formatter: function (val) {
                return val + "%";
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
        text: 'Current SoC',
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


let soc = new ApexCharts(document.querySelector("#soc"), soc_options);
soc.render();

// tariffs buy
//
let tariffs_buy_options= {
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
    colors: [
        function({ value }) {
            if (value <= 2) {
                return "#00E396"
            } else if (value > 2 && value <= 4) {
                return "#FEB019"
            } else {
                return "#FF4560"
            }
        }
    ],
    fill: {
        type:'solid',
        opacity: 0.8,
    },
    dataLabels: {
        enabled: false,
    },
    yaxis: {
        min: 0,
        max: 10,
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: false,
        },
        labels: {
            show: true,
            formatter: function (val) {
                return val + " kr";
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
        text: 'Tariffs Buy',
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


let tariffs_buy = new ApexCharts(document.querySelector("#tariffs-buy"), tariffs_buy_options);
tariffs_buy.render();

// combined production and estimated production
//
let prod_options = {
    series: [],
    chart: {
        id: 'prod',
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
    colors: ["#008FFB", "#00E396"],
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
                return val + " kWh";
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
            show: false,
        },
        labels: {
            show: false,
        },
    },
    tooltip: {
        enabled: false,
    },
    title: {
        text: 'Power Production',
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


let production = new ApexCharts(document.querySelector("#production"), prod_options);
production.render();

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
                return val + " kWh";
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
    colors: ["#FEB019"],
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


function refreshData() {
    $.getJSON('/combined_realtime', function(response) {
        realtime.updateSeries([response[0]])
        soc.updateSeries([response[1]])
    });

    $.getJSON('/tariffs_buy', function(response) {
        tariffs_buy.updateSeries([response])
    });

    $.getJSON('/combined_production', function(response) {
        production.updateSeries(response)
    });

    $.getJSON('/combined_load', function(response) {
        load.updateSeries(response)
    });

    $.getJSON('/forecast_cloud', function(response) {
        cloud.updateSeries([response])
    });

    $.getJSON('/forecast_temp', function(response) {
        temp.updateSeries([response])
    });

    let datetime = new Date();
    let datehour = new Date();
    datehour.setMinutes(0,0,0);
    let offset = datetime.getTimezoneOffset() * 60 * 1000;

    tariffs_buy.updateOptions({
        annotations: {
            xaxis: [
                {
                    x: datehour.getTime() - offset,
                },
            ]
        }
    });
    temp.updateOptions({
        annotations: {
            xaxis: [
                {
                    x: datetime.getTime() - offset,
                },
            ]
        }
    });
}

refreshData();
setInterval(() => {
    refreshData();
}, 60000);
