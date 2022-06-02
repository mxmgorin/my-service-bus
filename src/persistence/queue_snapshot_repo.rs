use std::time::Duration;

use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::utils::{LazyObject, LazyObjectAccess};
use crate::{settings::SettingsModel, topics::TopicSnapshot};

use crate::persistence_grpc::my_service_bus_queue_persistence_grpc_service_client::MyServiceBusQueuePersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::PersistenceError;

pub struct TopcsAndQueuesSnapshotRepo {
    grpc_address: String,
    grpc_client: LazyObject<MyServiceBusQueuePersistenceGrpcServiceClient<Channel>>,
    timeout: Duration,
}

impl TopcsAndQueuesSnapshotRepo {
    pub fn new(settings: &SettingsModel) -> TopcsAndQueuesSnapshotRepo {
        TopcsAndQueuesSnapshotRepo {
            grpc_address: settings.persistence_grpc_url.to_string(),
            grpc_client: LazyObject::new(),
            timeout: settings.grpc_timeout,
        }
    }

    async fn get_grpc_client<'s>(
        &'s self,
    ) -> Result<
        LazyObjectAccess<'s, MyServiceBusQueuePersistenceGrpcServiceClient<Channel>>,
        PersistenceError,
    > {
        if !self.grpc_client.has_instance().await {
            let grpc_addess = self.grpc_address.to_string();
            let instance = init_grpc_client(&grpc_addess, self.timeout).await?;
            self.grpc_client.init(instance).await;
        }

        let result = self.grpc_client.get();
        return Ok(result);
    }

    pub async fn save(&self, snapshot: Vec<TopicSnapshot>) -> Result<(), PersistenceError> {
        let grpc_client_lazy_object = self.get_grpc_client().await?;

        let mut grpc_client = grpc_client_lazy_object.get_mut().await;

        let grpc_client = grpc_client.as_mut();

        if grpc_client.is_none() {
            return Err(PersistenceError::GrpcClientIsNotInitialized(
                "queue_snapshot_repo::save".to_string(),
            ));
        }

        save_snapshot_with_timeout(grpc_client.unwrap(), snapshot, self.timeout).await;

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

        let result = load_snapshot_with_timeout(grpc_client.unwrap(), self.timeout).await;

        Ok(result)
    }
}

async fn init_grpc_client(
    grpc_address: &str,
    timeout: Duration,
) -> Result<MyServiceBusQueuePersistenceGrpcServiceClient<Channel>, PersistenceError> {
    let mut attempt_no = 0;

    loop {
        let result = tokio::time::timeout(
            timeout,
            MyServiceBusQueuePersistenceGrpcServiceClient::connect(grpc_address.to_string()),
        )
        .await;

        if let Ok(result) = result {
            match result {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    println!(
                        "Can not get grpc client. Attempt: {},  Reason:{}",
                        attempt_no, err
                    );
                }
            }
        } else {
            println!("Initializinf grpc client timeout. Attempt: {}", attempt_no);
        }

        if attempt_no >= 5 {
            return Err(PersistenceError::CanNotInitializeGrpcService);
        }

        attempt_no += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn save_snapshot_with_timeout(
    grpc: &mut MyServiceBusQueuePersistenceGrpcServiceClient<Channel>,
    snapshot: Vec<TopicSnapshot>,
    timeout: Duration,
) {
    let grpc_request: SaveQueueSnapshotGrpcRequest = snapshot.into();

    match tokio::time::timeout(timeout, grpc.save_snapshot(grpc_request)).await {
        Ok(result) => {
            result.unwrap();
            return;
        }
        Err(_) => panic!("save_snapshot timeout"),
    }
}

async fn load_snapshot_with_timeout(
    grpc: &mut MyServiceBusQueuePersistenceGrpcServiceClient<Channel>,
    timeout: Duration,
) -> Vec<TopicSnapshot> {
    let mut attempt_no = 0;

    loop {
        match tokio::time::timeout(timeout, grpc.get_snapshot(())).await {
            Ok(response) => {
                let response = response.unwrap();

                let mut response = response.into_inner();

                let mut result: Vec<TopicSnapshot> = Vec::new();

                while let Some(item) = response.next().await {
                    let grpc_model = item.unwrap();
                    result.push(grpc_model.into());
                }

                return result;
            }
            Err(_) => {
                if attempt_no >= 5 {
                    panic!("save_snapshot timeout");
                }

                attempt_no += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
