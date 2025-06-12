// Usage Policy
//
let policy_options = {
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
    legend: {
        show: false,
    },
    colors: [function({ value }) {
            if (value <= 20) {
                return "#FF4560"
            } else if (value > 20 && value < 70) {
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
        text: 'Policy',
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


let policy = new ApexCharts(document.querySelector("#policy"), policy_options);
policy.render();
