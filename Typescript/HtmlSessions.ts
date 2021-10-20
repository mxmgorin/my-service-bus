class HtmlSessions {

    public static renderSessions(sessions: ISessions): string {
        let result = '<table style="font-size:12px" class="table table-striped table-dark">' +
            '<tr><th style="width:50px">Id</th><th style="width:120px">Info</th><th>Publisher</th><th>Subscriber</th></tr>';


        for (let session of sessions.items.sort((a, b) => a.name > b.name ? 1 : -1)) {
            result += '<tr class="filter-line"><td>' + session.id + '</td>' +
                '<td><b>' + session.name + '</b><div>' + session.version + '</div>' +
                '<div><b>Ip:</b>' + session.ip + '</div>' +

                '<div id="session-info-' + session.id + '">' + this.renderSessionData(session) + '</div>' +

                '</td>' +

                '<td id="session-topics-' + session.id + '">' + this.renderSessionTopics(session) + '</td>' +
                '<td id="session-queues-' + session.id + '">' + this.renderSessionQueues(session) + '</td></tr>';
        }

        return result + "</table>";
    }


    private static renderSessionData(session: ISession): string {
        return '<div><b>Connected:</b>' + session.connected + '</div>' +
            '<div><b>Last incoming:</b>' + session.lastIncoming + '</div>' +
            '<div><b>Read:</b>' + Utils.format_bytes(session.readSize) + '</div>' +
            '<div><b>Written:</b>' + Utils.format_bytes(session.writtenSize) + '</div>' +
            '<div><b>Read/sec:</b>' + Utils.format_bytes(session.readPerSec) + '</div>' +
            '<div><b>Written/sec:</b>' + Utils.format_bytes(session.writtenPerSec) + '</div>';
    }



    private static renderSessionQueues(session: ISession): string {

        let result = "";
        for (let subscriber of session.subscribers) {
            let badgeType = subscriber.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">[' + subscriber.id + ']' + subscriber.topicId + " -> " + subscriber.queueId + '</span> ';
        }

        return result;
    }


    private static renderSessionTopics(session: ISession): string {

        let result = "";

        Utils.iterateSessionPublishers(session, (topic, value) => {
            let badgeType = value > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">' + topic + '</span> ';

        });

        return result;
    }

    public static updateSessionData(sessions: ISessions) {
        for (let session of sessions.items) {
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
    }

}