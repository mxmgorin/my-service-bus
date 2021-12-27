var main = /** @class */ (function () {
    function main() {
    }
    main.generatePosition = function (left, top, width, height) {
        return 'top:' + top + 'px; left:' + left + 'px; width:' + width + 'px; height:' + height + 'px';
    };
    main.resize = function () {
        var height = window.innerHeight;
        var width = window.innerWidth;
        if (this.windowHeight == height && this.windowWidth == width)
            return;
        this.windowHeight = height;
        this.windowWidth = width;
        var sbHeight = this.statusBarHeight;
        this.layoutElement.setAttribute('style', this.generatePosition(0, 0, width, height - sbHeight));
        this.statusBarElement.setAttribute('style', 'position:absolute; ' + this.generatePosition(0, height - sbHeight, width, sbHeight));
    };
    main.filterLines = function (filterPhrase) {
        var filter_lines = document.getElementsByClassName("filter-line");
        for (var i = 0; i < filter_lines.length; i++) {
            var el = filter_lines.item(i);
            if (Utils.filterIt(el.innerHTML, filterPhrase)) {
                el.classList.add('hidden');
            }
            else {
                el.classList.remove('hidden');
            }
        }
    };
    main.background = function () {
        var _this = this;
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
            .then(function (result) {
            _this.requested = false;
            var filterPhrase = document.getElementById('filter').value;
            filterPhrase == filterPhrase.trim();
            var filterPhraseChanged = filterPhrase != ServiceLocator.prevFilterPhrase || !ServiceLocator.prevFilterPhrase;
            ServiceLocator.prevFilterPhrase = filterPhrase;
            if (ServiceLocator.checkIfTopicsAreChanged(result.topics) || filterPhraseChanged) {
                _this.topicsElement.innerHTML = HtmlTopics.renderTopics(result.topics);
                ServiceLocator.topics = result.topics;
            }
            else {
                HtmlTopics.updateTopicData(result.topics);
            }
            if (ServiceLocator.checkIfSessionsAreChanged(result.sessions)) {
                _this.connectionsElement.innerHTML = HtmlSessions.renderSessions(result);
                ServiceLocator.sessions = result.sessions;
            }
            else {
                HtmlSessions.updateSessionData(result);
            }
            HtmlTopics.updateTopicSessions(result);
            HtmlTopics.updateTopicQueues(result);
            HtmlStatusBar.updateStatusbar(result);
            _this.filterLines(filterPhrase);
        }).fail(function () {
            _this.requested = false;
            HtmlStatusBar.updateOffline();
        });
    };
    main.requested = false;
    main.statusBarHeight = 24;
    return main;
}());
var $;
window.setInterval(function () { return main.background(); }, 1000);
window.onload = function () {
    main.background();
};
//# sourceMappingURL=main.js.map