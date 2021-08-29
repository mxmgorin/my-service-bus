use std::time::Duration;

use my_service_bus_shared::page_id::PageId;

use crate::{
    app::AppContext, message_pages::MessagesPage, persistence::PersistenceError, topics::Topic,
    utils::StopWatch,
};

pub async fn restore_page(app: &AppContext, topic: &Topic, page_id: PageId) {
    let page = load_page(app, topic, page_id).await;
    topic.messages.restore_page(page).await;
}

async fn load_page(app: &AppContext, topic: &Topic, page_id: PageId) -> MessagesPage {
    let mut sw = StopWatch::new();

    let mut attempt_no = 1;

    loop {
        sw.start();
        app.logs
            .add_info(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Init,
                format!("Restoring topic page {}. Attempt: {}", page_id, attempt_no),
                "Begin restoring".to_string(),
            )
            .await;

        let load_result = app
            .messages_pages_repo
            .load_page(topic.topic_id.as_str(), page_id)
            .await;

        sw.pause();

        if let Ok(page) = load_result {
            app.logs
                .add_info(
                    Some(topic.topic_id.to_string()),
                    crate::app::logs::SystemProcess::Init,
                    format!("Restoring topic page {}", page_id),
                    format!("Restored in {:?}", sw.duration_as_string()),
                )
                .await;
            return page;
        }

        let err = load_result.err().unwrap();

        match &err {
            PersistenceError::ZipOperationError(msg) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "Can not unzip payload. Creating empty page...",
                    msg.to_string(),
                )
                .await;

                let page = MessagesPage::new(page_id);
                return page;
            }
            PersistenceError::TonicError(status) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "Some HTTP Level Error. Delayin and retrying",
                    format!("{:?}", status),
                )
                .await;
            }
            PersistenceError::InvalidProtobufPayload(msg) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "Can not deserialize payload from Protobuf. Creating empty page...",
                    msg.to_string(),
                )
                .await;

                let page = MessagesPage::new(page_id);
                return page;
            }
            PersistenceError::GrpcClientError(_) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "PersistenceError Grpc error",
                    format!("{:?}", err),
                )
                .await;
            }

            PersistenceError::GrpcClientIsNotInitialized(_) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "PersistenceError::GrpcClientIsNotInitialized",
                    format!("{:?}", err),
                )
                .await
            }
        }

        let duration = Duration::from_secs(1);
        tokio::time::sleep(duration).await;
        attempt_no += 1;
    }
}

async fn log_error(app: &AppContext, topic: &Topic, page_id: PageId, message: &str, err: String) {
    app.logs
        .add_error(
            Some(topic.topic_id.to_string()),
            crate::app::logs::SystemProcess::Persistence,
            format!("Restoring topic page {}", page_id),
            message.to_string(),
            Some(err),
        )
        .await;
}