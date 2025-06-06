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
    legend: {
        show: false,
    },
    colors: ["#FEB019",
        function({ value }) {
            if (value <= 2) {
                return "#FF4560"
            } else if (value > 2 && value <= 7) {
                return "#FEB019"
            } else {
                return "#00E396"
            }
        }],
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
        text: 'Current SoC & Policy',
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

function refreshData() {
    $.getJSON('/combined_realtime', function(response) {
        realtime.updateSeries([response[0]]);
        soc.updateSeries([response[1]]);
    });

    $.getJSON('/tariffs_buy', function(response) {
        tariffs_buy.updateSeries([response])
    });

    let datehour = new Date();
    datehour.setMinutes(0,0,0);
    let offset = datehour.getTimezoneOffset() * 60 * 1000;

    tariffs_buy.updateOptions({
        annotations: {
            xaxis: [
                {
                    x: datehour.getTime() - offset,
                },
            ]
        }
    });
}

refreshData();
setInterval(() => {
    refreshData();
}, 60000);
