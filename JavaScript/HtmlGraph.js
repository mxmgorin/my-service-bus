var HtmlGraph = /** @class */ (function () {
    function HtmlGraph() {
    }
    HtmlGraph.renderGraph = function (c, showValue, getValue, highlight) {
        var max = Utils.getMax(c);
        var w = 50;
        var coef = max == 0 ? 0 : w / max;
        var result = '<svg style="font-size:16px" width="240" height="' +
            w +
            '"> <rect width="240" height="' +
            w +
            '" style="fill:none;stroke-width:;stroke:black" />';
        var i = 0;
        for (var _i = 0, c_1 = c; _i < c_1.length; _i++) {
            var m = c_1[_i];
            var y = w - getValue(m) * coef;
            var highLight = highlight(m);
            if (highLight) {
                result +=
                    '<line x1="' +
                        i +
                        '" y1="' +
                        w +
                        '" x2="' +
                        i +
                        '" y2="0" style="stroke:#ed969e;stroke-width:2" />';
            }
            var color = highLight ? "red" : "darkgray";
            result +=
                '<line x1="' +
                    i +
                    '" y1="' +
                    w +
                    '" x2="' +
                    i +
                    '" y2="' +
                    y +
                    '" style="stroke:' + color + ';stroke-width:2" />';
            i += 2;
        }
        var maxValue = showValue(max);
        return result + '<text x="1" y="16" fill="black">' + maxValue + '</text><text x="0" y="15" fill="lime">' + maxValue + '</text></svg>';
    };
    return HtmlGraph;
}());
//# sourceMappingURL=HtmlGraph.js.map