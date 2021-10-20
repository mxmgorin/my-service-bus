var HtmlMain = /** @class */ (function () {
    function HtmlMain() {
    }
    HtmlMain.layout = function () {
        return '<div id="main"><div>Filter:<input id="filter" class="form-control" style="width:300px; background-color:gray; color:white"></input></div><div id="topics"></div><h1>Connections</h1><div id="connections"></div></div>' +
            HtmlStatusBar.layout();
    };
    HtmlMain.drawLed = function (enabled, color) {
        return enabled ?
            '<div class="led-' + color + '"></div>'
            : '<div class="led-gray"></div>';
    };
    return HtmlMain;
}());
//# sourceMappingURL=HtmlMain.js.map