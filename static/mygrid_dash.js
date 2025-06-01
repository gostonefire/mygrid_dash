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
var realtime_options = {
    series: [],
    chart: {
        height: 350,
        type: 'bar',
    },
    colors: ["#00E396", "#FF4560", "#FEB019"],
    stroke: {
        show: true,
        width: 3,
        //colors: ['transparent']
    },
    fill: {
        type:'solid',
        opacity: [0.7, 0.7, 0.7],
    },
    plotOptions: {
        bar: {
            columnWidth: '70%',
            dataLabels: {
                position: 'top',
            }
        }
    },
    dataLabels: {
        enabled: true,
        formatter: function(value, { seriesIndex }) {
            if (seriesIndex <= 1) {
                return value + " kW";
            } else {
                return value + "%";
            }
        },
        offsetY: -20,
        style: {
            fontSize: '12px',
        }
    },
    yaxis: [
        {
            seriesName: 'Production',
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
        {
            seriesName: 'Production',
            show: false,
        },
        {
            seriesName: 'SoC',
            opposite: true,
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
                    return Math.round(val) + "%";
                }
            }
        }
    ],
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
        text: 'Realtime',
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


var realtime = new ApexCharts(document.querySelector("#realtime"), realtime_options);
realtime.render();

// combined production and estimated production
//
var prod_options = {
    series: [],
    chart: {
        height: 350,
        type: 'line',
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
        enabled: true,
        shared: true,
        x: {
            show: true,
            format: 'HH:mm',
        },
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


var production = new ApexCharts(document.querySelector("#production"), prod_options);
production.render();

// combined estimated and history load
//
var load_options = {
    series: [],
    chart: {
        height: 350,
        type: 'line',
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
        enabled: true,
        shared: true,
        x: {
            show: true,
            format: 'HH:mm',
        },
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

var load = new ApexCharts(document.querySelector("#load"), load_options);
load.render();

function refreshData() {
    $.getJSON('https://hobbylap.gridfire.org:8080/combined_realtime', function(response) {
        realtime.updateSeries(response)
    });

    $.getJSON('https://hobbylap.gridfire.org:8080/combined_production', function(response) {
        production.updateSeries(response)
    });

    $.getJSON('https://hobbylap.gridfire.org:8080/combined_load', function(response) {
        load.updateSeries(response)
    });
}

refreshData();
setInterval(() => {
    refreshData();
}, 60000);
