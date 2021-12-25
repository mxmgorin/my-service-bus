use std::collections::HashMap;

use async_trait::async_trait;
use my_service_bus_shared::{page_id::PageId, MessageId, MySbMessageContent};

use super::PersistenceError;
#[async_trait]
pub trait MessagesPagesRepo {
    async fn load_page(
        &self,
        topic_id: &str,
        page_id: PageId,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<Option<HashMap<MessageId, MySbMessageContent>>, PersistenceError>;
}
