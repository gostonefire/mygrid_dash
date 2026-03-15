const six = 360;
const twenty_two = 1320; // 1320;
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

function clampSoc(value) {
    if (value === null || value === undefined) {
        return null;
    }

    const number = Number(value);
    if (!Number.isFinite(number)) {
        return null;
    }

    return Math.max(0, Math.min(100, number));
}

function getSocBarColor(soc) {
    if (soc === null) {
        return 'transparent';
    }

    if (soc <= 25) {
        return '#FF4560';
    }

    if (soc <= 50) {
        return '#FEB019';
    }

    return '#00E396';
}

function renderSocBar(currentSoc, maxSoc, minSoc) {
    const safeCurrentSoc = clampSoc(currentSoc);
    const safeMaxSoc = clampSoc(maxSoc);
    const safeMinSoc = clampSoc(minSoc);

    const currentLabel = safeCurrentSoc === null ? '--' : `${Math.round(safeCurrentSoc)}%`;
    const minLabel = safeMinSoc === null ? '--' : `${Math.round(safeMinSoc)}%`;
    const maxLabel = safeMaxSoc === null ? '--' : `${Math.round(safeMaxSoc)}%`;

    const fillWidth = safeCurrentSoc === null ? 0 : safeCurrentSoc;
    const fillColor = getSocBarColor(safeCurrentSoc);

    const maxSocLeft = safeMaxSoc === null || safeMaxSoc <= 0 ? null : Math.max(1, Math.min(99, safeMaxSoc));
    const maxSocColor = getSocBarColor(safeMaxSoc);

    const maxSocMarker = maxSocLeft === null
        ? ''
        : `<div class="soc-bar-minmax-marker" style="left: ${maxSocLeft}%; background: ${maxSocColor};"></div>`;

    const minSocLeft = safeMinSoc === null || safeMinSoc <= 0 ? null : Math.max(1, Math.min(99, safeMinSoc));
    const minSocColor = '#343a3f';

    const minSocMarker = minSocLeft === null || safeMinSoc === safeCurrentSoc
        ? ''
        : `<div class="soc-bar-minmax-marker" style="left: ${minSocLeft}%; background: ${minSocColor};"></div>`;

    return `
        <div class="soc-bar-wrapper">
            <div class="soc-bar-fill" style="width: ${fillWidth}%; background: ${fillColor};"></div>
            ${minSocMarker}
            ${maxSocMarker}
            <span class="soc-bar-label">
                <span class="soc-bar-label-text">
                    <span class="soc-value-left">${minLabel}</span>
                    <span class="soc-separator">|</span>
                    <span class="soc-value-center">${currentLabel}</span>
                    <span class="soc-separator">|</span>
                    <span class="soc-value-right">${maxLabel}</span>
                </span>
            </span>
        </div>
    `;
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
        if (resp.policy !== "Green") {
            color = resp.policy
        }

        let temp_current = Math.round(resp.temp_current * 10) / 10;
        let temp_perceived = Math.round(resp.temp_perceived * 10) / 10

        let datetime = new Date();

        let symbols_body = $('#symbols');

        symbols_body.empty();
        for (let i = 0; i < resp.forecast_symbol.length; i++) {
            let row = resp.forecast_symbol[i];

            let d = new Date(row.x);
            let style = '';
            if (d.getHours() < datetime.getHours()) {
                style = 'font-weight: bold;color: dimgray';
            }

            let time = `${String(d.getHours()).padStart(2, '0')}`;
            symbols_body.append('<div  class="symbol">' +
                '<p class="symbol-text" style="' + style + '">' + time + '</p>' +
                '<img src="/symbols/' + row.y + '.webp" alt="" width="30px" height="30px">' +
                '</div>');
        }

        $("#policy-bar").width("100%").css("background-color", color);
        $("#current-temp").text(temp_current + " (" + temp_perceived + ") ℃");
        $("#minmax-today").text("Today: " + resp.today_max + " / " + resp.today_min + " ℃");
        $("#minmax-yesterday").text("Yesterday: " + resp.yesterday_max + " / " + resp.yesterday_min + " ℃");

        temp.updateSeries(resp.temp_diagram);

        if (resp.tariffs_buy != null) {
            $("#tariffs-buy").show();
            tariffs_buy.updateSeries([resp.tariffs_buy]);
        } else {
            $("#tariffs-buy").hide();
        }

        if (resp.tariffs_buy_tomorrow != null) {
            $("#tariffs-buy-tomorrow").show();
            tariffs_tomorrow.updateSeries([resp.tariffs_buy_tomorrow]);
        } else {
            $("#tariffs-buy-tomorrow").hide();
        }

        $("#schedule-saving").text("Scheduling saves: " + (resp.base_cost - resp.schedule_cost).toFixed(2));
        let schedule_body = $('#schedule-body');

        schedule_body.empty();
        for (let i = 0; i < resp.schedule.length; i++) {
            let row = resp.schedule[i];

            const safeTrueSoc = clampSoc(row.true_soc_in);
            const trueSocLabel = safeTrueSoc === null ? '--' : `${Math.round(safeTrueSoc)}`;
            const socInLabel = `${row.soc_in} (${trueSocLabel})%`;
            const socOutLabel = `${row.soc_out}%`;

            schedule_body.append('<tr><td>' + row.block_type + '</td><td>' + row.start + '</td><td>' +
                socInLabel + '</td><td>' + socOutLabel + '</td><td class="soc-cell">' +
                renderSocBar(row.current_soc, row.max_soc, row.min_soc) + '</td><td>' + row.cost + '</td><td>' + row.status + '</td></tr>');
        }

        $("#version").text("Version: " + resp.version);
        
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
        tariffs_buy.updateOptions({
            yaxis: {
                min: 0,
                max: resp.max_tariff,
            }
        });
        tariffs_tomorrow.updateOptions({
            yaxis: {
                min: 0,
                max: resp.max_tariff,
            }
        });
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
    .then(() => loadScriptSequentially('mygrid_tariffs.js'))
    .then(() => loadScriptSequentially('mygrid_tariffs_tomorrow.js'))
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
