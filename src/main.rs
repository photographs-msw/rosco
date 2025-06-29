extern crate derive_builder;

mod audio_gen;
mod common;
mod effect;
mod envelope;
mod midi;
mod note;
mod sequence;
mod track;
mod composition;
mod meter;
mod dsl;
mod compositions;
mod open_sound;

use crate::compositions::dsl_1;
// use crate::compositions::computer_punk_001;
// use crate::compositions::computer_punk_003;

// OpenSound Protocol examples
use crate::open_sound::{create_musical_server, create_client, create_integration};
use crate::open_sound::{server_example, client_example};

#[tokio::main]
async fn main() {
    println!("=== OpenSound Protocol Demo ===");
    println!("Choose an example to run:");
    println!("1. DSL Composition (original)");
    println!("2. OpenSound Server");
    println!("3. OpenSound Client");
    println!("4. OpenSound Integration");
    println!("5. Run both server and client");
    
    // For now, run the DSL composition as default
    // In a real application, you might want to add command line arguments
    // to choose which example to run
    
    // println!("\nRunning DSL composition...");
    // dsl_1::play();
    
    // Uncomment the following lines to run OpenSound examples:
    
    println!("\nRunning OpenSound server...");
    server_example::run_server().await;
    
    println!("\nRunning OpenSound client...");
    client_example::run_client().await;
    
    println!("\nRunning OpenSound integration...");
    let mut integration = create_integration();
    integration.queue_oscillator_note(440.0, 0.5, 0.0, 1000.0, "sine").unwrap();
    integration.play_queued_notes().unwrap();
}
