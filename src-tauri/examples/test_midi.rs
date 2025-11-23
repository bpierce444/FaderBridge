// Test MIDI enumeration on macOS
use midir::{MidiInput, MidiOutput};

fn main() {
    println!("=== Testing MIDI Device Enumeration ===\n");
    
    // Test MIDI Inputs
    println!("MIDI Inputs:");
    match MidiInput::new("Test-Input") {
        Ok(midi_in) => {
            let ports = midi_in.ports();
            println!("Found {} input ports", ports.len());
            for (i, port) in ports.iter().enumerate() {
                match midi_in.port_name(port) {
                    Ok(name) => println!("  [{}] {}", i, name),
                    Err(e) => println!("  [{}] ERROR: {}", i, e),
                }
            }
        }
        Err(e) => println!("Failed to create MIDI input: {}", e),
    }
    
    println!("\nMIDI Outputs:");
    match MidiOutput::new("Test-Output") {
        Ok(midi_out) => {
            let ports = midi_out.ports();
            println!("Found {} output ports", ports.len());
            for (i, port) in ports.iter().enumerate() {
                match midi_out.port_name(port) {
                    Ok(name) => println!("  [{}] {}", i, name),
                    Err(e) => println!("  [{}] ERROR: {}", i, e),
                }
            }
        }
        Err(e) => println!("Failed to create MIDI output: {}", e),
    }
}
