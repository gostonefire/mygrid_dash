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
