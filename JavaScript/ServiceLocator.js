var ServiceLocator = /** @class */ (function () {
    function ServiceLocator() {
    }
    ServiceLocator.checkIfTopicsAreChanged = function (topics) {
        if (!this.topics)
            return true;
        return this.topics.snapshotId != topics.snapshotId;
    };
    ServiceLocator.checkIfSessionsAreChanged = function (sessions) {
        if (!this.sessions)
            return true;
        return this.sessions.snapshotId != sessions.snapshotId;
    };
    ServiceLocator.checkIfFilterPhraseIsChanged = function (filterPhrase) {
        if (this.prevFilterPhrase == undefined) {
            ServiceLocator.prevFilterPhrase = filterPhrase;
            return true;
        }
        if (filterPhrase != ServiceLocator.prevFilterPhrase) {
            ServiceLocator.prevFilterPhrase = filterPhrase;
            return true;
        }
        return false;
    };
    return ServiceLocator;
}());
//# sourceMappingURL=ServiceLocator.js.map