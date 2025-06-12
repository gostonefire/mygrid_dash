function loadScriptSequentially(file) {
    return new Promise((resolve, reject) => {
        const newScript = document.createElement('script');
        newScript.setAttribute('src', file);
        newScript.setAttribute('async', 'true');

        newScript.onload = () => {
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
    $.getJSON('/policy', function(response) {
        policy.updateSeries([response])
    });

    $.getJSON('/forecast_temp', function(response) {
        temp.updateSeries(response)
    });

    $.getJSON('/tariffs_buy', function(response) {
        tariffs_buy.updateSeries([response])
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
    .then(() => loadScriptSequentially('mygrid_policy.js'))
    .then(() => loadScriptSequentially('mygrid_temp.js'))
    .then(() => loadScriptSequentially('mygrid_tariffs_buy.js'))
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
