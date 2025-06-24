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
    $.getJSON('/data/full', function(resp, textStatus, jqXHR) {
        const redirectUrl = jqXHR.getResponseHeader('X-Redirect-Location');
        if (redirectUrl) {
            window.location.replace(redirectUrl);
            return;
        }

        let color = "LimeGreen";
        if (resp.policy <= 20) {
            color = "Red"
        } else if (resp.policy > 20 && resp.policy < 70) {
            color = "Yellow"
        }
        $("#policy-bar").width(resp.policy + "%").css("background-color", color);
        $("#current-temp").text(Math.round(resp.temp_current * 10) / 10 + " ℃");

        realtime.updateSeries([resp.current_prod_load]);
        soc.updateSeries([resp.current_soc_policy]);
        tariffs_buy.updateSeries([resp.tariffs_buy]);
        production.updateSeries(resp.prod_diagram);
        load.updateSeries(resp.load_diagram);
        cloud.updateSeries([resp.cloud_diagram]);
        temp.updateSeries(resp.temp_diagram);
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
    .then(() => loadScriptSequentially('mygrid_cloud.js'))
    .then(() => loadScriptSequentially('mygrid_temp.js'))
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
