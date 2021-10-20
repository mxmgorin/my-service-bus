class ServiceLocator {
    public static topics: ITopics;
    public static sessions: ISessions;
    public static prevFilterPhrase: string;


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
}