#!/usr/bin/env python3
"""
test_quantum_usb_v2.py - Improved USB communication test for Quantum HD 2

Fixes:
- Properly sets alternate interface setting
- Tries different initialization approaches
- Better error handling

Usage:
    sudo python3 test_quantum_usb_v2.py
"""

import usb.core
import usb.util
import struct
import sys
import time

# Quantum HD 2 USB IDs
VENDOR_ID = 0x1ed8   # PreSonus/Fender
PRODUCT_ID = 0x020e  # Quantum HD 2
CONTROL_INTERFACE = 5  # "Quantum HD 2 Control" interface

# UCNet protocol constants
MAGIC_BYTES = bytes([0x55, 0x43, 0x00, 0x01])  # "UC\x00\x01"

# Payload types
PAYLOAD_HELLO = bytes([0x55, 0x4D])      # "UM"
PAYLOAD_JSON = bytes([0x4A, 0x4D])       # "JM"
PAYLOAD_KEEP_ALIVE = bytes([0x4B, 0x41]) # "KA"


def find_device():
    """Find and return the Quantum HD 2 device."""
    dev = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if dev is None:
        print("ERROR: Quantum HD 2 not found")
        sys.exit(1)
    
    print(f"Found: {dev.manufacturer} {dev.product}")
    print(f"  Serial: {dev.serial_number}")
    print(f"  Bus: {dev.bus}, Address: {dev.address}")
    return dev


def reset_device(dev):
    """Reset the USB device."""
    print("Resetting device...")
    try:
        dev.reset()
        time.sleep(1)
        print("Device reset complete")
        return True
    except usb.core.USBError as e:
        print(f"Reset failed: {e}")
        return False


def detach_all_kernel_drivers(dev):
    """Detach kernel drivers from all interfaces."""
    print("Detaching kernel drivers...")
    for cfg in dev:
        for intf in cfg:
            intf_num = intf.bInterfaceNumber
            try:
                if dev.is_kernel_driver_active(intf_num):
                    dev.detach_kernel_driver(intf_num)
                    print(f"  Detached driver from interface {intf_num}")
            except usb.core.USBError as e:
                print(f"  Could not detach interface {intf_num}: {e}")


def setup_interface(dev, interface_num, alt_setting=0):
    """Set up an interface with specific alternate setting."""
    print(f"Setting up interface {interface_num}, alt setting {alt_setting}...")
    
    # Detach kernel driver
    try:
        if dev.is_kernel_driver_active(interface_num):
            dev.detach_kernel_driver(interface_num)
            print(f"  Detached kernel driver")
    except usb.core.USBError as e:
        print(f"  Kernel driver note: {e}")
    
    # Claim interface
    try:
        usb.util.claim_interface(dev, interface_num)
        print(f"  Claimed interface")
    except usb.core.USBError as e:
        print(f"  Claim error: {e}")
        return False
    
    # Set alternate setting
    try:
        dev.set_interface_altsetting(interface=interface_num, alternate_setting=alt_setting)
        print(f"  Set alternate setting {alt_setting}")
    except usb.core.USBError as e:
        print(f"  Alt setting error: {e}")
    
    return True


def get_endpoints(dev, interface_num, alt_setting):
    """Get endpoints for a specific interface and alternate setting."""
    cfg = dev.get_active_configuration()
    intf = cfg[(interface_num, alt_setting)]
    
    ep_out = None
    ep_in = None
    
    for ep in intf:
        if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_OUT:
            ep_out = ep
        else:
            ep_in = ep
    
    return ep_out, ep_in


def try_control_transfer(dev):
    """Try USB control transfers to query device."""
    print("\n--- Testing Control Transfers ---")
    
    # Standard USB requests
    requests = [
        ("GET_STATUS", 0x80, 0x00, 0, 0, 2),
        ("GET_DESCRIPTOR (Device)", 0x80, 0x06, 0x0100, 0, 18),
        ("GET_DESCRIPTOR (Config)", 0x80, 0x06, 0x0200, 0, 255),
        # Vendor-specific requests (guessing common patterns)
        ("Vendor IN 0x01", 0xC0, 0x01, 0, 0, 64),
        ("Vendor IN 0x10", 0xC0, 0x10, 0, 0, 64),
        ("Vendor IN 0x20", 0xC0, 0x20, 0, 0, 64),
        ("Vendor IN 0x80", 0xC0, 0x80, 0, 0, 64),
        ("Vendor IN 0xFF", 0xC0, 0xFF, 0, 0, 64),
    ]
    
    for name, bmRequestType, bRequest, wValue, wIndex, wLength in requests:
        try:
            result = dev.ctrl_transfer(bmRequestType, bRequest, wValue, wIndex, wLength, timeout=500)
            print(f"  {name}: {bytes(result).hex()}")
        except usb.core.USBError as e:
            print(f"  {name}: {e}")


def try_bulk_communication(dev, ep_out, ep_in):
    """Try various bulk communication patterns."""
    print("\n--- Testing Bulk Communication ---")
    
    # Test patterns to try
    patterns = [
        ("UCNet Hello", MAGIC_BYTES + PAYLOAD_HELLO + bytes([0x6a, 0x00, 0x65, 0x00, 0x00, 0x00])),
        ("UCNet KA", MAGIC_BYTES + PAYLOAD_KEEP_ALIVE + bytes([0x6a, 0x00, 0x65, 0x00, 0x00, 0x00])),
        ("Empty", bytes([0x00] * 8)),
        ("All 0xFF", bytes([0xFF] * 8)),
        ("Simple ping", bytes([0x01, 0x00, 0x00, 0x00])),
        ("Query?", bytes([0x00, 0x01, 0x00, 0x00])),
    ]
    
    for name, data in patterns:
        print(f"\n  Trying '{name}': {data.hex()}")
        try:
            written = ep_out.write(data, timeout=500)
            print(f"    Sent {written} bytes")
            
            # Try to read response
            try:
                response = ep_in.read(512, timeout=500)
                print(f"    Response: {bytes(response).hex()}")
            except usb.core.USBError as e:
                print(f"    No response: {e}")
                
        except usb.core.USBError as e:
            print(f"    Send failed: {e}")


def try_interrupt_endpoint(dev):
    """Try reading from the interrupt endpoint (interface 0)."""
    print("\n--- Testing Interrupt Endpoint (Interface 0) ---")
    
    try:
        # Interface 0 has interrupt endpoint 0x84
        if dev.is_kernel_driver_active(0):
            dev.detach_kernel_driver(0)
        usb.util.claim_interface(dev, 0)
        
        cfg = dev.get_active_configuration()
        intf = cfg[(0, 0)]
        
        ep_in = None
        for ep in intf:
            if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                ep_in = ep
                break
        
        if ep_in:
            print(f"  Reading from endpoint 0x{ep_in.bEndpointAddress:02x}...")
            for i in range(5):
                try:
                    data = ep_in.read(ep_in.wMaxPacketSize, timeout=500)
                    print(f"    [{i}] {bytes(data).hex()}")
                except usb.core.USBError as e:
                    print(f"    [{i}] {e}")
        
        usb.util.release_interface(dev, 0)
    except usb.core.USBError as e:
        print(f"  Error: {e}")


def try_midi_interface(dev):
    """Try the MIDI interface (interface 4)."""
    print("\n--- Testing MIDI Interface (Interface 4) ---")
    
    try:
        if dev.is_kernel_driver_active(4):
            dev.detach_kernel_driver(4)
        usb.util.claim_interface(dev, 4)
        
        cfg = dev.get_active_configuration()
        intf = cfg[(4, 0)]
        
        ep_in = None
        ep_out = None
        for ep in intf:
            if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                ep_in = ep
            else:
                ep_out = ep
        
        if ep_in:
            print(f"  Reading MIDI from endpoint 0x{ep_in.bEndpointAddress:02x}...")
            print("  (Move a fader on the Quantum if it has physical controls)")
            for i in range(10):
                try:
                    data = ep_in.read(512, timeout=500)
                    print(f"    [{i}] MIDI: {bytes(data).hex()}")
                except usb.core.USBError:
                    pass  # Timeout, no data
        
        usb.util.release_interface(dev, 4)
    except usb.core.USBError as e:
        print(f"  Error: {e}")


def main():
    print("=" * 60)
    print("Quantum HD 2 USB Test v2")
    print("=" * 60)
    
    # Find device
    dev = find_device()
    
    # Option: Reset device first
    print("\n" + "=" * 60)
    print("Step 1: Device Reset")
    print("=" * 60)
    reset_device(dev)
    
    # Re-find after reset
    dev = find_device()
    
    # Detach all kernel drivers
    print("\n" + "=" * 60)
    print("Step 2: Detach Kernel Drivers")
    print("=" * 60)
    detach_all_kernel_drivers(dev)
    
    # Try control transfers first
    print("\n" + "=" * 60)
    print("Step 3: Control Transfers")
    print("=" * 60)
    try_control_transfer(dev)
    
    # Try interrupt endpoint
    print("\n" + "=" * 60)
    print("Step 4: Interrupt Endpoint")
    print("=" * 60)
    try_interrupt_endpoint(dev)
    
    # Try MIDI interface
    print("\n" + "=" * 60)
    print("Step 5: MIDI Interface")
    print("=" * 60)
    try_midi_interface(dev)
    
    # Try control interface with alt setting 0
    print("\n" + "=" * 60)
    print("Step 6: Control Interface (Alt 0)")
    print("=" * 60)
    if setup_interface(dev, CONTROL_INTERFACE, alt_setting=0):
        ep_out, ep_in = get_endpoints(dev, CONTROL_INTERFACE, 0)
        if ep_out and ep_in:
            print(f"  Endpoints: OUT=0x{ep_out.bEndpointAddress:02x}, IN=0x{ep_in.bEndpointAddress:02x}")
        else:
            print("  No endpoints in alt setting 0")
        usb.util.release_interface(dev, CONTROL_INTERFACE)
    
    # Try control interface with alt setting 1
    print("\n" + "=" * 60)
    print("Step 7: Control Interface (Alt 1)")
    print("=" * 60)
    if setup_interface(dev, CONTROL_INTERFACE, alt_setting=1):
        ep_out, ep_in = get_endpoints(dev, CONTROL_INTERFACE, 1)
        if ep_out and ep_in:
            print(f"  Endpoints: OUT=0x{ep_out.bEndpointAddress:02x}, IN=0x{ep_in.bEndpointAddress:02x}")
            try_bulk_communication(dev, ep_out, ep_in)
        usb.util.release_interface(dev, CONTROL_INTERFACE)
    
    print("\n" + "=" * 60)
    print("Test Complete")
    print("=" * 60)


if __name__ == "__main__":
    main()
