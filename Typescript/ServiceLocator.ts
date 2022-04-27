class ServiceLocator {
    public static topics: ITopics;
    public static sessions: ISessions;
    static prevFilterPhrase: string;


    public static checkIfTopicsAreChanged(topics: ITopics): boolean {

        if (!this.topics)
            return true;

        return this.topics.snapshotId != topics.snapshotId;

    }

    public static checkIfSessionsAreChanged(sessions: ISessions): boolean {

        if (!this.sessions)
            return true;

        return this.sessions.snapshotId != sessions.snapshotId;

    }


    public static checkIfFilterPhraseIsChanged(filterPhrase: string): boolean {

        if (this.prevFilterPhrase == undefined) {
            ServiceLocator.prevFilterPhrase = filterPhrase;
            return true;
        }

        if (filterPhrase != ServiceLocator.prevFilterPhrase) {
            ServiceLocator.prevFilterPhrase = filterPhrase;
            return true;
        }

        return false;
    }

}