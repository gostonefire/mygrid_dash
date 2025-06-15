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
        let color = "LimeGreen";
        if (response.y <= 20) {
            color = "Red"
        } else if (response.y > 20 && response.y < 70) {
            color = "Yellow"
        }

        $("#policy-bar").width(response.y + "%").css("background-color", color);
    });

    $.getJSON('/forecast_temp', function(response) {
        temp.updateSeries(response.slice(0,2));
        $("#current-temp").text(Math.round(response[2] * 10) / 10 + " ℃");
    });

    $.getJSON('/tariffs_buy', function(response) {
        tariffs_buy.updateSeries([response]);
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
