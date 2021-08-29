class HtmlGraph {

    public static renderGraph(c: number[], showValue: (v: number) => string, getValue: (v: number) => number, highlight: (v: number) => boolean) {
        const max = Utils.getMax(c);

        const w = 50;

        let coef = max == 0 ? 0 : w / max;

        let result =
            '<svg style="font-size:16px" width="240" height="' +
            w +
            '"> <rect width="240" height="' +
            w +
            '" style="fill:none;stroke-width:;stroke:black" />';

        let i = 0;
        for (let m of c) {
            let y = w - getValue(m) * coef;
            let highLight = highlight(m);
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

            let color = highLight ? "red" : "darkgray";

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

        let maxValue = showValue(max);
        return result + '<text x="1" y="16" fill="black">' + maxValue + '</text><text x="0" y="15" fill="lime">' + maxValue + '</text></svg>';
    }

}