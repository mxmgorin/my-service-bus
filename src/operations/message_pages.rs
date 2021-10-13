use std::time::Duration;

use my_service_bus_shared::{messages_page::MessagesPage, page_id::PageId};

use crate::{app::AppContext, persistence::PersistenceError, topics::Topic, utils::StopWatch};

pub async fn restore_page(app: &AppContext, topic: &Topic, page_id: PageId, process: &str) {
    println!(
        "Restoring page {}/{} as a part of the process {}",
        topic.topic_id, page_id, process
    );
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
                    format!(
                        "Can not unzip payload. Attempt {}. Creating empty page...",
                        attempt_no
                    ),
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
                    "Some HTTP Level Error. Delayin and retrying".to_string(),
                    format!("{:?}", status),
                )
                .await;
            }
            PersistenceError::InvalidProtobufPayload(msg) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "Can not deserialize payload from Protobuf. Creating empty page...".to_string(),
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
                    "PersistenceError Grpc error".to_string(),
                    format!("{:?}", err),
                )
                .await;
            }

            PersistenceError::GrpcClientIsNotInitialized(_) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    "PersistenceError::GrpcClientIsNotInitialized".to_string(),
                    format!("{:?}", err),
                )
                .await
            }
            PersistenceError::CompressedPageReaderError(err) => {
                log_error(
                    app,
                    topic,
                    page_id,
                    format!("PersistenceError::CompressedPageReaderError. Attmopt{} Creating empty page", attempt_no),
                    format!("{:?}", err),
                )
                .await;

                let page = MessagesPage::new(page_id);
                return page;
            }
        }

        let duration = Duration::from_secs(1);
        tokio::time::sleep(duration).await;
        attempt_no += 1;
    }
}

async fn log_error(app: &AppContext, topic: &Topic, page_id: PageId, message: String, err: String) {
    app.logs
        .add_error(
            Some(topic.topic_id.to_string()),
            crate::app::logs::SystemProcess::Persistence,
            format!("Restoring topic page {}", page_id),
            message,
            Some(err),
        )
        .await;
}
