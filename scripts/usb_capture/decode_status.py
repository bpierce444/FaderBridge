#!/usr/bin/env python3
"""
decode_status.py - Decode and log status messages from Quantum HD 2

Monitors Interface 0, EP 0x84 and tries to decode the 6-byte status messages.
"""

import usb.core
import usb.util
import sys
import time
from datetime import datetime

VENDOR_ID = 0x1ed8
PRODUCT_ID = 0x020e

# Track previous state to only show changes
last_data = None


def decode_status(data):
    """Try to decode the 6-byte status message."""
    if len(data) != 6:
        return f"Unknown format ({len(data)} bytes)"
    
    # Parse bytes
    b0, b1, b2, b3, b4, b5 = data
    
    # Hypothesis based on observed patterns:
    # 00 01 XX YY ZZ WW
    # b0, b1 = header (always 00 01?)
    # b2 = some selector/channel? (01, 02 observed)
    # b3 = value or type? (02, e2 observed)
    # b4, b5 = additional data (02 00, 20 00 observed)
    
    parts = []
    parts.append(f"Header: {b0:02x} {b1:02x}")
    parts.append(f"Byte2: {b2:02x} (dec: {b2})")
    parts.append(f"Byte3: {b3:02x} (dec: {b3})")
    parts.append(f"Byte4: {b4:02x} (dec: {b4})")
    parts.append(f"Byte5: {b5:02x} (dec: {b5})")
    
    # Try to interpret
    interpretation = ""
    if b3 == 0x02:
        interpretation = "Normal state?"
    elif b3 == 0xe2:
        interpretation = "Special state? (0xe2 = 226)"
    
    if b2 == 0x01:
        interpretation += " Channel/Select 1"
    elif b2 == 0x02:
        interpretation += " Channel/Select 2"
    
    return " | ".join(parts) + f" [{interpretation}]"


def main():
    global last_data
    
    print("=" * 70)
    print("Quantum HD 2 Status Decoder")
    print("=" * 70)
    
    dev = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if dev is None:
        print("Device not found")
        sys.exit(1)
    
    print(f"Found: {dev.product}")
    
    # Detach kernel driver from interface 0
    try:
        if dev.is_kernel_driver_active(0):
            dev.detach_kernel_driver(0)
    except:
        pass
    
    # Claim interface 0
    usb.util.claim_interface(dev, 0)
    
    print("\nListening on Interface 0, EP 0x84...")
    print("Adjust parameters on the Quantum HD 2 and watch for changes.")
    print("Press Ctrl+C to stop.\n")
    print("-" * 70)
    print(f"{'Time':<12} {'Raw Hex':<15} {'Decoded'}")
    print("-" * 70)
    
    start = time.time()
    
    try:
        while True:
            try:
                data = dev.read(0x84, 6, timeout=100)
                data_bytes = bytes(data)
                
                # Only print if data changed
                if data_bytes != last_data:
                    elapsed = time.time() - start
                    hex_str = data_bytes.hex()
                    decoded = decode_status(data_bytes)
                    
                    print(f"{elapsed:>10.2f}s  {hex_str:<15} {decoded}")
                    last_data = data_bytes
                    
            except usb.core.USBError:
                pass  # Timeout
                
    except KeyboardInterrupt:
        print("\n" + "-" * 70)
        print("Stopped")
    
    usb.util.release_interface(dev, 0)


if __name__ == "__main__":
    main()
