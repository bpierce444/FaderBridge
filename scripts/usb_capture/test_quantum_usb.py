#!/usr/bin/env python3
"""
test_quantum_usb.py - Test USB communication with Quantum HD 2

This script attempts to communicate with the Quantum HD 2's control interface
using the UCNet protocol format. Run this in a Linux VM with the device
passed through.

Usage:
    sudo python3 test_quantum_usb.py

Requirements:
    pip install pyusb
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
PAYLOAD_PARAM_VALUE = bytes([0x50, 0x56]) # "PV"
PAYLOAD_KEEP_ALIVE = bytes([0x4B, 0x41])  # "KA"


def find_device():
    """Find and return the Quantum HD 2 device."""
    dev = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if dev is None:
        print("ERROR: Quantum HD 2 not found")
        print("Make sure:")
        print("  1. Device is connected")
        print("  2. Device is passed through to VM")
        print("  3. ucdaemon is stopped on macOS host")
        sys.exit(1)
    
    print(f"Found: {dev.manufacturer} {dev.product}")
    print(f"  Serial: {dev.serial_number}")
    print(f"  Bus: {dev.bus}, Address: {dev.address}")
    return dev


def list_interfaces(dev):
    """List all interfaces on the device."""
    print("\nInterfaces:")
    for cfg in dev:
        print(f"  Configuration {cfg.bConfigurationValue}")
        for intf in cfg:
            class_name = {
                1: "Audio",
                255: "Vendor-specific",
                254: "Application-specific (DFU)"
            }.get(intf.bInterfaceClass, f"Class {intf.bInterfaceClass}")
            
            print(f"    Interface {intf.bInterfaceNumber}: {class_name}")
            for ep in intf:
                direction = "IN" if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else "OUT"
                ep_type = {
                    usb.util.ENDPOINT_TYPE_BULK: "Bulk",
                    usb.util.ENDPOINT_TYPE_INTR: "Interrupt",
                    usb.util.ENDPOINT_TYPE_ISO: "Isochronous",
                    usb.util.ENDPOINT_TYPE_CTRL: "Control"
                }.get(usb.util.endpoint_type(ep.bmAttributes), "Unknown")
                print(f"      Endpoint 0x{ep.bEndpointAddress:02x}: {direction} {ep_type} (max {ep.wMaxPacketSize} bytes)")


def claim_control_interface(dev):
    """Claim the control interface."""
    # Detach kernel driver if attached
    try:
        if dev.is_kernel_driver_active(CONTROL_INTERFACE):
            print(f"Detaching kernel driver from interface {CONTROL_INTERFACE}")
            dev.detach_kernel_driver(CONTROL_INTERFACE)
    except usb.core.USBError as e:
        print(f"Warning: Could not detach kernel driver: {e}")
    
    # Set configuration (use first configuration)
    try:
        dev.set_configuration()
    except usb.core.USBError as e:
        print(f"Warning: Could not set configuration: {e}")
    
    # Claim interface
    try:
        usb.util.claim_interface(dev, CONTROL_INTERFACE)
        print(f"Claimed interface {CONTROL_INTERFACE}")
    except usb.core.USBError as e:
        print(f"ERROR: Could not claim interface: {e}")
        sys.exit(1)
    
    return True


def get_endpoints(dev):
    """Get IN and OUT endpoints for the control interface."""
    cfg = dev.get_active_configuration()
    intf = cfg[(CONTROL_INTERFACE, 1)]  # Interface 5, alternate setting 1
    
    ep_out = usb.util.find_descriptor(
        intf,
        custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress) == usb.util.ENDPOINT_OUT
    )
    
    ep_in = usb.util.find_descriptor(
        intf,
        custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress) == usb.util.ENDPOINT_IN
    )
    
    if ep_out is None or ep_in is None:
        print("ERROR: Could not find endpoints")
        sys.exit(1)
    
    print(f"OUT endpoint: 0x{ep_out.bEndpointAddress:02x}")
    print(f"IN endpoint: 0x{ep_in.bEndpointAddress:02x}")
    
    return ep_out, ep_in


def build_ucnet_packet(payload_type, payload, c_bytes=None):
    """Build a UCNet protocol packet."""
    if c_bytes is None:
        c_bytes = bytes([0x6A, 0x00, 0x65, 0x00])  # Default: 'j', 0, 'e', 0
    
    if isinstance(payload, str):
        payload = payload.encode('utf-8')
    
    length = struct.pack('<H', len(payload))
    
    packet = MAGIC_BYTES + payload_type + c_bytes + length + payload
    return packet


def send_hello(ep_out, ep_in):
    """Send a UCNet Hello packet and wait for response."""
    print("\n--- Sending Hello (UM) ---")
    
    hello_payload = '{"id":"FaderBridge","version":"1.0"}'
    packet = build_ucnet_packet(PAYLOAD_HELLO, hello_payload)
    
    print(f"Packet ({len(packet)} bytes): {packet.hex()}")
    print(f"  Magic: {packet[:4].hex()}")
    print(f"  Type: {packet[4:6].hex()} ({packet[4:6]})")
    print(f"  C-bytes: {packet[6:10].hex()}")
    print(f"  Length: {struct.unpack('<H', packet[10:12])[0]}")
    print(f"  Payload: {packet[12:]}")
    
    try:
        written = ep_out.write(packet, timeout=1000)
        print(f"Sent {written} bytes")
    except usb.core.USBError as e:
        print(f"Send error: {e}")
        return None
    
    # Wait for response
    try:
        response = ep_in.read(512, timeout=2000)
        response_bytes = bytes(response)
        print(f"\nResponse ({len(response_bytes)} bytes): {response_bytes.hex()}")
        
        # Try to parse as UCNet
        if response_bytes[:4] == MAGIC_BYTES:
            print("  Valid UCNet response!")
            print(f"  Type: {response_bytes[4:6].hex()}")
        else:
            print("  Not a UCNet packet (different magic bytes)")
            
        return response_bytes
    except usb.core.USBError as e:
        print(f"Read error (timeout?): {e}")
        return None


def send_keep_alive(ep_out, ep_in):
    """Send a Keep-Alive packet."""
    print("\n--- Sending Keep-Alive (KA) ---")
    
    packet = build_ucnet_packet(PAYLOAD_KEEP_ALIVE, b'')
    print(f"Packet ({len(packet)} bytes): {packet.hex()}")
    
    try:
        written = ep_out.write(packet, timeout=1000)
        print(f"Sent {written} bytes")
    except usb.core.USBError as e:
        print(f"Send error: {e}")
        return None
    
    try:
        response = ep_in.read(512, timeout=2000)
        print(f"Response: {bytes(response).hex()}")
        return bytes(response)
    except usb.core.USBError as e:
        print(f"Read error: {e}")
        return None


def listen_for_traffic(ep_in, duration=10):
    """Listen for incoming traffic for a specified duration."""
    print(f"\n--- Listening for traffic ({duration}s) ---")
    print("Move faders on the Quantum or in Universal Control...")
    
    start = time.time()
    packets = []
    
    while time.time() - start < duration:
        try:
            data = ep_in.read(512, timeout=500)
            data_bytes = bytes(data)
            elapsed = time.time() - start
            print(f"[{elapsed:.2f}s] Received {len(data_bytes)} bytes: {data_bytes[:32].hex()}...")
            packets.append(data_bytes)
        except usb.core.USBError:
            # Timeout, continue listening
            pass
    
    print(f"\nCaptured {len(packets)} packets")
    return packets


def raw_read_test(ep_in, count=10):
    """Try to read raw data from the device."""
    print(f"\n--- Raw read test ({count} attempts) ---")
    
    for i in range(count):
        try:
            data = ep_in.read(512, timeout=1000)
            print(f"[{i}] Received {len(data)} bytes: {bytes(data).hex()}")
        except usb.core.USBError as e:
            print(f"[{i}] Timeout or error: {e}")


def main():
    print("=" * 60)
    print("Quantum HD 2 USB Control Interface Test")
    print("=" * 60)
    
    # Find device
    dev = find_device()
    
    # List interfaces
    list_interfaces(dev)
    
    # Claim control interface
    print(f"\n--- Claiming interface {CONTROL_INTERFACE} ---")
    claim_control_interface(dev)
    
    # Get endpoints
    ep_out, ep_in = get_endpoints(dev)
    
    # Test 1: Try to read any incoming data first
    print("\n" + "=" * 60)
    print("Test 1: Raw read (checking for unsolicited data)")
    print("=" * 60)
    raw_read_test(ep_in, count=5)
    
    # Test 2: Send Hello
    print("\n" + "=" * 60)
    print("Test 2: UCNet Hello packet")
    print("=" * 60)
    send_hello(ep_out, ep_in)
    
    # Test 3: Send Keep-Alive
    print("\n" + "=" * 60)
    print("Test 3: UCNet Keep-Alive packet")
    print("=" * 60)
    send_keep_alive(ep_out, ep_in)
    
    # Test 4: Listen for traffic
    print("\n" + "=" * 60)
    print("Test 4: Listen for device-initiated traffic")
    print("=" * 60)
    listen_for_traffic(ep_in, duration=10)
    
    # Cleanup
    print("\n--- Cleanup ---")
    usb.util.release_interface(dev, CONTROL_INTERFACE)
    print("Released interface")
    
    print("\n" + "=" * 60)
    print("Test complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()
