use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt};
use tokio::sync::mpsc;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundConfig};
use crate::open_sound::error::{OpenSoundError, OpenSoundResult};
use crate::open_sound::message::{parse_message, parse_bundle};
use crate::open_sound::handler::{RouteHandler};

/// OpenSound Control server
pub struct OpenSoundServer {
    config: OpenSoundConfig,
    route_manager: RouteManager,
    tcp_listener: Option<TcpListener>,
    udp_socket: Option<UdpSocket>,
    running: bool,
}

impl OpenSoundServer {
    pub fn new(config: OpenSoundConfig) -> Self {
        Self {
            config,
            route_manager: RouteManager::new(),
            tcp_listener: None,
            udp_socket: None,
            running: false,
        }
    }

    /// Add a route handler
    pub fn add_handler(&mut self, pattern: &str, handler: Box<dyn RouteHandler>) {
        self.route_manager.add_handler(pattern, handler);
    }

    /// Start TCP server
    pub async fn start_tcp(&mut self) -> OpenSoundResult<()> {
        if let Some(addr) = &self.config.tcp_address {
            let listener = TcpListener::bind(addr).await
                .map_err(|e| OpenSoundError::ConnectionError(format!("Failed to bind TCP: {}", e)))?;
            
            self.tcp_listener = Some(listener);
            println!("OpenSound TCP server listening on {}", addr);
        }
        Ok(())
    }

    /// Start UDP server
    pub async fn start_udp(&mut self) -> OpenSoundResult<()> {
        if let Some(addr) = &self.config.udp_address {
            let socket = UdpSocket::bind(addr).await
                .map_err(|e| OpenSoundError::ConnectionError(format!("Failed to bind UDP: {}", e)))?;
            
            self.udp_socket = Some(socket);
            println!("OpenSound UDP server listening on {}", addr);
        }
        Ok(())
    }

    /// Run the server
    pub async fn run(&mut self) -> OpenSoundResult<()> {
        self.running = true;

        // Start TCP and UDP servers
        self.start_tcp().await?;
        self.start_udp().await?;

        // Create channels for communication between tasks
        let (tx, mut rx) = mpsc::channel::<OpenSoundMessage>(100);

        // Spawn TCP handler task
        if let Some(listener) = self.tcp_listener.take() {
            let tx_clone = tx.clone();
            let route_manager = self.route_manager.clone();
            tokio::spawn(async move {
                Self::handle_tcp_connections(listener, tx_clone, route_manager).await;
            });
        }

        // Spawn UDP handler task
        if let Some(socket) = self.udp_socket.take() {
            let tx_clone = tx.clone();
            let route_manager = self.route_manager.clone();
            tokio::spawn(async move {
                Self::handle_udp_messages(socket, tx_clone, route_manager).await;
            });
        }

        // Main message processing loop
        while self.running {
            if let Some(message) = rx.recv().await {
                if let Err(e) = self.route_manager.handle_message(message) {
                    eprintln!("Error handling message: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Stop the server
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Handle TCP connections
    async fn handle_tcp_connections(
        listener: TcpListener,
        tx: mpsc::Sender<OpenSoundMessage>,
        route_manager: RouteManager,
    ) {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    let tx_clone = tx.clone();
                    let route_manager_clone = route_manager.clone();
                    tokio::spawn(async move {
                        Self::handle_tcp_connection(socket, addr, tx_clone, route_manager_clone).await;
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting TCP connection: {}", e);
                }
            }
        }
    }

    /// Handle individual TCP connection
    async fn handle_tcp_connection(
        mut socket: TcpStream,
        addr: SocketAddr,
        tx: mpsc::Sender<OpenSoundMessage>,
        route_manager: RouteManager,
    ) {
        // TEMP DEBUG
        println!("TCP connection established with {}", addr);

        let mut buffer = vec![0u8; 4096];
        
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    let data = &buffer[..n];
                    
                    match Self::process_message_data(data, &tx, &route_manager).await {
                        Ok(_) => {
                            println!("Successfully processed message from {}", addr);
                        }
                        Err(e) => {
                            eprintln!("Error processing message from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from TCP connection {}: {}", addr, e);
                    break;
                }
            }
        }
    }

    /// Handle UDP messages
    async fn handle_udp_messages(
        socket: UdpSocket,
        tx: mpsc::Sender<OpenSoundMessage>,
        route_manager: RouteManager,
    ) {
        let mut buffer = vec![0u8; 4096];
        
        loop {
            match socket.recv_from(&mut buffer).await {
                Ok((n, addr)) => {
                    let data = &buffer[..n];
                    match Self::process_message_data(data, &tx, &route_manager).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error processing UDP message from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP message: {}", e);
                }
            }
        }
    }

    /// Process message data (message or bundle)
    async fn process_message_data(
        data: &[u8],
        tx: &mpsc::Sender<OpenSoundMessage>,
        route_manager: &RouteManager,
    ) -> OpenSoundResult<()> {
        // Try to parse as bundle first
        if let Ok(bundle) = parse_bundle(data) {
            for message in bundle.messages {
                if let Err(e) = tx.send(message).await {
                    return Err(OpenSoundError::ConnectionError(format!("Failed to send message: {}", e)));
                }
            }
        } else {
            // Try to parse as single message
            let message = parse_message(data)?;
            if let Err(e) = tx.send(message).await {
                return Err(OpenSoundError::ConnectionError(format!("Failed to send message: {}", e)));
            }
        }
        Ok(())
    }
}

pub struct RouteManager {
    handlers: Arc<Mutex<HashMap<String, Box<dyn RouteHandler>>>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_handler(&mut self, pattern: &str, handler: Box<dyn RouteHandler>) {
        if let Ok(mut handlers) = self.handlers.lock() {
            handlers.insert(pattern.to_string(), handler);
        }
    }

    pub fn handle_message(&self, message: OpenSoundMessage) -> OpenSoundResult<()> {
        if let Ok(handlers) = self.handlers.lock() {
            // Try exact match first
            if let Some(handler) = handlers.get(&message.address_pattern) {
                return handler.handle(message);
            }

            // Try pattern matching
            for (pattern, handler) in handlers.iter() {
                if self.matches_pattern(pattern, &message.address_pattern) {
                    return handler.handle(message);
                }
            }
        }
        Err(OpenSoundError::InvalidAddressPattern(message.address_pattern))
    }

    fn matches_pattern(&self, pattern: &str, address: &str) -> bool {
        // Simple wildcard matching - can be extended for more complex patterns
        if pattern.ends_with("/*") {
            let pattern_prefix = &pattern[..pattern.len() - 2];
            address.starts_with(pattern_prefix)
        } else {
            pattern == address
        }
    }
}

impl Clone for RouteManager {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
} 


impl Default for OpenSoundServer {
    fn default() -> Self {
        Self::new(OpenSoundConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = OpenSoundConfig::default();
        let server = OpenSoundServer::new(config);
        assert!(!server.is_running());
    }
} 