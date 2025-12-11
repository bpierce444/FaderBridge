#!/usr/bin/env python3
"""
UCNet Control Test for StudioLive Mixers

Connects directly to a network-connected StudioLive mixer via UCNet (TCP port 53000),
performs the handshake, and sends fader control commands.

Usage:
    python3 test_mixer_control.py <mixer_ip> [value]
    
Examples:
    python3 test_mixer_control.py 192.168.1.209 0.5   # Set main fader to 50%
    python3 test_mixer_control.py 192.168.1.209 0.1   # Set main fader to 10%

Key Protocol Details:
    - Size field is little-endian and includes the 2-byte type
    - Subscribe must use clientType="Mac", clientInternalName="ucapp"
    - CBytes for Subscribe and PV packets: 0x72 0x00 0x65 0x00 ('r', 'e')
    - Parameter values are linear gain (0.0-1.0), not dB
"""

import socket
import struct
import sys
import time
import json

MAGIC = b'\x55\x43\x00\x01'
TYPE_HELLO = b'\x55\x4d'
TYPE_JSON = b'\x4a\x4d'
TYPE_PV = b'\x50\x56'
TYPE_KA = b'\x4b\x41'


def build_packet(ptype: bytes, payload: bytes) -> bytes:
    """Build UCNet packet with little-endian size including type."""
    size = struct.pack('<H', 2 + len(payload))
    return MAGIC + size + ptype + payload


def build_hello() -> bytes:
    return build_packet(TYPE_HELLO, b'\x00\x00\x65\x00\x15\xfa')


def build_subscribe() -> bytes:
    # Match UC's exact Subscribe format
    data = {
        "id": "Subscribe",
        "clientName": "Universal Control",  # Match UC
        "clientInternalName": "ucapp",  # Match UC
        "clientType": "Mac",  # Match UC
        "clientDescription": "FaderBridge",
        "clientIdentifier": "FaderBridge",
        "clientOptions": "perm users levl redu rtan",
        "clientEncoding": 23106,
    }
    json_bytes = json.dumps(data).encode()
    # UC uses r e (0x72 0x65) for Subscribe, not j e
    cbytes = b'\x72\x00\x65\x00'  # r e - what UC uses
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(TYPE_JSON, payload)


def build_pv(key: str, value: float, cbytes: bytes = b'\x72\x00\x65\x00') -> bytes:
    """Build ParameterValue packet."""
    key_bytes = key.encode('ascii') + b'\x00'
    padding = b'\x00\x00'
    val_bytes = struct.pack('<f', value)
    payload = cbytes + key_bytes + padding + val_bytes
    return build_packet(TYPE_PV, payload)


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 test_mixer_control.py <mixer_ip> [value]")
        return
    
    host = sys.argv[1]
    value = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
    value = max(0.0, min(1.0, value))
    
    print(f"Connecting to {host}:53000...")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    sock.connect((host, 53000))
    print("Connected!")
    
    try:
        # Send Hello
        sock.sendall(build_hello())
        print("Sent Hello")
        time.sleep(0.2)
        
        # Send Subscribe
        sock.sendall(build_subscribe())
        print("Sent Subscribe")
        time.sleep(1.0)  # Wait longer for mixer to process
        
        # Read response and state dump
        sock.settimeout(0.5)
        state = b''
        while True:
            try:
                chunk = sock.recv(8192)
                if not chunk:
                    break
                state += chunk
            except socket.timeout:
                break
        
        print(f"Received {len(state)} bytes")
        
        # Check for SubscriptionReply
        if b'SubscriptionReply' in state:
            print("Got SubscriptionReply - subscribed!")
        else:
            print("WARNING: No SubscriptionReply found")
        
        # Extract session CBytes from SubscriptionReply
        session_cbytes = b'\x65\x00\x6a\x00'  # Default: e j (response to j e)
        idx = state.find(b'SubscriptionReply')
        if idx > 0:
            for i in range(idx, max(0, idx-50), -1):
                if state[i:i+4] == MAGIC and state[i+6:i+8] == TYPE_JSON:
                    session_cbytes = state[i+8:i+12]
                    print(f"Session CBytes: {session_cbytes.hex()}")
                    break
        
        # Wait a moment for state to settle
        time.sleep(0.5)
        
        # Drain any remaining data
        sock.settimeout(0.2)
        try:
            while sock.recv(4096):
                pass
        except socket.timeout:
            pass
        
        # Send PV packet with session CBytes
        print(f"\nSending main/ch1/volume = {value}")
        pv1 = build_pv('main/ch1/volume', value, session_cbytes)
        print(f"  Packet (session cbytes): {pv1.hex()}")
        sock.sendall(pv1)
        
        time.sleep(0.3)
        
        # Also try with UC's CBytes (r e)
        pv2 = build_pv('main/ch1/volume', value, b'\x72\x00\x65\x00')
        print(f"  Packet (UC cbytes r/e): {pv2.hex()}")
        sock.sendall(pv2)
        
        time.sleep(0.5)
        
        # Check for response
        sock.settimeout(0.5)
        try:
            resp = sock.recv(4096)
            print(f"Response: {len(resp)} bytes")
        except socket.timeout:
            print("No response (timeout)")
        
        print("\nDid the fader move?")
        
    finally:
        sock.close()


if __name__ == "__main__":
    main()
