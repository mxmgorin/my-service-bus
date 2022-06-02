use std::collections::HashMap;
use std::time::Duration;

use futures_util::stream;

use my_service_bus_shared::page_id::PageId;
use my_service_bus_shared::protobuf_models::MessageProtobufModel;
use my_service_bus_shared::{MessageId, MySbMessageContent};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::settings::SettingsModel;
use crate::utils::{LazyObject, LazyObjectAccess};

use crate::persistence_grpc::my_service_bus_messages_persistence_grpc_service_client::MyServiceBusMessagesPersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::protobuf_models::NewMessagesProtobufContract;
use super::{MessagesPagesRepo, PersistenceError};
use async_trait::async_trait;
pub struct MessagesPagesGrpcRepo {
    grpc_address: String,
    grpc_client: LazyObject<MyServiceBusMessagesPersistenceGrpcServiceClient<Channel>>,
    time_out: Duration,
}

impl MessagesPagesGrpcRepo {
    pub fn new(settings: &SettingsModel) -> Self {
        Self {
            grpc_address: settings.persistence_grpc_url.to_string(),
            grpc_client: LazyObject::new(),
            time_out: settings.grpc_timeout,
        }
    }

    async fn get_grpc_client<'s>(
        &'s self,
    ) -> Result<
        LazyObjectAccess<'s, MyServiceBusMessagesPersistenceGrpcServiceClient<Channel>>,
        PersistenceError,
    > {
        if !self.grpc_client.has_instance().await {
            let instance = init_grpc_client(&self.grpc_address, self.time_out).await?;
            self.grpc_client.init(instance).await;
        }

        let instance = self.grpc_client.get();
        return Ok(instance);
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

        let grpc_protobuf_compressed =
            my_service_bus_shared::page_compressor::zip::compress_payload(
                grpc_protobuf.as_slice(),
            )?;

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

        let result = tokio::time::timeout(
            self.time_out,
            grpc_client.save_messages(stream::iter(grpc_chunks)),
        )
        .await?;

        if let Err(status) = result {
            return Err(PersistenceError::TonicError(status));
        }

        return Ok(());
    }
}

#[async_trait]
impl MessagesPagesRepo for MessagesPagesGrpcRepo {
    async fn load_page(
        &self,
        topic_id: &str,
        page_id: PageId,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<Option<HashMap<MessageId, MySbMessageContent>>, PersistenceError> {
        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client = grpc_client.as_mut();

        if grpc_client.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "messages_pages_repo::load_page".to_string(),
            ));
        }

        let grpc_client = grpc_client.unwrap();

        let mut grpc_stream = tokio::time::timeout(
            self.time_out,
            grpc_client.get_page(GetMessagesPageGrpcRequest {
                topic_id: topic_id.to_string(),
                page_no: page_id,
                from_message_id,
                to_message_id,
                version: 1,
            }),
        )
        .await??
        .into_inner();

        let mut messages: HashMap<MessageId, MySbMessageContent> = HashMap::new();

        while let Some(stream_result) =
            tokio::time::timeout(self.time_out, grpc_stream.next()).await?
        {
            let grpc_model = stream_result?;
            messages.insert(
                grpc_model.message_id,
                MySbMessageContent {
                    id: grpc_model.message_id,
                    content: grpc_model.data,
                    time: DateTimeAsMicroseconds::new(grpc_model.created),
                    headers: None, //TODO - restore it
                },
            );
        }

        println!(
            "Read Page{} with messages amount: {}",
            page_id,
            messages.len()
        );

        Ok(Some(messages))
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

async fn init_grpc_client(
    grpc_address: &str,
    time_out: Duration,
) -> Result<MyServiceBusMessagesPersistenceGrpcServiceClient<Channel>, PersistenceError> {
    let mut attempt_no = 0;

    loop {
        let init_result = tokio::time::timeout(
            time_out,
            MyServiceBusMessagesPersistenceGrpcServiceClient::connect(grpc_address.to_string()),
        )
        .await;

        if let Ok(init_result) = init_result {
            match init_result {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    println!("Can no initilize Messages Pages GRPC Repo. Err:{:?}", err);
                }
            }
        } else {
            println!("Initializing Messages Pages GRPC Repo timeout");
        }

        if attempt_no >= 5 {
            return Err(PersistenceError::CanNotInitializeGrpcService);
        }

        attempt_no += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
