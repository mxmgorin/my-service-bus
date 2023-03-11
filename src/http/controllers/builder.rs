use std::sync::Arc;

use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new(None, None);

    // topics
    controllers.register_get_action(Arc::new(super::topics::get_topics_action::GetTopicsAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::topics::create_topic_action::CreateTopicAction::new(app.clone())));

    // sessions
    controllers.register_delete_action(Arc::new(super::sessions::DeleteSessionAction::new(
        app.clone(),
    )));

    // greeting
    controllers.register_post_action(Arc::new(super::greeting::GreetingAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::greeting::PingAction::new(app.clone())));

    // status
    controllers.register_get_action(Arc::new(super::status::get_status_action::GetStatusAction::new(app.clone())));

    // queues
    controllers.register_get_action(Arc::new(super::queues::GetQueuesAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::queues::SetMessageIdAction::new(
        app.clone(),
    )));
    controllers
        .register_delete_action(Arc::new(super::queues::DeleteQueueAction::new(app.clone())));

    // debug
    controllers.register_post_action(Arc::new(super::debug::EnableDebugModeAction::new(app.clone())));
    controllers.register_delete_action(Arc::new(super::debug::DisableDebugModeAction::new(app.clone())));

    // delivery
    controllers.register_get_action(Arc::new(super::debug::OnDeliveryAction::new(app.clone())));

    // logs
    controllers.register_get_action(Arc::new(super::logs::LogsController::new(app.clone())));
    controllers.register_get_action(Arc::new(super::logs::LogsByTopicController::new(app.clone())));
    controllers.register_get_action(Arc::new(super::logs::GetLogsByProcessAction::new(app.clone())));

    // publish
    controllers.register_post_action(Arc::new(super::publisher::PublishAction::new(app.clone())));

    // home
    controllers.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    // prometheus
    controllers.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app,
    )));

    controllers
}
