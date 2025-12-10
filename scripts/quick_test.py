#!/usr/bin/env python3
"""Quick test to send a parameter to ucdaemon"""
import socket
import struct
import time
import json
import subprocess
import re

UCNET_MAGIC = b'\x55\x43\x00\x01'

def find_ucdaemon_port():
    """Find the port ucdaemon is listening on (it's dynamic)."""
    try:
        result = subprocess.run(
            ['lsof', '-i', '-P'],
            capture_output=True, text=True
        )
        # Look for Universal Control's connection to find the daemon port
        for line in result.stdout.split('\n'):
            if 'Universal' in line and 'localhost' in line and 'ESTABLISHED' in line:
                # Extract the destination port (e.g., localhost:52054)
                match = re.search(r'localhost:(\d+)->localhost:(\d+)', line)
                if match:
                    return int(match.group(2))
        
        # Fallback: look for listening ports in the 52xxx range
        result2 = subprocess.run(
            ['netstat', '-an'],
            capture_output=True, text=True
        )
        for line in result2.stdout.split('\n'):
            if 'LISTEN' in line and '127.0.0.1.52' in line:
                match = re.search(r'127\.0\.0\.1\.(\d+)', line)
                if match:
                    return int(match.group(1))
    except Exception as e:
        print(f"Error finding port: {e}")
    
    return 51801  # Default fallback

def build_packet(ptype, payload):
    size = len(ptype) + len(payload)
    return UCNET_MAGIC + struct.pack('<H', size) + ptype + payload

def build_hello():
    return build_packet(b'\x55\x4d', b'\x00\x00\x65\x00\x15\xFA')

def build_subscribe():
    # Match Universal Control's exact format
    data = {
        'id': 'Subscribe',
        'clientName': 'Universal Control',  # Match UC
        'clientInternalName': 'ucapp',       # Match UC exactly
        'clientType': 'PC',
        'clientDescription': 'Universal Control',
        'clientIdentifier': 'FaderBridge-001',
        'clientOptions': 'perm users levl redu rtan',
        'clientEncoding': 23106
    }
    json_bytes = json.dumps(data).encode()
    # C-bytes: k\0e\0 as seen in UC traffic
    cbytes = b'\x6b\x00\x65\x00'
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(b'\x4a\x4d', payload)

def build_param(path, value):
    direction = b'\x75\x00\x68\x00'
    path_bytes = path.encode('ascii') + b'\x00\x00\x00'
    value_bytes = struct.pack('<f', value)
    payload = direction + path_bytes + value_bytes
    return build_packet(b'\x50\x56', payload)

def main():
    port = find_ucdaemon_port()
    print(f"Found ucdaemon on port {port}")
    print("Connecting to ucdaemon...")
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    
    try:
        sock.connect(('localhost', port))
    except Exception as e:
        print(f"Connection failed: {e}")
        return
    
    print("Connected!")
    
    # Handshake
    sock.send(build_hello())
    time.sleep(0.1)
    sock.send(build_subscribe())
    time.sleep(0.3)
    
    # Drain responses
    try:
        while True:
            sock.recv(4096)
    except socket.timeout:
        pass
    
    print("Handshake complete")
    
    # Send parameter change
    packet = build_param('global/mainOutVolume', 0.1)
    print(f"Sending: mainOutVolume = 0.1")
    print(f"Packet: {packet.hex()}")
    sock.send(packet)
    
    time.sleep(1)
    sock.close()
    print("Done - check if volume changed!")

if __name__ == "__main__":
    main()
