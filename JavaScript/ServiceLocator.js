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
    return ServiceLocator;
}());
//# sourceMappingURL=ServiceLocator.js.map