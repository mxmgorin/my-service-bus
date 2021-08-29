use futures_util::stream;
use my_service_bus_shared::page_id::PageId;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::date_time::MyDateTime;
use crate::message_pages::MessagesPage;
use crate::messages::{MySbMessage, MySbMessageContent};
use crate::settings::SettingsModel;
use crate::utils::{LazyObject, LazyObjectAccess};

use crate::persistence_grpc::my_service_bus_messages_persistence_grpc_service_client::MyServiceBusMessagesPersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::errors::{GrpcClientError, PersistenceError};
use super::protobuf_models::{
    MessageProtobufModel, MessagesProtobufModel, NewMessagesProtobufContract,
};
use super::zip::decompress_payload;
pub struct MessagesPagesRepo {
    grpc_address: String,
    grpc_client: LazyObject<MyServiceBusMessagesPersistenceGrpcServiceClient<Channel>>,
}

impl MessagesPagesRepo {
    pub fn new(settings: &SettingsModel) -> Self {
        Self {
            grpc_address: settings.persistence_grpc_url.to_string(),
            grpc_client: LazyObject::new(),
        }
    }

    async fn get_grpc_client<'s>(
        &'s self,
    ) -> Result<
        LazyObjectAccess<'s, MyServiceBusMessagesPersistenceGrpcServiceClient<Channel>>,
        GrpcClientError,
    > {
        if !self.grpc_client.has_instance().await {
            let grpc_addess = self.grpc_address.to_string();
            let result =
                MyServiceBusMessagesPersistenceGrpcServiceClient::connect(grpc_addess).await?;

            self.grpc_client.init(result).await;
        }

        let instance = self.grpc_client.get();
        return Ok(instance);
    }

    pub async fn load_page(
        &self,
        topic_id: &str,
        page_id: PageId,
    ) -> Result<MessagesPage, PersistenceError> {
        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client = grpc_client.as_mut();

        if grpc_client.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "messages_pages_repo::load_page".to_string(),
            ));
        }

        let grpc_client = grpc_client.unwrap();

        let grpc_stream = grpc_client
            .get_page_compressed(GetMessagesPageGrpcRequest {
                topic_id: topic_id.to_string(),
                page_no: page_id,
            })
            .await?;

        let mut grpc_stream = grpc_stream.into_inner();

        let mut buffer: Vec<u8> = Vec::new();

        while let Some(stream_result) = grpc_stream.next().await {
            let stream_result = stream_result?;
            buffer.extend(stream_result.chunk);
        }

        let unzipped = decompress_payload(buffer.as_slice())?;

        let grpc_model: MessagesProtobufModel = prost::Message::decode(unzipped.as_slice())?;

        let page = MessagesPage::new(page_id);

        let mut msgs = Vec::new();

        for message in grpc_model.messages {
            let time_parse_result = MyDateTime::from_optional_bcl(message.created);

            match time_parse_result {
                Ok(time) => {
                    let msg = MySbMessage::Loaded(MySbMessageContent::new(
                        message.message_id,
                        message.data,
                        time,
                    ));
                    msgs.push(msg);
                }
                Err(err) => {
                    let msg = MySbMessage::CanNotBeLoaded {
                        id: message.message_id,
                        err: format!("{:?}", err),
                    };
                    msgs.push(msg);
                }
            }
        }

        page.restore(msgs).await;

        Ok(page)
    }

    pub async fn save_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
        payload_size: usize,
    ) -> Result<(), PersistenceError> {
        let grpc_messages = NewMessagesProtobufContract {
            topic_id: topic_id.to_string(),
            messages,
        };

        let grpc_protobuf = grpc_messages.into_protobuf_vec();

        let grpc_protobuf_compressed = super::zip::compress_payload(grpc_protobuf.as_slice())?;

        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client_result = grpc_client.as_mut();

        if grpc_client_result.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "messages_pages_repo::load_page".to_string(),
            ));
        }

        let grpc_client = grpc_client_result.unwrap();

        let mut grpc_chunks = Vec::new();

        for chunk in split(grpc_protobuf_compressed.as_slice(), payload_size) {
            grpc_chunks.push(CompressedMessageChunkModel { chunk });
        }

        let result = grpc_client.save_messages(stream::iter(grpc_chunks)).await;

        if let Err(status) = result {
            return Err(PersistenceError::TonicError(status));
        }

        return Ok(());
    }
}

fn split(src: &[u8], max_payload_size: usize) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();

    let mut pos: usize = 0;

    while pos < src.len() {
        if src.len() - pos < max_payload_size {
            let payload = &src[pos..];
            result.push(payload.to_vec());
            break;
        }
        let payload = &src[pos..pos + max_payload_size];
        result.push(payload.to_vec());
        pos += max_payload_size;
    }

    result
}