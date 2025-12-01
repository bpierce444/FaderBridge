use midir::{MidiInput, MidiOutput, Ignore};

fn main() {
    println!("=== Test: Simulate app behavior ===\n");
    
    // Step 1: Enumerate inputs (like discover_inputs)
    println!("Step 1: Enumerate inputs...");
    let fp2_port_at_enum;
    {
        let midi_in = MidiInput::new("FaderBridge").expect("Failed");
        let ports = midi_in.ports();
        println!("Found {} input ports", ports.len());
        
        fp2_port_at_enum = ports.iter().enumerate().find_map(|(i, p)| {
            let name = midi_in.port_name(p).ok()?;
            println!("  [{}] {}", i, name);
            if name == "PreSonus FP2" { Some(i) } else { None }
        });
        println!("FP2 found at port: {:?}", fp2_port_at_enum);
    }
    
    // Step 2: Enumerate outputs (like discover_outputs)
    println!("\nStep 2: Enumerate outputs...");
    {
        let midi_out = MidiOutput::new("FaderBridge").expect("Failed");
        let ports = midi_out.ports();
        println!("Found {} output ports", ports.len());
        for (i, port) in ports.iter().enumerate() {
            let name = midi_out.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
            println!("  [{}] {}", i, name);
        }
    }
    
    // Step 3: Now try to connect using the port number we found
    println!("\nStep 3: Connect using cached port number {}...", fp2_port_at_enum.unwrap_or(999));
    {
        let mut midi_in = MidiInput::new("FaderBridge").expect("Failed");
        midi_in.ignore(Ignore::None);
        
        let ports = midi_in.ports();
        println!("Found {} ports now", ports.len());
        
        // Show what's at each port now
        for (i, port) in ports.iter().enumerate() {
            let name = midi_in.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
            println!("  [{}] {}", i, name);
        }
        
        if let Some(port_num) = fp2_port_at_enum {
            if port_num < ports.len() {
                let port = &ports[port_num];
                let name = midi_in.port_name(port).unwrap_or_else(|_| "ERROR".to_string());
                println!("\nPort {} is now: {}", port_num, name);
                
                // Try to find FP2 by name instead
                let fp2_port = ports.iter().find(|p| {
                    midi_in.port_name(p).map(|n| n == "PreSonus FP2").unwrap_or(false)
                });
                
                if let Some(fp2) = fp2_port {
                    println!("Found FP2 by name, connecting...");
                    match midi_in.connect(fp2, "FaderBridge", |stamp, message, _| {
                        println!("MIDI: {} {:?}", stamp, message);
                    }, ()) {
                        Ok(_conn) => {
                            println!("SUCCESS!");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                        Err(e) => println!("FAILED: {}", e),
                    }
                } else {
                    println!("Could not find FP2 by name!");
                }
            }
        }
    }
    
    println!("\nDone.");
}
