use std::sync::Arc;

use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new();

    let topics_controller = Arc::new(super::topics::TopicsController::new(app.clone()));

    controllers.register_get_action(topics_controller.clone());
    controllers.register_post_action(topics_controller);

    controllers.register_delete_action(Arc::new(super::sessions::DeleteSessionAction::new(
        app.clone(),
    )));

    controllers.register_post_action(Arc::new(super::greeting::GreetingAction::new(app.clone())));
    //controllers.register_http_objects(greeting_controller);

    controllers.register_post_action(Arc::new(super::greeting::PingAction::new(app.clone())));

    let status_controller = super::status::status_controller::StatusController::new(app.clone());
    controllers.register_get_action(Arc::new(status_controller));

    let queues_controller = Arc::new(super::queues::QueuesController::new(app.clone()));
    controllers.register_get_action(queues_controller.clone());
    controllers.register_post_action(queues_controller.clone());
    controllers.register_delete_action(queues_controller);

    let locks_controller = super::debug::LocksController::new(app.clone());
    controllers.register_get_action(Arc::new(locks_controller));

    let debug_mode_controller = Arc::new(super::debug::DebugModeController::new(app.clone()));
    controllers.register_post_action(debug_mode_controller.clone());
    controllers.register_delete_action(debug_mode_controller.clone());

    let on_delivery_controller = Arc::new(super::debug::OnDeliveryAction::new(app.clone()));
    controllers.register_get_action(on_delivery_controller);

    let logs_controller = Arc::new(super::logs::LogsController::new(app.clone()));
    controllers.register_get_action(logs_controller);

    let logs_by_topic_controller = Arc::new(super::logs::LogsByTopicController::new(app.clone()));
    controllers.register_get_action(logs_by_topic_controller);

    let logs_by_process_controller =
        Arc::new(super::logs::LogsByProcessController::new(app.clone()));
    controllers.register_get_action(logs_by_process_controller);

    let publisher_controller = super::publisher::PublisherController::new(app.clone());
    controllers.register_post_action(Arc::new(publisher_controller));

    controllers.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app,
    )));

    controllers
}
