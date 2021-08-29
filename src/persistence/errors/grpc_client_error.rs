pub struct GrpcClientError {
    pub err: tonic::transport::Error,
}

impl From<tonic::transport::Error> for GrpcClientError {
    fn from(src: tonic::transport::Error) -> Self {
        Self { err: src }
    }
}
