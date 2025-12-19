// tariffs buy
//
let tariffs_buy_options= {
    series: [],
    chart: {
        height: 200,
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
        max: 8,
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
            datetimeUTC: false,
        },
    },
    tooltip: {
        enabled: false,
    },
    title: {
        text: 'Tariffs',
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
