use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::app::AppContext;

pub struct TimerDescription {
    pub task: JoinHandle<()>,
    pub description: String,
}

impl TimerDescription {
    pub fn new(description: &str, task: JoinHandle<()>) -> Self {
        println!("{} is started", description);
        Self {
            description: description.to_string(),
            task,
        }
    }
}

pub async fn start(app: Arc<AppContext>) {
    let mut tasks = Vec::new();

    if let Some(delivery_timeout) = app.delivery_timeout {
        let task = tokio::task::spawn(super::dead_subscribers_kicker::start(
            app.clone(),
            delivery_timeout,
        ));
        tasks.push(TimerDescription::new("Delivery timer", task));
    }

    let task = tokio::task::spawn(super::persist::start(app.clone()));
    tasks.push(TimerDescription::new("Persist timer", task));

    let task = tokio::task::spawn(super::metrics_timer::start(app.clone()));
    tasks.push(TimerDescription::new("Metrics timer", task));

    let task = tokio::task::spawn(super::gc::start(app.clone()));
    tasks.push(TimerDescription::new("GC timer", task));

    for timer_description in tasks {
        let result = timer_description.task.await;
        if let Err(err) = result {
            println!(
                "Error with timer: {}. {}",
                timer_description.description, err
            )
        }
    }
}
