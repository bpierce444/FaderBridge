#!/usr/bin/env python3
"""Dump the initial state from ucdaemon to find correct parameter paths."""
import socket
import struct
import time
import json
import subprocess
import re
import zlib

UCNET_MAGIC = b'\x55\x43\x00\x01'

def find_ucdaemon_port():
    try:
        result = subprocess.run(['lsof', '-i', '-P'], capture_output=True, text=True)
        for line in result.stdout.split('\n'):
            if 'Universal' in line and 'TCP' in line and 'ESTABLISHED' in line and 'localhost' in line:
                match = re.search(r'localhost:\d+->localhost:(\d+)', line)
                if match:
                    return int(match.group(1))
    except Exception as e:
        print(f"Error: {e}")
    return None

def build_packet(ptype, payload):
    size = len(ptype) + len(payload)
    return UCNET_MAGIC + struct.pack('<H', size) + ptype + payload

def build_hello():
    return build_packet(b'\x55\x4d', b'\x00\x00\x65\x00\x15\xfa')

def build_subscribe():
    data = {
        'id': 'Subscribe',
        'clientName': 'FaderBridge',
        'clientInternalName': 'faderbridge',
        'clientType': 'ONPC',
        'clientDescription': 'FaderBridge MIDI Controller',
        'clientIdentifier': 'FaderBridge-001',
        'clientOptions': 'perm users levl redu rtan',
        'clientEncoding': 23106
    }
    json_bytes = json.dumps(data).encode()
    cbytes = b'\x6a\x00\x65\x00'
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(b'\x4a\x4d', payload)

def main():
    port = find_ucdaemon_port()
    if not port:
        print("Could not find ucdaemon port")
        return
    
    print(f"Connecting to port {port}...")
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    sock.connect(('localhost', port))
    
    # Handshake
    sock.send(build_hello())
    time.sleep(0.1)
    sock.send(build_subscribe())
    time.sleep(1.0)
    
    # Read all data
    all_data = b''
    try:
        while True:
            data = sock.recv(8192)
            if data:
                all_data += data
            else:
                break
    except socket.timeout:
        pass
    
    sock.close()
    print(f"Received {len(all_data)} bytes total")
    
    # Parse packets
    pos = 0
    while pos < len(all_data):
        if all_data[pos:pos+4] != UCNET_MAGIC:
            pos += 1
            continue
        
        size = struct.unpack('<H', all_data[pos+4:pos+6])[0]
        ptype = all_data[pos+6:pos+8]
        payload = all_data[pos+8:pos+6+size]
        
        ptype_str = ptype.decode('ascii', errors='replace')
        print(f"\nPacket: {ptype_str} ({size} bytes)")
        
        if ptype == b'ZM':
            # Zlib compressed - format: CBytes(4) + uncompressed_len(4) + zlib_data
            cbytes = payload[:4]
            uncompressed_len = struct.unpack('<I', payload[4:8])[0]
            compressed = payload[8:]
            print(f"  CBytes: {cbytes.hex()}")
            print(f"  Uncompressed len: {uncompressed_len}")
            print(f"  Compressed: {len(compressed)} bytes")
            print(f"  First 20 bytes: {compressed[:20].hex()}")
            
            # Try different decompression methods
            for wbits in [15, -15, 31, -zlib.MAX_WBITS, zlib.MAX_WBITS]:
                try:
                    decompressed = zlib.decompress(compressed, wbits)
                    print(f"  Decompressed (wbits={wbits}): {len(decompressed)} bytes")
                    if len(decompressed) > 10:
                        # Look for parameter paths
                        text = decompressed.decode('utf-8', errors='replace')
                        print(f"  Full content:")
                        print(text)
                        break
                except Exception as e:
                    pass
            else:
                # Maybe it's not compressed, just raw data
                print(f"  Could not decompress, showing raw:")
                # Look for readable strings
                text = compressed.decode('utf-8', errors='replace')
                print(f"  As text: {text[:200]}")
        
        elif ptype == b'JM':
            cbytes = payload[:4]
            if len(payload) > 8:
                json_len = struct.unpack('<I', payload[4:8])[0]
                json_data = payload[8:8+json_len].decode('utf-8', errors='replace')
                print(f"  JSON: {json_data}")
        
        pos += 6 + size
    
    print("\nDone")

if __name__ == "__main__":
    main()
