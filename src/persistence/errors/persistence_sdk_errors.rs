use zip::result::ZipError;

use crate::{
    bcl_proto::{BclDateTime, BclDateTimeError, BclToUnixMicroseconds},
    date_time::MyDateTime,
};

use super::GrpcClientError;

#[derive(Debug)]
pub enum PersistenceError {
    ZipOperationError(ZipError),
    TonicError(tonic::Status),
    InvalidProtobufPayload(String),
    GrpcClientError(tonic::transport::Error),
    GrpcClientIsNotInitialized(String),
}

impl From<GrpcClientError> for PersistenceError {
    fn from(src: GrpcClientError) -> Self {
        Self::GrpcClientError(src.err)
    }
}

impl From<tonic::Status> for PersistenceError {
    fn from(src: tonic::Status) -> Self {
        Self::TonicError(src)
    }
}

impl From<prost::DecodeError> for PersistenceError {
    fn from(src: prost::DecodeError) -> Self {
        Self::InvalidProtobufPayload(format!("{:?}", src))
    }
}

impl From<ZipError> for PersistenceError {
    fn from(src: ZipError) -> Self {
        Self::ZipOperationError(src)
    }
}

impl MyDateTime {
    pub fn from_optional_bcl(dt: Option<BclDateTime>) -> Result<Self, BclDateTimeError> {
        if dt.is_none() {
            return Err(BclDateTimeError {
                reason: "Date time is null".to_string(),
            });
        }

        return Ok(MyDateTime::from_bcl(&dt.unwrap())?);
    }

    pub fn from_bcl(dt: &BclDateTime) -> Result<Self, BclDateTimeError> {
        let micros = dt.to_unix_microseconds()?;
        Ok(MyDateTime { micros })
    }
}
