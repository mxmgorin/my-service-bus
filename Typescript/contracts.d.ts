
interface IStatus {
    topics: ITopics,
    queues: object,
    sessions: ISessions
    system: ISystemStatus,
    subscribers: ISubscriber[]
}
interface ISession {
    id: number,
    ip: string,
    version: string,
    name: string,
    connected: string,
    lastIncoming: string,
    readSize: number,
    writtenSize: number,
    readPerSec: number,
    writtenPerSec: number,
}

interface ISessions {
    snapshotId: number,
    items: ISession[]
}

interface ITopics {
    snapshotId: number,
    items: ITopic[],
}

interface ITopicPublisher {
    sessionId: number,
    active: number
}

interface ITopic {
    id: string,
    messageId: number,
    packetPerSec: number,
    messagesPerSec: number,
    persistSize: number,
    publishHistory: number[],
    pages: IPage[],
    publishers: ITopicPublisher[]
}

interface IPage {
    id: number,
    percent: number,
    size: number
}


interface ITopicQueues {
    snapshotId: number,
    queues: ITopicQueue[]
}

interface ITopicQueue {
    id: string,
    queueType: number,
    size: number,
    data: IQueueIndexRange[]
}



interface ISubscriber {
    id: number,
    sessionId: number;
    topicId: string,
    queueId: string,
    active: number,
    deliveryMode: number,
    deliveryHistory: number[],
}



interface IQueueIndexRange {
    fromId: number,
    toId: number
}

interface ISystemStatus {
    usedmem: number,
    totalmem: number
}