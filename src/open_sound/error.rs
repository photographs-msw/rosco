use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenSoundError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    #[error("Message parsing error: {0}")]
    ParseError(String),

    #[error("Invalid argument type: expected {expected}, got {actual}")]
    InvalidArgumentType { expected: String, actual: String },

    #[error("Audio generation error: {0}")]
    AudioError(String),

    #[error("Invalid address pattern: {0}")]
    InvalidAddressPattern(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Invalid argument count: expected {expected}, got {actual}")]
    InvalidArgumentCount { expected: usize, actual: usize },

    #[error("Unsupported message type: {0}")]
    UnsupportedMessageType(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

pub type OpenSoundResult<T> = Result<T, OpenSoundError>; 