use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::date_time::MyDateTime;

#[derive(Debug, Clone, Copy)]
pub enum SystemProcess {
    System = 0,
    TcpSocket = 1,
    TopicOperation = 2,
    QueueOperation = 3,
    Init = 4,
    Timer = 5,
    Persistence = 6,
}

impl SystemProcess {
    pub fn iterate() -> Vec<Self> {
        let mut result = Vec::new();

        result.push(SystemProcess::System);
        result.push(SystemProcess::TcpSocket);
        result.push(SystemProcess::TopicOperation);
        result.push(SystemProcess::QueueOperation);
        result.push(SystemProcess::Init);
        result.push(SystemProcess::Timer);
        result.push(SystemProcess::Persistence);

        return result;
    }
    pub fn parse(value: &str) -> Option<Self> {
        if value == "system" {
            return Some(SystemProcess::System);
        }

        if value == "tcpsocket" {
            return Some(SystemProcess::TcpSocket);
        }

        if value == "topicpperation" {
            return Some(SystemProcess::TopicOperation);
        }

        if value == "queueoperation" {
            return Some(SystemProcess::QueueOperation);
        }

        if value == "init" {
            return Some(SystemProcess::Init);
        }

        if value == "timer" {
            return Some(SystemProcess::Timer);
        }

        if value == "persistence" {
            return Some(SystemProcess::Persistence);
        }

        return None;
    }

    pub fn as_u8(&self) -> u8 {
        let result = *self as u8;
        return result;
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
}
#[derive(Debug, Clone)]
pub struct LogItem {
    pub date: MyDateTime,

    pub topic: Option<String>,

    pub level: LogLevel,

    pub process: SystemProcess,

    pub process_name: String,

    pub message: String,

    pub err_ctx: Option<String>,
}

struct LogsData {
    items: Vec<Arc<LogItem>>,
    items_by_topic: HashMap<String, Vec<Arc<LogItem>>>,
    items_by_process: HashMap<u8, Vec<Arc<LogItem>>>,
}

pub struct Logs {
    data: RwLock<LogsData>,
}

impl Logs {
    pub fn new() -> Self {
        let logs_data = LogsData {
            items: Vec::new(),
            items_by_topic: HashMap::new(),
            items_by_process: HashMap::new(),
        };

        Self {
            data: RwLock::new(logs_data),
        }
    }

    async fn add(&self, item: LogItem) {
        let item = Arc::new(item);
        let mut wirte_access = self.data.write().await;

        let process_id = item.as_ref().process.as_u8();

        add_topc_data(
            &mut wirte_access.items_by_process,
            &process_id,
            item.clone(),
        );

        if let Some(topic_name) = &item.topic {
            add_topc_data(&mut wirte_access.items_by_topic, topic_name, item.clone());
        }

        let items = &mut wirte_access.items;
        items.push(item);
        gc_logs(items);
    }

    pub async fn add_info(
        &self,
        topic: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
    ) {
        let topic_str = format!("{:?}", &topic);

        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Info,
            topic,
            process_name,
            process,
            message: message,
            err_ctx: None,
        };

        println!(
            "{dt} {level:?} Process:{proces:?}\n Topic:{topic}\n Process:{processname}\n Message:{message}",
            dt = item.date.to_iso_string(),
            level = item.level,
            proces = item.process,
            processname = item.process_name,
            message = item.message,
            topic = topic_str
        );

        println!("-------------");

        self.add(item).await;
    }

    pub async fn add_error(
        &self,
        topic: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        err_ctx: Option<String>,
    ) {
        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Error,
            topic,
            process_name,
            process,
            message: message,
            err_ctx,
        };

        println!(
            "{dt} {level:?} {proces:?}\n Process:{processname}\n Message:{message}",
            dt = item.date.to_iso_string(),
            level = item.level,
            proces = item.process,
            processname = item.process_name,
            message = item.message
        );
        println!("-------------");
        self.add(item).await;
    }

    pub async fn get(&self) -> Vec<Arc<LogItem>> {
        let read_access = self.data.read().await;
        read_access.items.to_vec()
    }

    pub async fn get_by_topic(&self, topic: &str) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_topic.get(topic)?;
        return Some(result.to_vec());
    }

    pub async fn get_by_process(&self, process: SystemProcess) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_process.get(&process.as_u8())?;
        return Some(result.to_vec());
    }
}

fn add_topc_data<T>(
    items_by_topic: &mut HashMap<T, Vec<Arc<LogItem>>>,
    category: &T,
    item: Arc<LogItem>,
) where
    T: Eq + std::hash::Hash + Clone + Sized,
{
    if !items_by_topic.contains_key(category) {
        items_by_topic.insert(category.clone(), Vec::new());
    }

    let items = items_by_topic.get_mut(category);

    if let Some(items) = items {
        items.push(item);
        gc_logs(items);
    }
}

fn gc_logs(items: &mut Vec<Arc<LogItem>>) {
    while items.len() > 100 {
        items.remove(0);
    }
}
