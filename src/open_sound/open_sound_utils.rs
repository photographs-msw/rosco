// Utility functions for OpenSound integration tests and helpers
use std::io::Cursor;
use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

/// Enum for OSC arguments to handle mixed types
#[derive(Debug)]
pub enum OscArg {
    Int(i32),
    Float(f32),
    String(String),
}

/// Helper function to create OSC message bytes from address and arguments
pub fn create_osc_message_bytes(address: &str, args: &[(&str, impl ToString)]) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // Write address pattern (null-terminated, padded to 4-byte boundary)
    cursor.write_all(address.as_bytes()).unwrap();
    cursor.write_u8(0).unwrap();
    let padding = (4 - (address.len() + 1) % 4) % 4;
    for _ in 0..padding {
        cursor.write_u8(0).unwrap();
    }

    // Write type tag string
    let mut type_tags = String::from(",");
    for (tag, _) in args {
        type_tags.push_str(tag);
    }
    cursor.write_all(type_tags.as_bytes()).unwrap();
    cursor.write_u8(0).unwrap();
    let padding = (4 - (type_tags.len() + 1) % 4) % 4;
    for _ in 0..padding {
        cursor.write_u8(0).unwrap();
    }

    // Write arguments
    for (tag, value) in args {
        match *tag {
            "i" => {
                let int_val: i32 = value.to_string().parse().unwrap();
                cursor.write_i32::<BigEndian>(int_val).unwrap();
            }
            "f" => {
                let float_val: f32 = value.to_string().parse().unwrap();
                cursor.write_f32::<BigEndian>(float_val).unwrap();
            }
            "s" => {
                let str_val = value.to_string();
                cursor.write_all(str_val.as_bytes()).unwrap();
                cursor.write_u8(0).unwrap();
                let padding = (4 - (str_val.len() + 1) % 4) % 4;
                for _ in 0..padding {
                    cursor.write_u8(0).unwrap();
                }
            }
            _ => panic!("Unsupported type tag: {}", tag),
        }
    }

    buffer
}

/// Helper function to create OSC bundle bytes
pub fn create_osc_bundle_bytes(messages: &[(&str, &[(&str, f32)])]) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // Write bundle identifier
    cursor.write_all(b"#bundle").unwrap();
    cursor.write_u8(0).unwrap();

    // Write timestamp (immediate)
    cursor.write_u64::<BigEndian>(1).unwrap();

    // Write each message
    for (address, args) in messages {
        let message_bytes = create_osc_message_bytes(address, args);
        cursor.write_u32::<BigEndian>(message_bytes.len() as u32).unwrap();
        cursor.write_all(&message_bytes).unwrap();
    }

    buffer
}

/// Helper function to create OSC message bytes with mixed types
pub fn create_osc_message_bytes_mixed(address: &str, args: &[OscArg]) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // Write address pattern (null-terminated, padded to 4-byte boundary)
    cursor.write_all(address.as_bytes()).unwrap();
    cursor.write_u8(0).unwrap();
    let padding = (4 - (address.len() + 1) % 4) % 4;
    for _ in 0..padding {
        cursor.write_u8(0).unwrap();
    }

    // Write type tag string
    let mut type_tags = String::from(",");
    for arg in args {
        type_tags.push_str(match arg {
            OscArg::Int(_) => "i",
            OscArg::Float(_) => "f",
            OscArg::String(_) => "s",
        });
    }
    cursor.write_all(type_tags.as_bytes()).unwrap();
    cursor.write_u8(0).unwrap();
    let padding = (4 - (type_tags.len() + 1) % 4) % 4;
    for _ in 0..padding {
        cursor.write_u8(0).unwrap();
    }

    // Write arguments
    for arg in args {
        match arg {
            OscArg::Int(val) => {
                cursor.write_i32::<BigEndian>(*val).unwrap();
            }
            OscArg::Float(val) => {
                cursor.write_f32::<BigEndian>(*val).unwrap();
            }
            OscArg::String(val) => {
                cursor.write_all(val.as_bytes()).unwrap();
                cursor.write_u8(0).unwrap();
                let padding = (4 - (val.len() + 1) % 4) % 4;
                for _ in 0..padding {
                    cursor.write_u8(0).unwrap();
                }
            }
        }
    }

    buffer
}

/// Helper function to create OSC bundle bytes with mixed types
pub fn create_osc_bundle_bytes_mixed(messages: &[(&str, &[OscArg])]) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // Write bundle identifier
    cursor.write_all(b"#bundle").unwrap();
    cursor.write_u8(0).unwrap();

    // Write timestamp (immediate)
    cursor.write_u64::<BigEndian>(1).unwrap();

    // Write each message
    for (address, args) in messages {
        let message_bytes = create_osc_message_bytes_mixed(address, args);
        cursor.write_u32::<BigEndian>(message_bytes.len() as u32).unwrap();
        cursor.write_all(&message_bytes).unwrap();
    }

    buffer
} 