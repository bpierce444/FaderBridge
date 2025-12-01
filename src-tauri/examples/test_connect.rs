use midir::{MidiInput, Ignore};

fn main() {
    println!("=== Test 1: Enumerate then connect ===\n");
    
    // First enumeration (simulates app startup)
    println!("Step 1: First enumeration...");
    {
        let midi_in = MidiInput::new("FaderBridge").expect("Failed to create MIDI input");
        let ports = midi_in.ports();
        println!("Found {} ports:", ports.len());
        for (i, port) in ports.iter().enumerate() {
            let name = midi_in.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
            println!("  [{}] {}", i, name);
        }
        // midi_in is dropped here
    }
    
    println!("\nStep 2: Second enumeration (simulates cached return)...");
    {
        let midi_in = MidiInput::new("FaderBridge").expect("Failed to create MIDI input");
        let ports = midi_in.ports();
        println!("Found {} ports:", ports.len());
        for (i, port) in ports.iter().enumerate() {
            let name = midi_in.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
            println!("  [{}] {}", i, name);
        }
        // midi_in is dropped here
    }
    
    println!("\nStep 3: Try to connect to port 2 (PreSonus FP2)...");
    {
        let mut midi_in = MidiInput::new("FaderBridge").expect("Failed to create MIDI input");
        midi_in.ignore(Ignore::None);
        
        let ports = midi_in.ports();
        println!("Found {} ports", ports.len());
        
        if ports.len() > 2 {
            let port = &ports[2];
            let name = midi_in.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
            println!("Port 2 name: {}", name);
            
            println!("Attempting connection...");
            match midi_in.connect(port, "FaderBridge", |stamp, message, _| {
                println!("MIDI: {} {:?}", stamp, message);
            }, ()) {
                Ok(conn) => {
                    println!("SUCCESS! Connected to port 2");
                    println!("Move a fader to see messages (waiting 3 seconds)...");
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    drop(conn);
                    println!("Disconnected.");
                }
                Err(e) => {
                    println!("FAILED to connect: {}", e);
                }
            }
        } else {
            println!("Not enough ports!");
        }
    }
    
    println!("\nDone.");
}
