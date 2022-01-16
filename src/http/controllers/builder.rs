use std::sync::Arc;

use my_http_server::middlewares::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new();

    let topics_controller = Arc::new(super::topics::TopicsController::new(app.clone()));

    controllers.register_get_action("/Topics", topics_controller.clone());
    controllers.register_post_action("/Topics/Create", topics_controller);

    let sessions_controller = super::sessions::SessionsController::new(app.clone());

    controllers.register_delete_action("/Sessions", Arc::new(sessions_controller));

    let greeting_controller = Arc::new(super::greeting::GreetingController::new(app.clone()));
    controllers.register_post_action("/Greeting", greeting_controller);
    //controllers.register_http_objects(greeting_controller);

    let greeting_ping_controller = Arc::new(super::greeting::PingController::new(app.clone()));
    controllers.register_post_action("/Greeting/Ping", greeting_ping_controller);

    let status_controller = super::status::status_controller::StatusController::new(app.clone());
    controllers.register_get_action("/Status", Arc::new(status_controller));

    let queues_controller = Arc::new(super::queues::QueuesController::new(app.clone()));
    controllers.register_get_action("/Queues", queues_controller.clone());
    controllers.register_post_action("/Queues/SetMessageId", queues_controller.clone());
    controllers.register_delete_action("/Queues", queues_controller);

    let locks_controller = super::debug::LocksController::new(app.clone());
    controllers.register_get_action("/Locks", Arc::new(locks_controller));

    let debug_mode_controller = Arc::new(super::debug::DebugModeController::new(app.clone()));
    controllers.register_post_action("/Debug/Enable", debug_mode_controller.clone());
    controllers.register_delete_action("/Debug/Disable", debug_mode_controller.clone());

    let on_delivery_controller = Arc::new(super::debug::OnDeliveryController::new(app.clone()));
    controllers.register_get_action("/Debug/OnDelivery", on_delivery_controller);

    let logs_controller = Arc::new(super::logs::LogsController::new(app.clone()));
    controllers.register_get_action("/Logs", logs_controller);

    let logs_by_topic_controller = Arc::new(super::logs::LogsByTopicController::new(app.clone()));
    controllers.register_get_action("/Logs/Topic/{topicId}", logs_by_topic_controller);

    let logs_by_process_controller =
        Arc::new(super::logs::LogsByProcessController::new(app.clone()));
    controllers.register_get_action("/Logs/Process/{processId}", logs_by_process_controller);

    let publisher_controller = super::publisher::PublisherController::new(app.clone());
    controllers.register_post_action("/Publish", Arc::new(publisher_controller));

    let home_controller = super::home_controller::HomeController::new(app.clone());
    controllers.register_get_action("/", Arc::new(home_controller));

    controllers
}
