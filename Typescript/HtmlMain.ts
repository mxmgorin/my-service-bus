class HtmlMain {

    public static layout(): string {
        return '<div id="main"><div id="topics"></div><h1>Connections</h1><div id="connections"></div></div>' +
            HtmlStatusBar.layout();

    }

    public static drawLed(enabled: boolean, color: string): string {

        return enabled ?
            '<div class="led-' + color + '"></div>'
            : '<div class="led-gray"></div>';
    }


}