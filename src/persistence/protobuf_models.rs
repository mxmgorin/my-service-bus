use my_service_bus_shared::bcl::BclDateTime;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessagesProtobufModel {
    #[prost(message, repeated, tag = "1")]
    pub messages: Vec<MessageProtobufModel>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewMessagesProtobufContract {
    #[prost(string, tag = "1")]
    pub topic_id: ::prost::alloc::string::String,

    #[prost(repeated, message, tag = "2")]
    pub messages: Vec<MessageProtobufModel>,
}

impl NewMessagesProtobufContract {
    pub fn into_protobuf_vec(&self) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::new();
        prost::Message::encode(self, &mut payload).unwrap(); //Remove Unwrap
        payload
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageProtobufModel {
    #[prost(int64, tag = "1")]
    pub message_id: i64,
    #[prost(message, tag = "2")]
    pub created: Option<BclDateTime>,
    #[prost(bytes, tag = "3")]
    pub data: Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub metadata: Vec<MessageMetaDataProtobufModel>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageMetaDataProtobufModel {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
