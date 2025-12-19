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
        if (resp.policy !== "Green") {
            color = resp.policy
        }

        let temp_current = Math.round(resp.temp_current * 10) / 10;
        let temp_perceived = Math.round(resp.temp_perceived * 10) / 10

        $("#policy-bar").width("100%").css("background-color", color);
        $("#current-temp").text(temp_current + " (" + temp_perceived + ") ℃");
        $("#minmax-today").text("Today: " + resp.today_max + " / " + resp.today_min + " ℃");
        $("#minmax-yesterday").text("Yesterday: " + resp.yesterday_max + " / " + resp.yesterday_min + " ℃");
        
        realtime.updateSeries([resp.current_prod_load]);
        soc.updateSeries([resp.current_soc_soh]);
        tariffs_buy.updateSeries([resp.tariffs_buy]);
        production.updateSeries(resp.prod_diagram);
        load.updateSeries(resp.load_diagram);
        cloud.updateSeries([resp.cloud_diagram]);
        temp.updateSeries(resp.temp_diagram);

        let datetime = new Date();
        let coeff = 1000 * 60 * 15;
        let datetime_quarters = new Date(Math.floor((datetime.getTime() - resp.time_delta) / coeff) * coeff);

        tariffs_buy.updateOptions({
            annotations: {
                xaxis: [
                    {
                        x: datetime_quarters.getTime(),
                    },
                ]
            }
        });
        temp.updateOptions({
            annotations: {
                xaxis: [
                    {
                        x: datetime.getTime() - resp.time_delta,
                    },
                ]
            }
        });
    });
}

loadScriptSequentially('locale_se.js')
    .then(() => loadScriptSequentially('mygrid_realtime.js'))
    .then(() => loadScriptSequentially('mygrid_soc_soh.js'))
    .then(() => loadScriptSequentially('mygrid_tariffs.js'))
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
