use std::time::Duration;

use crate::utils::FromStr;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModelJson {
    #[serde(rename = "GrpcUrl")]
    pub persistence_grpc_url: String,

    #[serde(rename = "EventuallyPersistenceDelay")]
    pub eventually_persistence_delay: String,

    #[serde(rename = "QueueGcTimeout")]
    pub queue_gc_timeout: String,

    #[serde(rename = "DebugMode")]
    pub debug_mode: bool,

    #[serde(rename = "MaxDeliverySize")]
    pub max_delivery_size: usize,
}

pub struct SettingsModel {
    pub persistence_grpc_url: String,
    pub eventually_persistence_delay: Duration,
    pub queue_gc_timeout: Duration,
    pub debug_mode: bool,

    pub max_delivery_size: usize,
}

pub async fn read() -> SettingsModel {
    let home_path = env!("HOME");
    let filename = format!("{}/{}", home_path, ".myservicebus");

    println!("Reading settings file {}", filename);

    let file = File::open(&filename).await;

    if let Err(err) = file {
        panic!(
            "Can not open settings file: {}. The reason is: {:?}",
            filename, err
        );
    }

    let mut file = file.unwrap();

    let mut file_content: Vec<u8> = Vec::new();

    loop {
        let res = file.read_buf(&mut file_content).await.unwrap();

        if res == 0 {
            break;
        }
    }

    let result: SettingsModelJson = serde_yaml::from_slice(file_content.as_slice()).unwrap();

    result.into()
}

impl Into<SettingsModel> for SettingsModelJson {
    fn into(self) -> SettingsModel {
        let queue_gc_timeout = Duration::from_str(self.queue_gc_timeout.as_str()).unwrap();

        let eventually_persistence_delay =
            Duration::from_str(self.eventually_persistence_delay.as_str()).unwrap();

        SettingsModel {
            persistence_grpc_url: self.persistence_grpc_url,
            debug_mode: self.debug_mode,
            queue_gc_timeout,
            eventually_persistence_delay,
            max_delivery_size: self.max_delivery_size,
        }
    }
}
