#!/usr/bin/env python3
"""
Monitor ucdaemon traffic and try sending a parameter.
This script connects as a second client and watches what happens.
"""
import socket
import struct
import time
import json
import subprocess
import re
import threading

UCNET_MAGIC = b'\x55\x43\x00\x01'

def find_ucdaemon_port():
    """Find the port ucdaemon is listening on by looking at Universal Control's connection."""
    try:
        # Best method: find what port Universal Control is connected to
        result = subprocess.run(['lsof', '-i', '-P'], capture_output=True, text=True)
        for line in result.stdout.split('\n'):
            if 'Universal' in line and 'TCP' in line and 'ESTABLISHED' in line and 'localhost' in line:
                # Extract destination port from "localhost:49627->localhost:52054"
                match = re.search(r'localhost:\d+->localhost:(\d+)', line)
                if match:
                    port = int(match.group(1))
                    print(f"Found UC connected to port {port}")
                    return port
    except Exception as e:
        print(f"Error finding port: {e}")
    return None

def build_packet(ptype, payload):
    size = len(ptype) + len(payload)
    return UCNET_MAGIC + struct.pack('<H', size) + ptype + payload

def build_hello():
    # Hello format from protocol.rs
    return build_packet(b'\x55\x4d', b'\x00\x00\x65\x00\x15\xfa')

def build_keepalive():
    # KA with j\0e\0 payload
    return build_packet(b'\x4b\x41', b'\x6a\x00\x65\x00')

def build_subscribe():
    data = {
        'id': 'Subscribe',
        'clientName': 'FaderBridge',
        'clientInternalName': 'faderbridge',  # Use unique name
        'clientType': 'ONPC',  # Try different type
        'clientDescription': 'FaderBridge MIDI Controller',
        'clientIdentifier': 'FaderBridge-001',
        'clientOptions': 'perm users levl redu rtan',
        'clientEncoding': 23106
    }
    json_bytes = json.dumps(data).encode()
    cbytes = b'\x6a\x00\x65\x00'  # j\0e\0
    payload = cbytes + struct.pack('<I', len(json_bytes)) + json_bytes
    return build_packet(b'\x4a\x4d', payload)

def build_param_pv(path, value):
    """Build a PV (ParameterValue) packet - used for broadcasts"""
    direction = b'\x75\x00\x68\x00'
    path_bytes = path.encode('ascii') + b'\x00\x00\x00'
    value_bytes = struct.pack('<f', value)
    payload = direction + path_bytes + value_bytes
    return build_packet(b'\x50\x56', payload)

def build_param_ps(path, value):
    """Build a PS (ParameterSet) packet - used for setting values"""
    # PS format: CBytes (4) + key + null + padding (2) + value (4)
    cbytes = b'\x6a\x00\x65\x00'  # j\0e\0 - typical CBytes
    path_bytes = path.encode('ascii') + b'\x00'  # null terminated
    padding = b'\x00\x00'  # 2 bytes padding
    value_bytes = struct.pack('<f', value)
    payload = cbytes + path_bytes + padding + value_bytes
    return build_packet(b'\x50\x53', payload)  # PS type

def parse_packet(data, offset=0):
    """Parse a UCNet packet from data starting at offset."""
    if len(data) < offset + 8:
        return None, offset
    
    if data[offset:offset+4] != UCNET_MAGIC:
        return None, offset + 1
    
    size = struct.unpack('<H', data[offset+4:offset+6])[0]
    ptype = data[offset+6:offset+8].decode('ascii', errors='replace')
    
    if len(data) < offset + 6 + size:
        return None, offset  # Incomplete packet
    
    payload = data[offset+8:offset+6+size]
    return {'type': ptype, 'size': size, 'payload': payload}, offset + 6 + size

def receiver_thread(sock, stop_event):
    """Receive and print packets."""
    buffer = b''
    while not stop_event.is_set():
        try:
            data = sock.recv(4096)
            if not data:
                break
            buffer += data
            
            # Parse packets
            offset = 0
            while offset < len(buffer):
                pkt, new_offset = parse_packet(buffer, offset)
                if pkt is None:
                    if new_offset == offset:
                        break
                    offset = new_offset
                    continue
                
                # Print packet info
                if pkt['type'] == 'PV':
                    # Parameter value
                    payload = pkt['payload']
                    if len(payload) > 8:
                        direction = payload[:4]
                        rest = payload[4:]
                        null_idx = rest.find(b'\x00')
                        if null_idx > 0:
                            path = rest[:null_idx].decode('ascii', errors='replace')
                            value_bytes = rest[-4:]
                            value = struct.unpack('<f', value_bytes)[0]
                            dir_str = 'h->u' if direction == b'\x68\x00\x75\x00' else 'u->h'
                            print(f"  RX PV [{dir_str}]: {path} = {value:.4f}")
                elif pkt['type'] == 'JM':
                    # JSON
                    if len(pkt['payload']) > 8:
                        json_len = struct.unpack('<I', pkt['payload'][4:8])[0]
                        json_data = pkt['payload'][8:8+json_len].decode('utf-8', errors='replace')
                        print(f"  RX JM: {json_data[:100]}")
                elif pkt['type'] == 'KA':
                    pass  # Ignore keepalives
                else:
                    print(f"  RX {pkt['type']}: {len(pkt['payload'])} bytes")
                
                offset = new_offset
            
            buffer = buffer[offset:]
            
        except socket.timeout:
            continue
        except Exception as e:
            if not stop_event.is_set():
                print(f"Receive error: {e}")
            break

def main():
    port = find_ucdaemon_port()
    if not port:
        print("Could not find ucdaemon port")
        return
    
    print(f"Found ucdaemon on port {port}")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(1.0)
    
    try:
        sock.connect(('localhost', port))
    except Exception as e:
        print(f"Connection failed: {e}")
        return
    
    print("Connected!")
    
    # Start receiver thread
    stop_event = threading.Event()
    recv_thread = threading.Thread(target=receiver_thread, args=(sock, stop_event))
    recv_thread.daemon = True
    recv_thread.start()
    
    # Send hello
    print("Sending Hello...")
    sock.send(build_hello())
    time.sleep(0.2)
    
    # Send subscribe
    print("Sending Subscribe...")
    sock.send(build_subscribe())
    time.sleep(0.5)
    
    # Send keepalive immediately
    print("Sending KeepAlive...")
    sock.send(build_keepalive())
    time.sleep(0.5)
    
    print("\n--- Handshake complete ---")
    print("Move the Main slider in Universal Control NOW!")
    print("Watching for 10 seconds (sending keepalives)...")
    
    # Keep sending keepalives while watching
    for i in range(10):
        time.sleep(1)
        sock.send(build_keepalive())
        print(f"  [keepalive {i+1}]")
    
    # Try different parameter paths for Quantum HD 2
    # Device ID from settings file: QT8E24050049
    
    paths_to_try = [
        'main/ch1/volume',                    # From settings file structure
        'QT8E24050049/main/ch1/volume',       # With device ID prefix
        'global/mainOutVolume',               # Original guess
    ]
    
    for path in paths_to_try:
        print(f"\nTrying PS packet: {path} = -10.0")
        packet = build_param_ps(path, -10.0)
        print(f"  Packet: {packet.hex()}")
        sock.send(packet)
        time.sleep(0.5)
    
    print("\nWaiting for response (3 seconds)...")
    time.sleep(3)
    
    stop_event.set()
    sock.close()
    print("Done")

if __name__ == "__main__":
    main()
