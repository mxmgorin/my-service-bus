use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::utils::{LazyObject, LazyObjectAccess};
use crate::{settings::SettingsModel, topics::TopicSnapshot};

use crate::persistence_grpc::my_service_bus_queue_persistence_grpc_service_client::MyServiceBusQueuePersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::errors::GrpcClientError;
use super::PersistenceError;

pub struct TopcsAndQueuesSnapshotRepo {
    grpc_address: String,
    grpc_client: LazyObject<MyServiceBusQueuePersistenceGrpcServiceClient<Channel>>,
}

impl TopcsAndQueuesSnapshotRepo {
    pub fn new(settings: &SettingsModel) -> TopcsAndQueuesSnapshotRepo {
        TopcsAndQueuesSnapshotRepo {
            grpc_address: settings.persistence_grpc_url.to_string(),
            grpc_client: LazyObject::new(),
        }
    }

    async fn get_grpc_client<'s>(
        &'s self,
    ) -> Result<
        LazyObjectAccess<'s, MyServiceBusQueuePersistenceGrpcServiceClient<Channel>>,
        GrpcClientError,
    > {
        if !self.grpc_client.has_instance().await {
            let grpc_addess = self.grpc_address.to_string();
            let result =
                MyServiceBusQueuePersistenceGrpcServiceClient::connect(grpc_addess).await?;
            self.grpc_client.init(result).await;
        }

        let result = self.grpc_client.get();
        return Ok(result);
    }

    pub async fn save(&self, snapshot: Vec<TopicSnapshot>) -> Result<(), PersistenceError> {
        let grpc_request: SaveQueueSnapshotGrpcRequest = snapshot.into();

        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client = grpc_client.as_mut();

        if grpc_client.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "queue_snapshot_repo::save".to_string(),
            ));
        }

        grpc_client.unwrap().save_snapshot(grpc_request).await?;

        Ok(())
    }

    pub async fn load(&self) -> Result<Vec<TopicSnapshot>, PersistenceError> {
        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client = grpc_client.as_mut();

        if grpc_client.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "queue_snapshot_repo::load".to_string(),
            ));
        }

        let grpc_client = grpc_client.unwrap();

        let mut grpc_response = grpc_client.get_snapshot(()).await?.into_inner();

        let mut result: Vec<TopicSnapshot> = Vec::new();

        while let Some(item) = grpc_response.next().await {
            let grpc_model = item?;
            result.push(grpc_model.into());
        }

        Ok(result)
    }
}
