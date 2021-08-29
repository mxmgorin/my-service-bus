use crate::{operations::OperationFailResult, persistence::PersistenceError};

pub enum AppError {
    OperationFail(OperationFailResult),
    PersistenceFail(PersistenceError),
}

impl From<prost::DecodeError> for OperationFailResult {
    fn from(src: prost::DecodeError) -> Self {
        Self::InvalidProtobufPayload(format!("{:?}", src))
    }
}

impl From<String> for OperationFailResult {
    fn from(src: String) -> Self {
        Self::Other(src)
    }
}
