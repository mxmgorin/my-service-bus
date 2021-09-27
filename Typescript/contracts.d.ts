
interface IStatus {
    topics: ITopics,
    queues: object,
    sessions: ISessions
    system: ISystemStatus
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
    publishers: object,
    subscribers: ISubscriber[]
}

interface ISessions {
    snapshotId: number,
    items: ISession[]
}

interface ITopics {
    snapshotId: number,
    items: ITopic[],
}


interface ITopic {
    id: string,
    messageId: number,
    packetPerSec: number,
    messagesPerSec: number,
    persistSize: number,
    publishHistory: number[]
    pages: IPage[]
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
    topicId: string,
    queueId: string,
    active: number,
    deliveryHistory: number[]
}



interface IQueueIndexRange {
    fromId: number,
    toId: number
}

interface ISystemStatus {
    usedmem: number,
    totalmem: number
}