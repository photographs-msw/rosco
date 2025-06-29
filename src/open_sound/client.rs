use std::net::SocketAddr;
use tokio::net::{TcpStream, UdpSocket};
use tokio::io::AsyncWriteExt;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundBundle, OpenSoundConfig};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};
use crate::open_sound::message::{serialize_message, serialize_bundle};

/// OpenSound Control client
pub struct OpenSoundClient {
    config: OpenSoundConfig,
    tcp_stream: Option<TcpStream>,
    udp_socket: Option<UdpSocket>,
    tcp_addr: Option<SocketAddr>,
    udp_addr: Option<SocketAddr>,
}

impl OpenSoundClient {
    /// Create a new OpenSound client
    pub fn new(config: OpenSoundConfig) -> Self {
        Self {
            config,
            tcp_stream: None,
            udp_socket: None,
            tcp_addr: None,
            udp_addr: None,
        }
    }

    /// Connect to TCP server
    pub async fn connect_tcp(&mut self, addr: &str) -> OpenSoundResult<()> {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| OpenSoundError::ConnectionError(format!("Invalid TCP address: {}", e)))?;
        
        let stream = TcpStream::connect(socket_addr).await
            .map_err(|e| OpenSoundError::ConnectionError(format!("Failed to connect to TCP server: {}", e)))?;
        
        self.tcp_stream = Some(stream);
        self.tcp_addr = Some(socket_addr);
        println!("Connected to OpenSound TCP server at {}", addr);
        Ok(())
    }

    /// Connect to UDP server
    pub async fn connect_udp(&mut self, addr: &str) -> OpenSoundResult<()> {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| OpenSoundError::ConnectionError(format!("Invalid UDP address: {}", e)))?;
        
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| OpenSoundError::ConnectionError(format!("Failed to bind UDP socket: {}", e)))?;
        
        self.udp_socket = Some(socket);
        self.udp_addr = Some(socket_addr);
        println!("Connected to OpenSound UDP server at {}", addr);
        Ok(())
    }

    /// Send a message over TCP
    pub async fn send_message_tcp(&mut self, message: &OpenSoundMessage) -> OpenSoundResult<()> {
        if let Some(ref mut stream) = self.tcp_stream {
            let data = serialize_message(message)?;
            stream.write_all(&data).await
                .map_err(|e| OpenSoundError::Network(e))?;
            Ok(())
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to TCP server".to_string()))
        }
    }

    /// Send a message over UDP
    pub async fn send_message_udp(&self, message: &OpenSoundMessage) -> OpenSoundResult<()> {
        if let (Some(ref socket), Some(addr)) = (&self.udp_socket, self.udp_addr) {
            let data = serialize_message(message)?;
            socket.send_to(&data, addr).await
                .map_err(|e| OpenSoundError::Network(e))?;
            Ok(())
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to UDP server".to_string()))
        }
    }

    /// Send a bundle over TCP
    pub async fn send_bundle_tcp(&mut self, bundle: &OpenSoundBundle) -> OpenSoundResult<()> {
        if let Some(ref mut stream) = self.tcp_stream {
            let data = serialize_bundle(bundle)?;
            stream.write_all(&data).await
                .map_err(|e| OpenSoundError::Network(e))?;
            Ok(())
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to TCP server".to_string()))
        }
    }

    /// Send a bundle over UDP
    pub async fn send_bundle_udp(&self, bundle: &OpenSoundBundle) -> OpenSoundResult<()> {
        if let (Some(ref socket), Some(addr)) = (&self.udp_socket, self.udp_addr) {
            let data = serialize_bundle(bundle)?;
            socket.send_to(&data, addr).await
                .map_err(|e| OpenSoundError::Network(e))?;
            Ok(())
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to UDP server".to_string()))
        }
    }

    /// Send a message (auto-selects TCP if available, otherwise UDP)
    pub async fn send_message(&mut self, message: &OpenSoundMessage) -> OpenSoundResult<()> {
        if self.tcp_stream.is_some() {
            self.send_message_tcp(message).await
        } else if self.udp_socket.is_some() {
            self.send_message_udp(message).await
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to any server".to_string()))
        }
    }

    /// Send a bundle (auto-selects TCP if available, otherwise UDP)
    pub async fn send_bundle(&mut self, bundle: &OpenSoundBundle) -> OpenSoundResult<()> {
        if self.tcp_stream.is_some() {
            self.send_bundle_tcp(bundle).await
        } else if self.udp_socket.is_some() {
            self.send_bundle_udp(bundle).await
        } else {
            Err(OpenSoundError::ConnectionError("Not connected to any server".to_string()))
        }
    }

    /// Check if connected to TCP
    pub fn is_tcp_connected(&self) -> bool {
        self.tcp_stream.is_some()
    }

    /// Check if connected to UDP
    pub fn is_udp_connected(&self) -> bool {
        self.udp_socket.is_some()
    }

    /// Disconnect from all servers
    pub fn disconnect(&mut self) {
        self.tcp_stream = None;
        self.udp_socket = None;
        self.tcp_addr = None;
        self.udp_addr = None;
    }
}

impl Default for OpenSoundClient {
    fn default() -> Self {
        Self::new(OpenSoundConfig::default())
    }
}

/// Builder for creating OpenSound messages
pub struct MessageBuilder {
    address_pattern: String,
    arguments: Vec<crate::open_sound::types::OpenSoundArgument>,
}

impl MessageBuilder {
    pub fn new(address_pattern: &str) -> Self {
        Self {
            address_pattern: address_pattern.to_string(),
            arguments: Vec::new(),
        }
    }

    pub fn arg_i32(mut self, value: i32) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::Int32(value));
        self
    }

    pub fn arg_f32(mut self, value: f32) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::Float32(value));
        self
    }

    pub fn arg_string(mut self, value: &str) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::String(value.to_string()));
        self
    }

    pub fn arg_blob(mut self, value: Vec<u8>) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::Blob(value));
        self
    }

    pub fn arg_true(mut self) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::True);
        self
    }

    pub fn arg_false(mut self) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::False);
        self
    }

    pub fn arg_null(mut self) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::Null);
        self
    }

    pub fn arg_impulse(mut self) -> Self {
        self.arguments.push(crate::open_sound::types::OpenSoundArgument::Impulse);
        self
    }

    pub fn build(self) -> OpenSoundMessage {
        OpenSoundMessage {
            address_pattern: self.address_pattern,
            arguments: self.arguments,
        }
    }
}

/// Convenience functions for creating common musical messages
impl OpenSoundClient {
    /// Create an oscillator note message
    pub fn create_oscillator_message(
        frequency: f32,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
        waveforms: &str,
    ) -> OpenSoundMessage {
        MessageBuilder::new("/note/oscillator")
            .arg_f32(frequency)
            .arg_f32(volume)
            .arg_f32(start_time_ms)
            .arg_f32(duration_ms)
            .arg_string(waveforms)
            .build()
    }

    /// Create a sample note message
    pub fn create_sample_message(
        file_path: &str,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
    ) -> OpenSoundMessage {
        MessageBuilder::new("/note/sample")
            .arg_string(file_path)
            .arg_f32(volume)
            .arg_f32(start_time_ms)
            .arg_f32(duration_ms)
            .build()
    }

    /// Create a play message
    pub fn create_play_message(
        note_type: &str,
        frequency: f32,
        volume: f32,
        start_time_ms: f32,
        duration_ms: f32,
        waveforms: &str,
    ) -> OpenSoundMessage {
        MessageBuilder::new("/note/play")
            .arg_string(note_type)
            .arg_f32(frequency)
            .arg_f32(volume)
            .arg_f32(start_time_ms)
            .arg_f32(duration_ms)
            .arg_string(waveforms)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_sound::types::OpenSoundArgument;

    #[test]
    fn test_message_builder() {
        let message = MessageBuilder::new("/test")
            .arg_i32(42)
            .arg_f32(3.14)
            .arg_string("hello")
            .build();

        assert_eq!(message.address_pattern, "/test");
        assert_eq!(message.arguments.len(), 3);
    }

    #[test]
    fn test_create_oscillator_message() {
        let message = OpenSoundClient::create_oscillator_message(
            440.0, 0.5, 0.0, 1000.0, "sine"
        );

        assert_eq!(message.address_pattern, "/note/oscillator");
        assert_eq!(message.arguments.len(), 5);
    }
} 