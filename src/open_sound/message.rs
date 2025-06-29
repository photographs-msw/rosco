use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::open_sound::types::{OpenSoundMessage, OpenSoundBundle, OpenSoundArgument};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};

/// Parse an OSC message from bytes
pub fn parse_message(data: &[u8]) -> OpenSoundResult<OpenSoundMessage> {
    let mut bytes = Bytes::copy_from_slice(data);
    
    // Parse address pattern
    let address_pattern = parse_string(&mut bytes)?;
    
    // Parse type tag string
    let type_tags = parse_string(&mut bytes)?;
    
    if !type_tags.starts_with(',') {
        return Err(OpenSoundError::ParseError(
            format!("Type tag string must start with ',' but got '{}'", type_tags)
        ));
    }
    
    // Parse arguments based on type tags
    let mut arguments = Vec::new();
    for tag in type_tags[1..].chars() {
        let arg = parse_argument(&mut bytes, tag)?;
        arguments.push(arg);
    }
    
    Ok(OpenSoundMessage {
        address_pattern,
        arguments,
    })
}

/// Parse an OSC bundle from bytes
pub fn parse_bundle(data: &[u8]) -> OpenSoundResult<OpenSoundBundle> {
    let mut bytes = Bytes::copy_from_slice(data);
    
    // Check bundle identifier
    let identifier = parse_string(&mut bytes)?;
    if identifier != "#bundle" {
        return Err(OpenSoundError::ParseError("Invalid bundle identifier".to_string()));
    }
    
    // Parse timestamp
    let timestamp = bytes.get_u64();
    
    // Parse messages
    let mut messages = Vec::new();
    while bytes.has_remaining() {
        let message_size = bytes.get_u32() as usize;
        if message_size > bytes.remaining() {
            return Err(OpenSoundError::ParseError("Message size exceeds remaining data".to_string()));
        }
        
        let message_data = bytes.copy_to_bytes(message_size);
        let message = parse_message(&message_data)?;
        messages.push(message);
    }
    
    Ok(OpenSoundBundle {
        timestamp,
        messages,
    })
}

/// Serialize an OSC message to bytes
pub fn serialize_message(message: &OpenSoundMessage) -> OpenSoundResult<Vec<u8>> {
    let mut buffer = BytesMut::new();
    
    // Write address pattern
    write_string(&mut buffer, &message.address_pattern)?;
    
    // Write type tag string
    let mut type_tags = String::from(",");
    for arg in &message.arguments {
        type_tags.push(match arg {
            OpenSoundArgument::Int32(_) => 'i',
            OpenSoundArgument::Float32(_) => 'f',
            OpenSoundArgument::String(_) => 's',
            OpenSoundArgument::Blob(_) => 'b',
            OpenSoundArgument::True => 'T',
            OpenSoundArgument::False => 'F',
            OpenSoundArgument::Null => 'N',
            OpenSoundArgument::Impulse => 'I',
        });
    }
    write_string(&mut buffer, &type_tags)?;
    
    // Write arguments
    for arg in &message.arguments {
        write_argument(&mut buffer, arg)?;
    }
    
    Ok(buffer.to_vec())
}

/// Serialize an OSC bundle to bytes
pub fn serialize_bundle(bundle: &OpenSoundBundle) -> OpenSoundResult<Vec<u8>> {
    let mut buffer = BytesMut::new();
    
    // Write bundle identifier
    write_string(&mut buffer, "#bundle")?;
    
    // Write timestamp
    buffer.put_u64(bundle.timestamp);
    
    // Write messages
    for message in &bundle.messages {
        let message_data = serialize_message(message)?;
        buffer.put_u32(message_data.len() as u32);
        buffer.extend_from_slice(&message_data);
    }
    
    Ok(buffer.to_vec())
}

// Helper functions

fn parse_string(bytes: &mut Bytes) -> OpenSoundResult<String> {
    if !bytes.has_remaining() {
        return Err(OpenSoundError::ParseError("No data remaining for string".to_string()));
    }
    
    let mut string_bytes = Vec::new();
    let mut found_null = false;
    
    while bytes.has_remaining() {
        let byte = bytes.get_u8();
        if byte == 0 {
            found_null = true;
            break;
        }
        string_bytes.push(byte);
    }
    
    if !found_null {
        return Err(OpenSoundError::ParseError("String not null-terminated".to_string()));
    }
    
    // Pad to 4-byte boundary
    let padding_needed = (4 - ((string_bytes.len() + 1) % 4)) % 4;
    for _ in 0..padding_needed {
        if bytes.has_remaining() {
            let byte = bytes.get_u8();
            if byte != 0 {
                return Err(OpenSoundError::ParseError(
                    format!("Invalid string padding: expected 0, got {}", byte)
                ));
            }
        }
    }
    
    String::from_utf8(string_bytes)
        .map_err(|e| OpenSoundError::ParseError(format!("Invalid UTF-8: {}", e)))
}

fn parse_argument(bytes: &mut Bytes, tag: char) -> OpenSoundResult<OpenSoundArgument> {
    match tag {
        'i' => Ok(OpenSoundArgument::Int32(bytes.get_i32())),
        'f' => Ok(OpenSoundArgument::Float32(bytes.get_f32())),
        's' => Ok(OpenSoundArgument::String(parse_string(bytes)?)),
        'b' => parse_blob(bytes),
        'T' => Ok(OpenSoundArgument::True),
        'F' => Ok(OpenSoundArgument::False),
        'N' => Ok(OpenSoundArgument::Null),
        'I' => Ok(OpenSoundArgument::Impulse),
        _ => Err(OpenSoundError::ParseError(format!("Unknown type tag: {}", tag))),
    }
}

fn parse_blob(bytes: &mut Bytes) -> OpenSoundResult<OpenSoundArgument> {
    if bytes.remaining() < 4 {
        return Err(OpenSoundError::ParseError("Insufficient data for blob size".to_string()));
    }
    
    let size = bytes.get_u32() as usize;
    if size > bytes.remaining() {
        return Err(OpenSoundError::ParseError("Blob size exceeds remaining data".to_string()));
    }
    
    let mut blob_data = vec![0u8; size];
    bytes.copy_to_slice(&mut blob_data);
    
    // Pad to 4-byte boundary
    let padding_needed = (4 - (size % 4)) % 4;
    for _ in 0..padding_needed {
        if bytes.has_remaining() {
            let byte = bytes.get_u8();
            if byte != 0 {
                return Err(OpenSoundError::ParseError("Invalid blob padding".to_string()));
            }
        }
    }
    
    Ok(OpenSoundArgument::Blob(blob_data))
}

fn write_string(buffer: &mut BytesMut, s: &str) -> OpenSoundResult<()> {
    buffer.extend_from_slice(s.as_bytes());
    buffer.put_u8(0);
    
    // Pad to 4-byte boundary
    let padding = (4 - (s.len() + 1) % 4) % 4;
    for _ in 0..padding {
        buffer.put_u8(0);
    }
    
    Ok(())
}

fn write_argument(buffer: &mut BytesMut, arg: &OpenSoundArgument) -> OpenSoundResult<()> {
    match arg {
        OpenSoundArgument::Int32(i) => buffer.put_i32(*i),
        OpenSoundArgument::Float32(f) => buffer.put_f32(*f),
        OpenSoundArgument::String(s) => write_string(buffer, s)?,
        OpenSoundArgument::Blob(b) => {
            buffer.put_u32(b.len() as u32);
            buffer.extend_from_slice(b);
            
            // Pad to 4-byte boundary
            let padding = (4 - b.len() % 4) % 4;
            for _ in 0..padding {
                buffer.put_u8(0);
            }
        }
        OpenSoundArgument::True | OpenSoundArgument::False | 
        OpenSoundArgument::Null | OpenSoundArgument::Impulse => {}
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_sound::types::OpenSoundMessage;

    #[test]
    fn test_serialize_deserialize_message() {
        let message = OpenSoundMessage {
            address_pattern: "/test".to_string(),
            arguments: vec![
                OpenSoundArgument::Int32(42),
                OpenSoundArgument::Float32(3.14),
                OpenSoundArgument::String("hello".to_string()),
            ],
        };
        
        let serialized = serialize_message(&message).unwrap();
        let deserialized = parse_message(&serialized).unwrap();
        
        assert_eq!(message.address_pattern, deserialized.address_pattern);
        assert_eq!(message.arguments.len(), deserialized.arguments.len());
    }
} 