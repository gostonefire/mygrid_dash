function loadScriptSequentially(file) {
    return new Promise((resolve, reject) => {
        const newScript = document.createElement('script');
        newScript.setAttribute('src', file);
        newScript.setAttribute('async', 'true');

        newScript.onload = () => {
            displayMessage(`${file} loaded successfully`, 'success');
            resolve(); // Resolve the promise
        };
        newScript.onerror = () => {
            displayMessage(`Error loading script: ${file}`, 'error');
            reject(new Error(`Error loading script: ${file}`));
        };

        document.head.appendChild(newScript);
    });
}

function refreshData() {
    $.getJSON('/combined_realtime', function(response) {
        realtime.updateSeries([response[0]]);
        soc.updateSeries([response[1]]);
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

loadScriptSequentially('locale_se.js')
    .then(() => loadScriptSequentially('mygrid_realtime.js'))
    .then(() => loadScriptSequentially('mygrid_soc_policy.js'))
    .then(() => loadScriptSequentially('mygrid_tariffs_buy.js'))
    .then(() => loadScriptSequentially('mygrid_prod.js'))
    .then(() => loadScriptSequentially('mygrid_load.js'))
    .then(() => loadScriptSequentially('mygrid_sync_weather.js'))
    .then(() => {
        refreshData();
        setInterval(() => {
            refreshData();
        }, 60000);
    })
    .catch(error => displayMessage(error.message, 'error'));



function displayMessage(message, type) {
    console.log(message, type);
}
