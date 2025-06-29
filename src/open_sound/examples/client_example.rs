use tokio::time::{sleep, Duration};
use crate::open_sound::client::OpenSoundClient;
use crate::open_sound::types::{OpenSoundMessage, OpenSoundConfig, OpenSoundArgument};

/// Example OpenSound client for sending musical notes
pub async fn run_client() {
    println!("Starting OpenSound client example...");
    
    // Create client configuration
    let config = OpenSoundConfig::default();
    let mut client = OpenSoundClient::new(config);
    
    // Connect to TCP server
    match client.connect_tcp("127.0.0.1:8000").await {
        Ok(_) => println!("Connected to TCP server"),
        Err(e) => {
            eprintln!("Failed to connect to TCP server: {}", e);
            return;
        }
    }
    
    // Send some example messages
    send_example_messages(&mut client).await;
    
    // Disconnect
    client.disconnect();
    println!("Client disconnected");
}

/// Send example musical messages
async fn send_example_messages(client: &mut OpenSoundClient) {
    println!("Sending example messages...");
    
    // Example 1: Send an oscillator note
    let oscillator_message = OpenSoundMessage {
        address_pattern: "/note/oscillator".to_string(),
        arguments: vec![
            OpenSoundArgument::Float32(440.0),  // frequency (A4)
            OpenSoundArgument::Float32(0.5),    // volume
            OpenSoundArgument::Float32(0.0),    // start_time_ms
            OpenSoundArgument::Float32(1000.0), // duration_ms
            OpenSoundArgument::String("sine".to_string()), // waveforms
        ],
    };
    
    if let Err(e) = client.send_message(&oscillator_message).await {
        eprintln!("Failed to send oscillator message: {}", e);
    } else {
        println!("Sent oscillator note: A4 (440Hz)");
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Example 2: Send a sample note
    let sample_message = OpenSoundMessage {
        address_pattern: "/note/sample".to_string(),
        arguments: vec![
            OpenSoundArgument::String("/path/to/sample.wav".to_string()), // file_path
            OpenSoundArgument::Float32(0.7),    // volume
            OpenSoundArgument::Float32(500.0),  // start_time_ms
            OpenSoundArgument::Float32(2000.0), // duration_ms
        ],
    };
    
    if let Err(e) = client.send_message(&sample_message).await {
        eprintln!("Failed to send sample message: {}", e);
    } else {
        println!("Sent sample note");
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Example 3: Send an immediate play message
    let play_message = OpenSoundMessage {
        address_pattern: "/note/play".to_string(),
        arguments: vec![
            OpenSoundArgument::String("oscillator".to_string()), // note_type
            OpenSoundArgument::Float32(880.0),  // frequency (A5)
            OpenSoundArgument::Float32(0.3),    // volume
            OpenSoundArgument::Float32(0.0),    // start_time_ms
            OpenSoundArgument::Float32(500.0),  // duration_ms
            OpenSoundArgument::String("square".to_string()), // waveforms
        ],
    };
    
    if let Err(e) = client.send_message(&play_message).await {
        eprintln!("Failed to send play message: {}", e);
    } else {
        println!("Sent immediate play note: A5 (880Hz) square wave");
    }
}

/// Example using the convenience functions
pub async fn run_client_with_convenience_functions() {
    println!("Starting OpenSound client with convenience functions...");
    
    let mut client = OpenSoundClient::new(OpenSoundConfig::default());
    
    // Connect to UDP server
    match client.connect_udp("127.0.0.1:8001").await {
        Ok(_) => println!("Connected to UDP server"),
        Err(e) => {
            eprintln!("Failed to connect to UDP server: {}", e);
            return;
        }
    }
    
    // Send messages using convenience functions
    let messages = vec![
        OpenSoundClient::create_oscillator_message(440.0, 0.5, 0.0, 1000.0, "sine"),
        OpenSoundClient::create_oscillator_message(554.37, 0.4, 1000.0, 1000.0, "triangle"), // C#5
        OpenSoundClient::create_oscillator_message(659.25, 0.6, 2000.0, 1000.0, "saw"),     // E5
        OpenSoundClient::create_play_message("oscillator", 880.0, 0.3, 0.0, 500.0, "square"),
    ];
    
    for (i, message) in messages.iter().enumerate() {
        if let Err(e) = client.send_message(message).await {
            eprintln!("Failed to send message {}: {}", i + 1, e);
        } else {
            println!("Sent message {}: {}", i + 1, message.address_pattern);
        }
        sleep(Duration::from_millis(300)).await;
    }
    
    client.disconnect();
    println!("Client disconnected");
}

// TODO MODIFY TO TAKE A SCALE AS AN ENUM ARGUMENT
/// Example of sending a musical scale
pub async fn send_musical_scale(client: &mut OpenSoundClient) {
    println!("Sending C major scale...");
    
    let scale_frequencies = vec![
        261.63, // C4
        293.66, // D4
        329.63, // E4
        349.23, // F4
        392.00, // G4
        440.00, // A4
        493.88, // B4
        523.25, // C5
    ];
    
    for (i, freq) in scale_frequencies.iter().enumerate() {
        let message = OpenSoundClient::create_oscillator_message(
            *freq,
            0.4,
            i as f32 * 500.0, // Each note starts 500ms after the previous
            400.0,             // Each note lasts 400ms
            "sine",
        );
        
        if let Err(e) = client.send_message(&message).await {
            eprintln!("Failed to send scale note {}: {}", i + 1, e);
        } else {
            println!("Sent scale note {}: {:.2}Hz", i + 1, freq);
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}

/// Example of sending a chord
pub async fn send_chord(client: &mut OpenSoundClient) {
    println!("Sending C major chord...");
    
    let chord_frequencies = vec![
        261.63, // C4 (root)
        329.63, // E4 (third)
        392.00, // G4 (fifth)
    ];
    
    let mut messages = Vec::new();
    for (i, &freq) in chord_frequencies.iter().enumerate() {
        messages.push(OpenSoundClient::create_oscillator_message(
            freq,
            0.3,
            0.0,    // All notes start at the same time
            2000.0, // All notes last 2 seconds
            "sine",
        ));
    }
    
    // Send all chord notes
    for (i, message) in messages.iter().enumerate() {
        if let Err(e) = client.send_message(message).await {
            eprintln!("Failed to send chord note {}: {}", i + 1, e);
        } else {
            println!("Sent chord note {}: {:.2}Hz", i + 1, chord_frequencies[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = OpenSoundClient::new(OpenSoundConfig::default());
        assert!(!client.is_tcp_connected());
        assert!(!client.is_udp_connected());
    }

    #[test]
    fn test_create_oscillator_message() {
        let message = OpenSoundClient::create_oscillator_message(440.0, 0.5, 0.0, 1000.0, "sine");
        
        assert_eq!(message.address_pattern, "/note/oscillator");
        assert_eq!(message.arguments.len(), 5);
    }
} 