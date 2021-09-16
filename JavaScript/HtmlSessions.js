var HtmlSessions = /** @class */ (function () {
    function HtmlSessions() {
    }
    HtmlSessions.renderSessions = function (sessions) {
        var result = '<table style="font-size:12px" class="table table-striped table-dark">' +
            '<tr><th style="width:50px">Id</th><th style="width:120px">Info</th><th>Publisher</th><th>Subscriber</th></tr>';
        for (var _i = 0, _a = sessions.items.sort(function (a, b) { return a.name > b.name ? 1 : -1; }); _i < _a.length; _i++) {
            var session = _a[_i];
            result += '<tr><td>' + session.id + '</td>' +
                '<td><b>' + session.name + '</b><div>' + session.version + '</div>' +
                '<div><b>Ip:</b>' + session.ip + '</div>' +
                '<div id="session-info-' + session.id + '">' + this.renderSessionData(session) + '</div>' +
                '</td>' +
                '<td id="session-topics-' + session.id + '">' + this.renderSessionTopics(session) + '</td>' +
                '<td id="session-queues-' + session.id + '">' + this.renderSessionQueues(session) + '</td></tr>';
        }
        return result + "</table>";
    };
    HtmlSessions.renderSessionData = function (session) {
        return '<div><b>Connected:</b>' + session.connected + '</div>' +
            '<div><b>Last incoming:</b>' + session.lastIncoming + '</div>' +
            '<div><b>Read:</b>' + Utils.format_bytes(session.readSize) + '</div>' +
            '<div><b>Written:</b>' + Utils.format_bytes(session.writtenSize) + '</div>' +
            '<div><b>Read/sec:</b>' + Utils.format_bytes(session.readPerSec) + '</div>' +
            '<div><b>Written/sec:</b>' + Utils.format_bytes(session.writtenPerSec) + '</div>';
    };
    HtmlSessions.renderSessionQueues = function (session) {
        var result = "";
        for (var _i = 0, _a = session.subscribers; _i < _a.length; _i++) {
            var subscriber = _a[_i];
            var badgeType = subscriber.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">' + subscriber.topicId + " -> " + subscriber.queueId + '</span> ';
        }
        return result;
    };
    HtmlSessions.renderSessionTopics = function (session) {
        var result = "";
        Utils.iterateSessionPublishers(session, function (topic, value) {
            var badgeType = value > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">' + topic + '</span> ';
        });
        return result;
    };
    HtmlSessions.updateSessionData = function (sessions) {
        for (var _i = 0, _a = sessions.items; _i < _a.length; _i++) {
            var session = _a[_i];
            var el = document.getElementById('session-info-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionData(session);
            }
            var el = document.getElementById('session-topics-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionTopics(session);
            }
            var el = document.getElementById('session-queues-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionQueues(session);
            }
        }
    };
    return HtmlSessions;
}());
//# sourceMappingURL=HtmlSessions.js.map