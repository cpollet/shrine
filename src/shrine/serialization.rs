use crate::serialize::bson::BsonSerDe;
use crate::serialize::json::JsonSerDe;
use crate::serialize::message_pack::MessagePackSerDe;
use crate::serialize::SerDe;
use crate::shrine::local::Secrets;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::{Display, Formatter};

/// The serialization format
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum SerializationFormat {
    /// BSON, the data storage and network transfer format used by MongoDB.
    #[default]
    Bson,
    /// JSON, the ubiquitous JavaScript Object Notation used by many HTTP APIs.
    Json,
    /// MessagePack, an efficient binary format that resembles a compact JSON.
    MessagePack,
}

impl SerializationFormat {
    pub fn serializer(&self) -> Box<dyn SerDe<Secrets>> {
        match self {
            SerializationFormat::Bson => Box::new(BsonSerDe::new()),
            SerializationFormat::Json => Box::new(JsonSerDe::new()),
            SerializationFormat::MessagePack => Box::new(MessagePackSerDe::new()),
        }
    }
}

impl Display for SerializationFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationFormat::Bson => write!(f, "BSON"),
            SerializationFormat::Json => write!(f, "JSON"),
            SerializationFormat::MessagePack => write!(f, "MessagePack"),
        }
    }
}
