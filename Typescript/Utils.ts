interface IQueueSubscriber {
    session: ISession, subscriber: ISubscriber
}

interface ITopicSession {
    session: ISession,
    active: boolean
}

class Utils {


    public static filterIt(line: string, filterPhrase: string): boolean {
        if (filterPhrase == "")
            return false;

        return line.indexOf(filterPhrase) == -1;
    }

    public static copyToClipboardHtml(text: string): string {
        return ' style="cursor:pointer" clipboard=' + text + ' onclick="Utils.copyToClipboard()"';
    }

    public static copyToClipboard(el: HTMLElement) {
        let attr = el.attributes.getNamedItem('clipboard');
        if (attr) {
            navigator.clipboard.writeText(attr.value);
        }

    }

    public static getMax(c: number[]): number {
        let result = 0;

        for (const i of c) {
            if (i > result) result = i;
        }

        return result;
    }

    public static formatNumber(n: number): string {

        return n.toString().replace(/(\d)(?=(\d{3})+(?!\d))/g, '$1,');
    }


    public static format_bytes(n: number): string {
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
    }


    public static iterateTopicQueues(status: IStatus, callback: (topic: string, queues: ITopicQueues) => void) {
        let topics = Object.keys(status.queues)
        for (let topic of topics) {
            callback(topic, status.queues[topic]);
        }
    }

    public static iterateSessionPublishers(session: ISession, data: (topic: string, value: number) => void) {
        let topics = Object.keys(session.publishers)
        for (let topic of topics) {
            data(topic, session.publishers[topic]);
        }
    }

    public static getQueueSubscribers(status: IStatus, topicId: string, queueId: string): IQueueSubscriber[] {
        let result: IQueueSubscriber[] = [];
        for (let session of status.sessions.items) {
            for (let subscriber of session.subscribers) {
                if (subscriber.topicId == topicId && subscriber.queueId == queueId) {
                    result.push({ session, subscriber });
                }
            }
        }

        return result;
    }


    public static format_duration(micros: number): string {

        if (micros == 0)
            return "0";

        if (micros < 1000) {
            return micros + "µs";
        }

        if (micros < 1000000) {
            return (micros / 1000).toFixed(3) + "ms";
        }


        return (micros / 1000000).toFixed(3) + "s"


    }

    public static iterateBySessionsWithTopic(status: IStatus, topic: string): ITopicSession[] {

        let result: ITopicSession[] = [];
        for (let session of status.sessions.items) {

            Utils.iterateSessionPublishers(session, (topicFromSession, active) => {

                if (topicFromSession == topic) {
                    result.push({
                        session, active: active > 0
                    });
                }

            });

        }


        return result;
    }
}