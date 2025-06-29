# OpenSound Control (OSC) Protocol Implementation

This module provides a complete implementation of the OpenSound Control protocol for musical note generation and audio synthesis. It integrates seamlessly with the existing audio system to convert OSC messages into `PlaybackNote` objects that can be processed by the audio generation system.

## Features

- **TCP and UDP Support**: Both transport protocols are supported for maximum flexibility
- **Message Parsing**: Full OSC message and bundle parsing with proper padding and alignment
- **Musical Integration**: Direct integration with the audio synthesis system via `gen_notes_stream()`
- **Async Support**: Built on tokio for high-performance async I/O
- **Type Safety**: Strongly typed message arguments and comprehensive error handling
- **Pattern Matching**: Support for wildcard route patterns
- **Bundle Support**: Full OSC bundle support with timestamps

## Quick Start

### Server Example

```rust
use crate::open_sound::{OpenSoundServer, MusicalNoteHandler};

#[tokio::main]
async fn main() {
    let mut server = OpenSoundServer::default();
    server.add_handler("/note/*", Box::new(MusicalNoteHandler::new()));
    server.run().await.unwrap();
}
```

### Client Example

```rust
use crate::open_sound::OpenSoundClient;

#[tokio::main]
async fn main() {
    let mut client = OpenSoundClient::default();
    client.connect_tcp("127.0.0.1:8000").await.unwrap();
    
    let message = client.create_oscillator_message(440.0, 0.5, 0.0, 1000.0, "sine");
    client.send_message(&message).await.unwrap();
}
```

## Message Format

### Oscillator Note
```
/note/oscillator frequency volume start_time_ms duration_ms waveforms
```

**Arguments:**
- `frequency` (f32): Frequency in Hz (e.g., 440.0 for A4)
- `volume` (f32): Volume level 0.0-1.0
- `start_time_ms` (f32): Start time in milliseconds
- `duration_ms` (f32): Duration in milliseconds
- `waveforms` (string): Comma-separated list of waveforms (sine, square, triangle, saw, noise)

**Example:**
```
/note/oscillator 440.0 0.5 0.0 1000.0 "sine,square"
```

### Sample Note
```
/note/sample file_path volume start_time_ms duration_ms
```

**Arguments:**
- `file_path` (string): Path to the audio file
- `volume` (f32): Volume level 0.0-1.0
- `start_time_ms` (f32): Start time in milliseconds
- `duration_ms` (f32): Duration in milliseconds

**Example:**
```
/note/sample "/path/to/sample.wav" 0.7 500.0 2000.0
```

### Immediate Play
```
/note/play note_type frequency volume start_time_ms duration_ms waveforms
```

**Arguments:**
- `note_type` (string): "oscillator" or "sample"
- `frequency` (f32): Frequency in Hz (for oscillator)
- `volume` (f32): Volume level 0.0-1.0
- `start_time_ms` (f32): Start time in milliseconds
- `duration_ms` (f32): Duration in milliseconds
- `waveforms` (string): Comma-separated list of waveforms (for oscillator)

**Example:**
```
/note/play "oscillator" 880.0 0.3 0.0 500.0 "square"
```

## API Reference

### OpenSoundServer

The main server implementation that handles incoming OSC messages.

```rust
pub struct OpenSoundServer {
    config: OpenSoundConfig,
    route_manager: RouteManager,
    // ... internal fields
}

impl OpenSoundServer {
    pub fn new(config: OpenSoundConfig) -> Self;
    pub fn add_handler(&mut self, pattern: &str, handler: Box<dyn RouteHandler>);
    pub async fn start_tcp(&mut self) -> OpenSoundResult<()>;
    pub async fn start_udp(&mut self) -> OpenSoundResult<()>;
    pub async fn run(&mut self) -> OpenSoundResult<()>;
    pub fn stop(&mut self);
}
```

### OpenSoundClient

The client implementation for sending OSC messages.

```rust
pub struct OpenSoundClient {
    config: OpenSoundConfig,
    // ... internal fields
}

impl OpenSoundClient {
    pub fn new(config: OpenSoundConfig) -> Self;
    pub async fn connect_tcp(&mut self, addr: &str) -> OpenSoundResult<()>;
    pub async fn connect_udp(&mut self, addr: &str) -> OpenSoundResult<()>;
    pub async fn send_message(&self, message: &OpenSoundMessage) -> OpenSoundResult<()>;
    pub async fn send_bundle(&self, bundle: &OpenSoundBundle) -> OpenSoundResult<()>;
    
    // Convenience methods
    pub fn create_oscillator_message(frequency, volume, start_time, duration, waveforms) -> OpenSoundMessage;
    pub fn create_sample_message(file_path, volume, start_time, duration) -> OpenSoundMessage;
    pub fn create_play_message(note_type, frequency, volume, start_time, duration, waveforms) -> OpenSoundMessage;
}
```

### OpenSoundIntegration

Integration layer that connects OSC messages to the audio system.

```rust
pub struct OpenSoundIntegration {
    oscillator_tables: OscillatorTables,
    note_queue: VecDeque<PlaybackNote>,
    // ... internal fields
}

impl OpenSoundIntegration {
    pub fn new() -> Self;
    pub fn queue_note(&mut self, note: PlaybackNote);
    pub fn play_queued_notes(&mut self) -> OpenSoundResult<()>;
    pub fn play_immediate_note(&mut self, note: PlaybackNote) -> OpenSoundResult<()>;
    pub fn process_message(&mut self, message: OpenSoundMessage) -> OpenSoundResult<()>;
    
    // Convenience methods
    pub fn queue_oscillator_note(frequency, volume, start_time, duration, waveforms) -> OpenSoundResult<()>;
    pub fn queue_sample_note(file_path, volume, start_time, duration) -> OpenSoundResult<()>;
    pub fn play_oscillator_note(frequency, volume, start_time, duration, waveforms) -> OpenSoundResult<()>;
}
```

## Configuration

### OpenSoundConfig

```rust
pub struct OpenSoundConfig {
    pub tcp_address: Option<String>,
    pub udp_address: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl Default for OpenSoundConfig {
    fn default() -> Self {
        Self {
            tcp_address: Some("127.0.0.1:8000".to_string()),
            udp_address: Some("127.0.0.1:8001".to_string()),
            sample_rate: 44100,
            buffer_size: 1024,
        }
    }
}
```

## Error Handling

The implementation uses a comprehensive error handling system:

```rust
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
```

## Examples

### Running a Server

```rust
use crate::open_sound::server_example;

#[tokio::main]
async fn main() {
    server_example::run_server().await;
}
```

### Running a Client

```rust
use crate::open_sound::client_example;

#[tokio::main]
async fn main() {
    client_example::run_client().await;
}
```

### Sending a Musical Scale

```rust
use crate::open_sound::client_example;

#[tokio::main]
async fn main() {
    let mut client = OpenSoundClient::default();
    client.connect_tcp("127.0.0.1:8000").await.unwrap();
    
    client_example::send_musical_scale(&client).await;
}
```

### Using Integration Directly

```rust
use crate::open_sound::OpenSoundIntegration;

fn main() {
    let mut integration = OpenSoundIntegration::new();
    
    // Queue some notes
    integration.queue_oscillator_note(440.0, 0.5, 0.0, 1000.0, "sine").unwrap();
    integration.queue_oscillator_note(554.37, 0.4, 1000.0, 1000.0, "triangle").unwrap();
    
    // Play all queued notes
    integration.play_queued_notes().unwrap();
}
```

## Testing

Run the tests with:

```bash
cargo test open_sound
```

## Performance Considerations

- **Async I/O**: The implementation uses tokio for high-performance async I/O
- **Message Queuing**: Messages are processed asynchronously to avoid blocking
- **Memory Efficiency**: Uses efficient byte buffers and avoids unnecessary allocations
- **Error Recovery**: Graceful error handling with detailed error messages

## Integration with Existing System

The OpenSound Protocol implementation integrates seamlessly with the existing audio system:

1. **Message Parsing**: OSC messages are parsed into strongly-typed arguments
2. **Note Creation**: Messages are converted into `PlaybackNote` objects
3. **Audio Generation**: Notes are processed by the existing `gen_notes_stream()` function
4. **Effect Support**: All existing effects (envelopes, LFOs, delays, flangers) are supported

## Future Enhancements

- **MIDI Integration**: Convert MIDI messages to OSC format
- **Pattern Matching**: More sophisticated OSC address pattern matching
- **Bundle Timing**: Precise timing for OSC bundles
- **Network Discovery**: Automatic server discovery
- **Security**: Authentication and encryption support
- **Performance Monitoring**: Metrics and profiling tools 
 