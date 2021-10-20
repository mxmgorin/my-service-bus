var Utils = /** @class */ (function () {
    function Utils() {
    }
    Utils.filterIt = function (line, filterPhrase) {
        if (filterPhrase == "")
            return false;
        return line.indexOf(filterPhrase) == -1;
    };
    Utils.copyToClipboardHtml = function (text) {
        return ' style="cursor:pointer" clipboard=' + text + ' onclick="Utils.copyToClipboard(this)"';
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
    Utils.iterateTopicQueues = function (status, callback) {
        var topics = Object.keys(status.queues);
        for (var _i = 0, topics_1 = topics; _i < topics_1.length; _i++) {
            var topic = topics_1[_i];
            callback(topic, status.queues[topic]);
        }
    };
    Utils.iterateSessionPublishers = function (session, data) {
        var topics = Object.keys(session.publishers);
        for (var _i = 0, topics_2 = topics; _i < topics_2.length; _i++) {
            var topic = topics_2[_i];
            data(topic, session.publishers[topic]);
        }
    };
    Utils.getQueueSubscribers = function (status, topicId, queueId) {
        var result = [];
        for (var _i = 0, _a = status.sessions.items; _i < _a.length; _i++) {
            var session = _a[_i];
            for (var _b = 0, _c = session.subscribers; _b < _c.length; _b++) {
                var subscriber = _c[_b];
                if (subscriber.topicId == topicId && subscriber.queueId == queueId) {
                    result.push({ session: session, subscriber: subscriber });
                }
            }
        }
        return result;
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
    Utils.iterateBySessionsWithTopic = function (status, topic) {
        var result = [];
        var _loop_1 = function (session) {
            Utils.iterateSessionPublishers(session, function (topicFromSession, active) {
                if (topicFromSession == topic) {
                    result.push({
                        session: session,
                        active: active > 0
                    });
                }
            });
        };
        for (var _i = 0, _a = status.sessions.items; _i < _a.length; _i++) {
            var session = _a[_i];
            _loop_1(session);
        }
        return result;
    };
    return Utils;
}());
//# sourceMappingURL=Utils.js.map