use my_service_bus_shared::MessageId;

#[derive(Debug)]
pub struct MessageToSendModel {
    pub msg_id: MessageId,
    pub attempt_no: i32,
    pub msg_size: usize,
}
