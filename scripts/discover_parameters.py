#!/usr/bin/env python3
"""
UCNet Parameter Discovery for StudioLive SE24

Connects to the mixer, subscribes, and captures the full state dump to extract
all available parameter paths. Also allows interactive parameter testing.

Usage:
    python3 discover_parameters.py <mixer_ip> [--dump] [--test key value]
    
Examples:
    python3 discover_parameters.py 192.168.1.209 --dump
    python3 discover_parameters.py 192.168.1.209 --test "main/ch1/pan" 0.5
"""

import socket
import struct
import sys
import time
import json
import argparse
import re
from collections import defaultdict

MAGIC = b'\x55\x43\x00\x01'
TYPE_HELLO = b'\x55\x4d'
TYPE_JSON = b'\x4a\x4d'
TYPE_PV = b'\x50\x56'
TYPE_KA = b'\x4b\x41'
TYPE_PS = b'\x50\x53'  # ParameterString
TYPE_PL = b'\x50\x4c'  # ParameterList?
TYPE_ZB = b'\x5a\x42'  # Compressed data


def build_packet(ptype: bytes, payload: bytes) -> bytes:
    """Build UCNet packet with little-endian size including type."""
    size = struct.pack('<H', 2 + len(payload))
    return MAGIC + size + ptype + payload


def build_hello() -> bytes:
    return build_packet(TYPE_HELLO, b'\x00\x00\x65\x00\x15\xfa')


def build_subscribe() -> bytes:
    data = {
        "id": "Subscribe",
        "clientName": "Universal Control",
        "clientInternalName": "ucapp",
        "clientType": "Mac",
        "clientDescription": "ParameterDiscovery",
        "clientIdentifier": "ParameterDiscovery",
        "clientOptions": "perm users levl redu rtan",
        "clientEncoding": 23106,
    }
    json_bytes = json.dumps(data).encode()
    cbytes = b'\x72\x00\x65\x00'  # r e
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(TYPE_JSON, payload)


def build_pv(key: str, value: float, cbytes: bytes = b'\x72\x00\x65\x00') -> bytes:
    """Build ParameterValue packet."""
    key_bytes = key.encode('ascii') + b'\x00'
    padding = b'\x00\x00'
    val_bytes = struct.pack('<f', value)
    payload = cbytes + key_bytes + padding + val_bytes
    return build_packet(TYPE_PV, payload)


def parse_packets(data: bytes) -> list:
    """Parse UCNet packets from raw data."""
    packets = []
    pos = 0
    while pos < len(data) - 8:
        # Find magic
        idx = data.find(MAGIC, pos)
        if idx == -1:
            break
        
        if idx + 8 > len(data):
            break
            
        size = struct.unpack('<H', data[idx+4:idx+6])[0]
        ptype = data[idx+6:idx+8]
        
        if idx + 6 + size > len(data):
            pos = idx + 1
            continue
            
        payload = data[idx+8:idx+6+size]
        packets.append({
            'offset': idx,
            'size': size,
            'type': ptype,
            'type_str': ptype.decode('ascii', errors='replace'),
            'payload': payload
        })
        pos = idx + 6 + size
    
    return packets


def extract_pv_params(packets: list) -> dict:
    """Extract ParameterValue parameters from packets."""
    params = {}
    for pkt in packets:
        if pkt['type'] == TYPE_PV:
            payload = pkt['payload']
            if len(payload) < 10:
                continue
            
            # Skip CBytes (4 bytes)
            key_start = 4
            # Find null terminator for key
            null_idx = payload.find(b'\x00', key_start)
            if null_idx == -1:
                continue
            
            key = payload[key_start:null_idx].decode('ascii', errors='replace')
            
            # Value is last 4 bytes (after padding)
            if len(payload) >= null_idx + 7:
                try:
                    value = struct.unpack('<f', payload[-4:])[0]
                    params[key] = value
                except:
                    pass
    
    return params


def extract_ps_params(packets: list) -> dict:
    """Extract ParameterString parameters from packets."""
    params = {}
    for pkt in packets:
        if pkt['type'] == TYPE_PS:
            payload = pkt['payload']
            if len(payload) < 10:
                continue
            
            # Skip CBytes (4 bytes)
            key_start = 4
            # Find null terminator for key
            null_idx = payload.find(b'\x00', key_start)
            if null_idx == -1:
                continue
            
            key = payload[key_start:null_idx].decode('ascii', errors='replace')
            
            # String value follows after padding
            val_start = null_idx + 3  # Skip null + 2 padding bytes
            val_end = payload.find(b'\x00', val_start)
            if val_end == -1:
                val_end = len(payload)
            
            try:
                value = payload[val_start:val_end].decode('ascii', errors='replace')
                params[key] = value
            except:
                pass
    
    return params


def categorize_params(params: dict) -> dict:
    """Categorize parameters by path prefix."""
    categories = defaultdict(dict)
    for key, value in params.items():
        parts = key.split('/')
        if len(parts) >= 2:
            category = parts[0]
            categories[category][key] = value
        else:
            categories['other'][key] = value
    return dict(categories)


def connect_and_subscribe(host: str, timeout: float = 10.0) -> tuple:
    """Connect to mixer and subscribe, return socket and state data."""
    print(f"Connecting to {host}:53000...")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    sock.connect((host, 53000))
    print("Connected!")
    
    # Send Hello
    sock.sendall(build_hello())
    print("Sent Hello")
    time.sleep(0.2)
    
    # Send Subscribe
    sock.sendall(build_subscribe())
    print("Sent Subscribe")
    
    # Collect state dump
    print(f"Collecting state dump (waiting {timeout}s)...")
    sock.settimeout(0.5)
    state = b''
    start = time.time()
    
    while time.time() - start < timeout:
        try:
            chunk = sock.recv(65536)
            if chunk:
                state += chunk
                print(f"  Received {len(state)} bytes...", end='\r')
        except socket.timeout:
            # Check if we've stopped receiving
            if len(state) > 1000 and time.time() - start > 2:
                break
    
    print(f"\nTotal received: {len(state)} bytes")
    
    # Check for SubscriptionReply
    if b'SubscriptionReply' in state:
        print("Got SubscriptionReply - subscribed!")
    else:
        print("WARNING: No SubscriptionReply found")
    
    return sock, state


def dump_parameters(host: str):
    """Dump all parameters from the mixer."""
    sock, state = connect_and_subscribe(host)
    
    try:
        # Parse packets
        packets = parse_packets(state)
        print(f"\nParsed {len(packets)} packets")
        
        # Count packet types
        type_counts = defaultdict(int)
        for pkt in packets:
            type_counts[pkt['type_str']] += 1
        
        print("\nPacket types:")
        for t, count in sorted(type_counts.items()):
            print(f"  {t}: {count}")
        
        # Extract PV parameters
        pv_params = extract_pv_params(packets)
        print(f"\nFound {len(pv_params)} PV (float) parameters")
        
        # Extract PS parameters
        ps_params = extract_ps_params(packets)
        print(f"Found {len(ps_params)} PS (string) parameters")
        
        # Categorize and display
        if pv_params:
            categories = categorize_params(pv_params)
            print("\n" + "="*60)
            print("FLOAT PARAMETERS (PV)")
            print("="*60)
            
            for category in sorted(categories.keys()):
                params = categories[category]
                print(f"\n[{category}] ({len(params)} params)")
                for key in sorted(params.keys())[:50]:  # Limit display
                    print(f"  {key} = {params[key]:.4f}")
                if len(params) > 50:
                    print(f"  ... and {len(params) - 50} more")
        
        if ps_params:
            categories = categorize_params(ps_params)
            print("\n" + "="*60)
            print("STRING PARAMETERS (PS)")
            print("="*60)
            
            for category in sorted(categories.keys()):
                params = categories[category]
                print(f"\n[{category}] ({len(params)} params)")
                for key in sorted(params.keys())[:30]:
                    val = params[key][:50] if len(params[key]) > 50 else params[key]
                    print(f"  {key} = \"{val}\"")
                if len(params) > 30:
                    print(f"  ... and {len(params) - 30} more")
        
        # Save full dump to file
        output_file = f"se24_parameters_{int(time.time())}.txt"
        with open(output_file, 'w') as f:
            f.write("# SE24 Parameter Dump\n")
            f.write(f"# Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            
            f.write("## Float Parameters (PV)\n\n")
            for key in sorted(pv_params.keys()):
                f.write(f"{key} = {pv_params[key]}\n")
            
            f.write("\n## String Parameters (PS)\n\n")
            for key in sorted(ps_params.keys()):
                f.write(f"{key} = \"{ps_params[key]}\"\n")
        
        print(f"\nFull dump saved to: {output_file}")
        
        # Also save raw data for analysis
        raw_file = f"se24_raw_{int(time.time())}.bin"
        with open(raw_file, 'wb') as f:
            f.write(state)
        print(f"Raw data saved to: {raw_file}")
        
    finally:
        sock.close()


def test_parameter(host: str, key: str, value: float):
    """Test setting a specific parameter."""
    sock, state = connect_and_subscribe(host, timeout=3.0)
    
    try:
        # Extract session CBytes
        session_cbytes = b'\x65\x00\x6a\x00'
        idx = state.find(b'SubscriptionReply')
        if idx > 0:
            for i in range(idx, max(0, idx-50), -1):
                if state[i:i+4] == MAGIC and state[i+6:i+8] == TYPE_JSON:
                    session_cbytes = state[i+8:i+12]
                    print(f"Session CBytes: {session_cbytes.hex()}")
                    break
        
        # Drain remaining data
        sock.settimeout(0.2)
        try:
            while sock.recv(4096):
                pass
        except socket.timeout:
            pass
        
        # Send PV packet
        print(f"\nSending: {key} = {value}")
        
        # Try with session CBytes
        pv = build_pv(key, value, session_cbytes)
        print(f"  Packet: {pv.hex()}")
        sock.sendall(pv)
        time.sleep(0.3)
        
        # Also try with UC CBytes
        pv2 = build_pv(key, value, b'\x72\x00\x65\x00')
        sock.sendall(pv2)
        time.sleep(0.3)
        
        # Check response
        sock.settimeout(0.5)
        try:
            resp = sock.recv(4096)
            print(f"Response: {len(resp)} bytes")
            if resp:
                # Try to parse any PV response
                packets = parse_packets(resp)
                for pkt in packets:
                    if pkt['type'] == TYPE_PV:
                        print(f"  PV response: {pkt['payload'].hex()}")
        except socket.timeout:
            print("No response (timeout)")
        
        print("\nDid the parameter change on the mixer?")
        
    finally:
        sock.close()


def interactive_mode(host: str):
    """Interactive parameter testing mode."""
    sock, state = connect_and_subscribe(host, timeout=3.0)
    
    # Extract session CBytes
    session_cbytes = b'\x72\x00\x65\x00'
    idx = state.find(b'SubscriptionReply')
    if idx > 0:
        for i in range(idx, max(0, idx-50), -1):
            if state[i:i+4] == MAGIC and state[i+6:i+8] == TYPE_JSON:
                session_cbytes = state[i+8:i+12]
                break
    
    print(f"\nSession CBytes: {session_cbytes.hex()}")
    print("\nInteractive mode. Commands:")
    print("  set <key> <value>  - Set a float parameter")
    print("  get                - Read any incoming data")
    print("  quit               - Exit")
    print("\nExample: set main/ch1/volume 0.5")
    print("         set main/ch1/pan 0.5")
    print("         set main/ch1/mute 1.0")
    print()
    
    sock.settimeout(0.1)
    
    try:
        while True:
            try:
                cmd = input("> ").strip()
            except EOFError:
                break
            
            if not cmd:
                continue
            
            if cmd == 'quit':
                break
            
            if cmd == 'get':
                try:
                    data = sock.recv(8192)
                    if data:
                        packets = parse_packets(data)
                        for pkt in packets:
                            print(f"  {pkt['type_str']}: {pkt['payload'][:40].hex()}...")
                except socket.timeout:
                    print("  (no data)")
                continue
            
            if cmd.startswith('set '):
                parts = cmd.split()
                if len(parts) != 3:
                    print("Usage: set <key> <value>")
                    continue
                
                key = parts[1]
                try:
                    value = float(parts[2])
                except ValueError:
                    print("Value must be a number")
                    continue
                
                pv = build_pv(key, value, session_cbytes)
                sock.sendall(pv)
                print(f"Sent: {key} = {value}")
                
                # Also send with UC cbytes
                pv2 = build_pv(key, value, b'\x72\x00\x65\x00')
                sock.sendall(pv2)
                
                time.sleep(0.2)
                
                # Check for response
                try:
                    resp = sock.recv(4096)
                    if resp:
                        print(f"Response: {len(resp)} bytes")
                except socket.timeout:
                    pass
                
                continue
            
            print(f"Unknown command: {cmd}")
    
    finally:
        sock.close()
        print("Disconnected.")


def main():
    parser = argparse.ArgumentParser(description='UCNet Parameter Discovery')
    parser.add_argument('host', help='Mixer IP address')
    parser.add_argument('--dump', action='store_true', help='Dump all parameters')
    parser.add_argument('--test', nargs=2, metavar=('KEY', 'VALUE'), 
                        help='Test setting a parameter')
    parser.add_argument('--interactive', '-i', action='store_true',
                        help='Interactive mode')
    
    args = parser.parse_args()
    
    if args.dump:
        dump_parameters(args.host)
    elif args.test:
        test_parameter(args.host, args.test[0], float(args.test[1]))
    elif args.interactive:
        interactive_mode(args.host)
    else:
        # Default: dump parameters
        dump_parameters(args.host)


if __name__ == "__main__":
    main()
