var Utils = /** @class */ (function () {
    function Utils() {
    }
    Utils.filterIt = function (line, filterPhrase) {
        if (filterPhrase == "")
            return false;
        return line.indexOf(filterPhrase) == -1;
    };
    Utils.copyToClipboardHtml = function (text) {
        return ' style="cursor:pointer" clipboard=' + text + ' onclick="Utils.copyToClipboard()"';
    };
    Utils.copyToClipboard = function (el) {
        var attr = el.attributes.getNamedItem('clipboard');
        if (attr) {
            navigator.clipboard.writeText(attr.value);
        }
    };
    Utils.getMax = function (c) {
        var result = 0;
        for (var _i = 0, c_1 = c; _i < c_1.length; _i++) {
            var i = c_1[_i];
            if (i > result)
                result = i;
        }
        return result;
    };
    Utils.formatNumber = function (n) {
        return n.toString().replace(/(\d)(?=(\d{3})+(?!\d))/g, '$1,');
    };
    Utils.format_bytes = function (n) {
        if (n < 1024) {
            return n.toFixed(2) + "b";
        }
        n = n / 1024;
        if (n < 1024) {
            return n.toFixed(2) + "Kb";
        }
        n = n / 1024;
        if (n < 1024) {
            return n.toFixed(2) + "Mb";
        }
        n = n / 1024;
        return n.toFixed(2) + "Gb";
    };
    Utils.format_duration = function (micros) {
        if (micros == 0)
            return "0";
        if (micros < 1000) {
            return micros + "µs";
        }
        if (micros < 1000000) {
            return (micros / 1000).toFixed(3) + "ms";
        }
        return (micros / 1000000).toFixed(3) + "s";
    };
    return Utils;
}());
//# sourceMappingURL=Utils.js.map