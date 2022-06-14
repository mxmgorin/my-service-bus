class main {
    private static body: HTMLElement;
    private static layoutElement: HTMLElement;
    private static topicsElement: HTMLElement;
    private static connectionsElement: HTMLElement;
    private static statusBarElement: HTMLElement;
    private static requested = false;

    private static windowHeight: number;
    private static windowWidth: number;

    private static statusBarHeight = 24;

    static generatePosition(left: number, top: number, width: number, height: number): string {
        return 'top:' + top + 'px; left:' + left + 'px; width:' + width + 'px; height:' + height + 'px';
    }

    static resize() {

        let height = window.innerHeight;
        let width = window.innerWidth;


        if (this.windowHeight == height && this.windowWidth == width)
            return;

        this.windowHeight = height;
        this.windowWidth = width;

        let sbHeight = this.statusBarHeight;

        this.layoutElement.setAttribute('style',
            this.generatePosition(0, 0, width, height - sbHeight));

        this.statusBarElement.setAttribute('style',
            'position:absolute; ' + this.generatePosition(0, height - sbHeight, width, sbHeight))

    }

    static filterLines(filterPhrase: string) {
        let filter_lines = document.getElementsByClassName("filter-line");

        for (let i = 0; i < filter_lines.length; i++) {
            let el = filter_lines.item(i);

            if (Utils.filterIt(el.innerHTML, filterPhrase)) {
                el.classList.add('hidden');
            }
            else {
                el.classList.remove('hidden');
            }
        }
    }

    static background() {

        if (!this.body) {
            this.body = document.getElementsByTagName('body')[0];
            this.body.innerHTML = HtmlMain.layout();

            this.layoutElement = document.getElementById('main');
            this.topicsElement = document.getElementById('topics');
            this.connectionsElement = document.getElementById('connections');
            this.statusBarElement = document.getElementById('status-bar');
        }

        this.resize();


        if (this.requested)
            return;

        this.requested = true;


        $.ajax({ url: '/status', type: 'get' })
            .then((result: IStatusApiContract) => {
                this.requested = false;

                let filterPhrase = (<HTMLInputElement>document.getElementById('filter')).value;

                filterPhrase == filterPhrase.trim();

                let filterPhraseIsChanged = ServiceLocator.checkIfFilterPhraseIsChanged(filterPhrase);

                if (filterPhraseIsChanged) {
                    console.log("filterPhraseIsChanged");
                }

                let topics_are_changed = ServiceLocator.checkIfTopicsAreChanged(result.topics);

                if (topics_are_changed) {
                    console.log("topics_are_changed");
                }

                if (topics_are_changed || filterPhraseIsChanged) {
                    this.topicsElement.innerHTML = HtmlTopics.renderTopics(result.topics);
                    ServiceLocator.topics = result.topics;
                }
                else {
                    HtmlTopics.updateTopicData(result.topics);
                }

                if (ServiceLocator.checkIfSessionsAreChanged(result.sessions)) {
                    this.connectionsElement.innerHTML = HtmlSessions.renderSessions(result);
                    ServiceLocator.sessions = result.sessions;
                }
                else {
                    HtmlSessions.updateSessionData(result);
                }

                HtmlTopics.updateTopicSessions(result);

                HtmlTopics.updateTopicQueues(result);

                HtmlStatusBar.updateStatusbar(result);

                HtmlStatusBar.updateSessionsAmount(result.sessions.items.length);

                this.filterLines(filterPhrase);



            }).fail(() => {
                this.requested = false;
                HtmlStatusBar.updateOffline();
            })

    }

}


let $: any;

window.setInterval(() => main.background(), 1000);


window.onload = () => {
    main.background();
};