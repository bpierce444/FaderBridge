#!/usr/bin/env python3
"""
full_monitor.py - Monitor ALL non-audio endpoints on Quantum HD 2

Captures from:
- Interface 0, EP 0x84 (Interrupt) - Input channel status
- Interface 4, EP 0x82 (Bulk/MIDI) - Possibly MIDI or control
- Interface 5, EP 0x81 (Bulk) - Control interface

Also tries sending queries to Interface 5 to see if that triggers responses.
"""

import usb.core
import usb.util
import sys
import time
import threading
from collections import defaultdict

VENDOR_ID = 0x1ed8
PRODUCT_ID = 0x020e

# Track state
last_data = defaultdict(lambda: None)
lock = threading.Lock()


def format_hex(data):
    """Format bytes as hex with spaces."""
    return ' '.join(f'{b:02x}' for b in data)


def monitor_endpoint(dev, interface, endpoint, name, duration=60):
    """Monitor a single endpoint."""
    global last_data
    
    print(f"[{name}] Starting monitor...")
    start = time.time()
    count = 0
    
    while time.time() - start < duration:
        try:
            data = dev.read(endpoint, 512, timeout=100)
            data_bytes = bytes(data)
            
            with lock:
                if data_bytes != last_data[name]:
                    elapsed = time.time() - start
                    print(f"[{elapsed:>7.2f}s] [{name:<20}] {format_hex(data_bytes)}")
                    last_data[name] = data_bytes
                    count += 1
                    
        except usb.core.USBError:
            pass  # Timeout
    
    print(f"[{name}] Done - {count} unique messages")


def try_control_queries(dev, duration=60):
    """Periodically send queries on Interface 5 to try to get responses."""
    print("[Control Query] Starting...")
    
    # Claim interface 5 with alt setting 1
    try:
        if dev.is_kernel_driver_active(5):
            dev.detach_kernel_driver(5)
        usb.util.claim_interface(dev, 5)
        dev.set_interface_altsetting(interface=5, alternate_setting=1)
    except Exception as e:
        print(f"[Control Query] Setup error: {e}")
        return
    
    start = time.time()
    queries_sent = 0
    
    # Different query patterns to try
    queries = [
        bytes([0x00, 0x00, 0x00, 0x00]),  # Empty
        bytes([0x01, 0x00, 0x00, 0x00]),  # Query type 1?
        bytes([0x02, 0x00, 0x00, 0x00]),  # Query type 2?
        bytes([0x00, 0x01, 0x00, 0x00]),  # Alt format
        bytes([0x55, 0x43, 0x00, 0x01]),  # UCNet magic
    ]
    
    query_idx = 0
    
    while time.time() - start < duration:
        # Send a query every 5 seconds
        if int(time.time() - start) % 5 == 0 and int(time.time() - start) > queries_sent * 5:
            query = queries[query_idx % len(queries)]
            try:
                dev.write(0x01, query, timeout=100)
                elapsed = time.time() - start
                print(f"[{elapsed:>7.2f}s] [Control Query      ] SENT: {format_hex(query)}")
                queries_sent += 1
                query_idx += 1
            except usb.core.USBError as e:
                pass
        
        # Try to read response
        try:
            data = dev.read(0x81, 512, timeout=100)
            data_bytes = bytes(data)
            elapsed = time.time() - start
            with lock:
                print(f"[{elapsed:>7.2f}s] [Control Response   ] {format_hex(data_bytes)}")
        except usb.core.USBError:
            pass
        
        time.sleep(0.05)
    
    usb.util.release_interface(dev, 5)
    print("[Control Query] Done")


def main():
    print("=" * 70)
    print("Quantum HD 2 Full Monitor")
    print("=" * 70)
    print()
    print("Monitoring for 60 seconds. Adjust ALL parameters on the Quantum:")
    print("  - Input gains (Ch1, Ch2)")
    print("  - 48V, HPF, Pad toggles")
    print("  - Main output level")
    print("  - Headphone level")
    print("  - Sample rate")
    print("  - Any other settings")
    print()
    
    dev = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if dev is None:
        print("Device not found")
        sys.exit(1)
    
    print(f"Found: {dev.product}\n")
    
    # Detach all kernel drivers
    for i in range(7):
        try:
            if dev.is_kernel_driver_active(i):
                dev.detach_kernel_driver(i)
        except:
            pass
    
    # Claim interfaces we need
    for i in [0, 4]:
        try:
            usb.util.claim_interface(dev, i)
        except:
            pass
    
    print("-" * 70)
    print(f"{'Time':<10} {'Endpoint':<22} {'Data'}")
    print("-" * 70)
    
    # Start monitor threads
    threads = []
    
    # Interface 0, EP 0x84 - Interrupt (input channel status)
    t1 = threading.Thread(target=monitor_endpoint, args=(dev, 0, 0x84, "Int0-Status", 60))
    t1.daemon = True
    threads.append(t1)
    
    # Interface 4, EP 0x82 - Bulk (MIDI?)
    t2 = threading.Thread(target=monitor_endpoint, args=(dev, 4, 0x82, "Int4-MIDI", 60))
    t2.daemon = True
    threads.append(t2)
    
    # Interface 5 - Control (send queries and listen)
    t3 = threading.Thread(target=try_control_queries, args=(dev, 60))
    t3.daemon = True
    threads.append(t3)
    
    # Start all threads
    for t in threads:
        t.start()
    
    # Wait for completion
    for t in threads:
        t.join()
    
    print("-" * 70)
    print("\nMonitoring complete!")
    print("\nSummary of discovered parameters:")
    print("  Byte2 = Channel (01=Ch1, 02=Ch2)")
    print("  Byte3 = Parameter:")
    print("    0x02 = Gain")
    print("    0xe2 = 48V Phantom")
    print("    0xe3 = HPF")
    print("    0xe4 = Pad")


if __name__ == "__main__":
    main()
