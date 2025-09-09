const six = 360;
const twenty_two = 840; // 1320;
let timer = 0;

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

function refreshData(forceRefresh) {
    const date_now = new Date();
    const now = date_now.getHours() * 60 + date_now.getMinutes();

    const dim_screen = $("#dim_screen");
    if ((now >= twenty_two || now < six) && !forceRefresh) {
        dim_screen.show();
        return;
    }
    dim_screen.hide();

    $.getJSON('/data/small', function(resp, textStatus, jqXHR) {
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
        $("#minmax-today").text("Today: " + resp.today_max + " / " + resp.today_min + " ℃");
        $("#minmax-yesterday").text("Yesterday: " + resp.yesterday_max + " / " + resp.yesterday_min + " ℃");

        temp.updateSeries(resp.temp_diagram);
        tariffs_buy.updateSeries([resp.tariffs_buy]);

        let schedule_body = $('#schedule-body');

        schedule_body.empty();
        for (let i = 0; i < resp.schedule.length; i++) {
            let row = resp.schedule[i];

            schedule_body.append('<tr><td>' + row.block_type + '</td><td>' + row.start + '</td><td>' +
                row.length + '</td><td>' + row.soc_in + '</td><td>' + row.soc_out + '</td><td>' +
                row.status + '</td></tr>');
        }
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

function undimScreen() {
    clearInterval(timer);

    $("#dim_screen").hide();
    refreshData(true);
    setTimeout(() => {
        $("#dim_screen").show();
        timer = setInterval(() => {
            refreshData(false);
        }, 60000);
    }, 60000);
}

loadScriptSequentially('locale_se.js')
    .then(() => loadScriptSequentially('mygrid_temp.js'))
    .then(() => loadScriptSequentially('mygrid_tariffs_buy.js'))
    .then(() => {
        refreshData(true);
        timer = setInterval(() => {
            refreshData(false);
        }, 60000);
    })
    .catch(error => displayMessage(error.message, 'error'));



function displayMessage(message, type) {
    console.log(message, type);
}
