#[derive(Debug)]
pub enum SessionOperationError {
    Disconnected,
    CanNotSendOperationToSocket(String),
    JustDisconnected,
}
