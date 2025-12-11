#!/usr/bin/env python3
"""
Listen for parameter changes from SE24 mixer.

Connect to the mixer and display any PV (ParameterValue) packets received.
This helps discover parameter paths by watching what changes when you
interact with the mixer physically.

Usage:
    python3 listen_params.py <mixer_ip> [duration_seconds]
"""

import socket
import struct
import time
import json
import sys

MAGIC = b'\x55\x43\x00\x01'
TYPE_HELLO = b'\x55\x4d'
TYPE_JSON = b'\x4a\x4d'
TYPE_PV = b'\x50\x56'
TYPE_PS = b'\x50\x53'


def build_packet(ptype, payload):
    size = struct.pack('<H', 2 + len(payload))
    return MAGIC + size + ptype + payload


def build_hello():
    return build_packet(TYPE_HELLO, b'\x00\x00\x65\x00\x15\xfa')


def build_subscribe():
    data = {
        'id': 'Subscribe',
        'clientName': 'Universal Control',
        'clientInternalName': 'ucapp',
        'clientType': 'Mac',
        'clientDescription': 'ParamListener',
        'clientIdentifier': 'ParamListener',
        'clientOptions': 'perm users levl redu rtan',
        'clientEncoding': 23106,
    }
    json_bytes = json.dumps(data).encode()
    cbytes = b'\x72\x00\x65\x00'
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(TYPE_JSON, payload)


def parse_packets(data):
    """Parse UCNet packets and yield (type, key, value) tuples."""
    pos = 0
    while pos < len(data) - 8:
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
        
        if ptype == TYPE_PV:
            # Float parameter
            key_start = 4
            null_idx = payload.find(b'\x00', key_start)
            if null_idx > key_start and len(payload) >= null_idx + 7:
                key = payload[key_start:null_idx].decode('ascii', errors='replace')
                value = struct.unpack('<f', payload[-4:])[0]
                yield ('PV', key, value)
        
        elif ptype == TYPE_PS:
            # String parameter
            key_start = 4
            null_idx = payload.find(b'\x00', key_start)
            if null_idx > key_start:
                key = payload[key_start:null_idx].decode('ascii', errors='replace')
                val_start = null_idx + 3
                val_end = payload.find(b'\x00', val_start)
                if val_end > val_start:
                    value = payload[val_start:val_end].decode('utf-8', errors='replace')
                    yield ('PS', key, value)
        
        pos = idx + 6 + size


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 listen_params.py <mixer_ip> [duration_seconds]")
        return
    
    host = sys.argv[1]
    duration = int(sys.argv[2]) if len(sys.argv) > 2 else 60
    
    print(f"Connecting to {host}:53000...")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    sock.connect((host, 53000))
    print("Connected!")
    
    # Handshake
    sock.sendall(build_hello())
    time.sleep(0.2)
    sock.sendall(build_subscribe())
    time.sleep(1.0)
    
    # Drain initial state
    sock.settimeout(0.5)
    initial_bytes = 0
    try:
        while True:
            chunk = sock.recv(8192)
            if chunk:
                initial_bytes += len(chunk)
            else:
                break
    except socket.timeout:
        pass
    
    print(f"Received {initial_bytes} bytes of initial state")
    print()
    print(f"Listening for {duration} seconds...")
    print("Move faders, press mute/solo, turn knobs on the mixer!")
    print("-" * 60)
    
    sock.settimeout(0.5)
    start_time = time.time()
    seen_values = {}  # Track last seen value for each param
    
    try:
        while time.time() - start_time < duration:
            try:
                data = sock.recv(8192)
                if not data:
                    continue
                
                for ptype, key, value in parse_packets(data):
                    # Skip permission params
                    if key.startswith('permissions/'):
                        continue
                    
                    # Only show if value changed
                    last_val = seen_values.get(key)
                    if last_val is None or (isinstance(value, float) and abs(value - last_val) > 0.001):
                        seen_values[key] = value
                        
                        if ptype == 'PV':
                            print(f"PV: {key} = {value:.4f}")
                        else:
                            print(f"PS: {key} = \"{value}\"")
                
            except socket.timeout:
                pass
    
    except KeyboardInterrupt:
        print("\nStopped by user.")
    
    sock.close()
    
    # Summary
    print()
    print("-" * 60)
    print("Parameters seen:")
    for key in sorted(seen_values.keys()):
        val = seen_values[key]
        if isinstance(val, float):
            print(f"  {key} = {val:.4f}")
        else:
            print(f"  {key} = \"{val}\"")


if __name__ == "__main__":
    main()
