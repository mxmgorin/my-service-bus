use my_service_bus_shared::{
    page_id::PageId, protobuf_models::MessageProtobufModel, MessageId, MySbMessageContent,
};

use crate::settings::SettingsModel;

#[cfg(test)]
use super::MessagesPagesMockRepo;
use super::{MessagesPagesGrpcRepo, PersistenceError};

pub enum MessagesPagesRepo {
    Grpc(MessagesPagesGrpcRepo),
    #[cfg(test)]
    Mock(MessagesPagesMockRepo),
}

impl MessagesPagesRepo {
    pub fn create_production_instance(settings: &SettingsModel) -> Self {
        Self::Grpc(MessagesPagesGrpcRepo::new(settings))
    }

    #[cfg(test)]
    pub fn create_mock_instance() -> Self {
        Self::Mock(MessagesPagesMockRepo::new())
    }

    pub async fn load_page(
        &self,
        topic_id: &str,
        page_id: PageId,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<Option<Vec<MySbMessageContent>>, PersistenceError> {
        match self {
            MessagesPagesRepo::Grpc(repo) => {
                repo.load_page(topic_id, page_id, from_message_id, to_message_id)
                    .await
            }
            #[cfg(test)]
            MessagesPagesRepo::Mock(repo) => {
                repo.load_page(topic_id, from_message_id, to_message_id)
                    .await
            }
        }
    }

    pub async fn save_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        match self {
            MessagesPagesRepo::Grpc(repo) => repo.save_messages(topic_id, messages).await,
            #[cfg(test)]
            MessagesPagesRepo::Mock(repo) => repo.save_messages(topic_id, messages).await,
        }
    }
}
