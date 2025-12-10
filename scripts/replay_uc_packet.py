#!/usr/bin/env python3
"""
Replay the exact PV packet that UC sends, byte-for-byte.
From tcpdump capture:
  55 43 00 01 1c 00 50 56 72 00 65 00 6d 61 69 6e 2f 63 68 31 2f 76 6f 6c 75 6d 65 00 00 00 XX XX XX XX
"""

import socket
import struct
import sys
import time

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 scripts/replay_uc_packet.py <mixer_ip> [value]")
        print("Example: python3 scripts/replay_uc_packet.py 192.168.1.209 0.5")
        return
    
    host = sys.argv[1]
    value = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
    
    # Clamp value
    value = max(0.0, min(1.0, value))
    
    # Build the exact packet UC sends
    # Header: 55 43 00 01 1c 00 50 56
    # CBytes: We'll try different values
    # Key: main/ch1/volume + null
    # Padding: 00 00
    # Value: 4 bytes LE float
    
    magic = b'\x55\x43\x00\x01'
    
    # Use PV with exact same format as UC (confirmed from capture)
    ptype = b'\x50\x56'  # PV
    cbytes = b'\x72\x00\x65\x00'  # r e - exactly what UC uses
    
    key = b'main/ch1/volume\x00'
    padding = b'\x00\x00'
    val_bytes = struct.pack('<f', value)
    
    payload = cbytes + key + padding + val_bytes
    size = struct.pack('<H', 2 + len(payload))  # type + payload
    
    packet = magic + size + ptype + payload
    
    print(f"Connecting to {host}:53000...")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5.0)
    
    try:
        sock.connect((host, 53000))
        print("Connected!")
        
        # Send Hello first
        hello = b'\x55\x43\x00\x01\x08\x00\x55\x4d\x00\x00\x65\x00\x15\xfa'
        print(f"Sending Hello: {hello.hex()}")
        sock.sendall(hello)
        time.sleep(0.2)
        
        # Try to receive response
        sock.settimeout(2.0)
        try:
            resp = sock.recv(4096)
            print(f"Received {len(resp)} bytes after Hello")
            print(f"  First 32 bytes: {resp[:32].hex()}")
        except socket.timeout:
            print("No response to Hello (timeout)")
        
        # Send Subscribe - this is required before the mixer will accept commands
        import json
        sub_data = {
            "id": "Subscribe",
            "clientName": "FaderBridge",
            "clientInternalName": "faderbridge",
            "clientType": "PC",
            "clientDescription": "FaderBridge MIDI Controller",
            "clientIdentifier": "FaderBridge",
            "clientOptions": "perm users levl redu rtan",
            "clientEncoding": 23106,
        }
        sub_json = json.dumps(sub_data).encode()
        sub_cbytes = b'\x6a\x00\x65\x00'  # j e
        sub_payload = sub_cbytes + struct.pack('<I', len(sub_json)) + sub_json
        sub_size = struct.pack('<H', 2 + len(sub_payload))
        sub_packet = b'\x55\x43\x00\x01' + sub_size + b'\x4a\x4d' + sub_payload
        
        print(f"\nSending Subscribe ({len(sub_packet)} bytes)")
        sock.sendall(sub_packet)
        time.sleep(0.5)
        
        # Read subscription response - may come in multiple packets
        all_resp = b''
        sock.settimeout(0.5)
        for _ in range(10):  # Read up to 10 chunks
            try:
                chunk = sock.recv(8192)
                if chunk:
                    all_resp += chunk
                    print(f"  Received chunk: {len(chunk)} bytes")
                else:
                    break
            except socket.timeout:
                break
        
        print(f"Total received after Subscribe: {len(all_resp)} bytes")
        
        # Look for SubscriptionReply and extract CBytes from response
        if b'SubscriptionReply' in all_resp:
            print("  Got SubscriptionReply! We are subscribed.")
            # Find the JM packet with SubscriptionReply and get its CBytes
            idx = all_resp.find(b'SubscriptionReply')
            if idx > 8:
                # Go back to find the packet header
                for i in range(idx, max(0, idx-100), -1):
                    if all_resp[i:i+4] == b'\x55\x43\x00\x01':
                        ptype = all_resp[i+6:i+8]
                        if ptype == b'\x4a\x4d':  # JM
                            resp_cbytes = all_resp[i+8:i+12]
                            print(f"  Response CBytes: {resp_cbytes.hex()} ({resp_cbytes[0]:c}{resp_cbytes[2]:c})")
                            # Use these CBytes for our PV packet!
                            global session_cbytes
                            session_cbytes = resp_cbytes
                        break
        if b'SubscriptionLost' in all_resp:
            print("  Got SubscriptionLost - rejected!")
        
        # Decode packet types received
        pos = 0
        while pos < len(all_resp) - 8:
            if all_resp[pos:pos+4] == b'\x55\x43\x00\x01':
                size = struct.unpack('<H', all_resp[pos+4:pos+6])[0]
                ptype = all_resp[pos+6:pos+8].decode('ascii', errors='replace')
                print(f"  Packet: {ptype} ({size} bytes)")
                pos += 6 + size
            else:
                pos += 1
        
        # Wait a bit for subscription to complete
        time.sleep(1.0)
        
        # Drain any pending data and look for PV packets in the state dump
        sock.settimeout(0.2)
        state_data = b''
        try:
            while True:
                d = sock.recv(8192)
                if not d:
                    break
                state_data += d
                print(f"  (received {len(d)} bytes)")
        except socket.timeout:
            pass
        
        print(f"  Total state dump: {len(state_data)} bytes")
        
        # Find the first JM packet with SubscriptionReply and get CBytes
        session_cbytes = None
        idx = state_data.find(b'SubscriptionReply')
        if idx > 0:
            # Search backwards for packet header
            for i in range(idx, max(0, idx-50), -1):
                if state_data[i:i+4] == b'\x55\x43\x00\x01' and state_data[i+6:i+8] == b'\x4a\x4d':
                    session_cbytes = state_data[i+8:i+12]
                    print(f"  Session CBytes from SubscriptionReply: {session_cbytes.hex()}")
                    break
        
        # Scan all packet types in the state dump
        pos = 0
        packet_types = {}
        print("  Scanning all packet types...")
        while pos < len(state_data) - 10:
            if state_data[pos:pos+4] == b'\x55\x43\x00\x01':
                size = struct.unpack('<H', state_data[pos+4:pos+6])[0]
                ptype = state_data[pos+6:pos+8].decode('ascii', errors='replace')
                packet_types[ptype] = packet_types.get(ptype, 0) + 1
                
                # Show JSON messages
                if ptype == 'JM':
                    payload = state_data[pos+8:pos+6+size]
                    if len(payload) > 8:
                        cbytes = payload[:4]
                        json_len = struct.unpack('<I', payload[4:8])[0]
                        json_data = payload[8:8+json_len]
                        try:
                            print(f"    JM: {json_data.decode('utf-8')}")
                        except:
                            print(f"    JM: (decode error)")
                
                pos += 6 + size
                continue
            pos += 1
        
        print(f"  Packet types found: {packet_types}")
        
        # Try with session CBytes from SubscriptionReply
        key_bytes = b'main/ch1/volume\x00'
        padding_bytes = b'\x00\x00'
        
        if session_cbytes:
            print(f"\nUsing session CBytes: {session_cbytes.hex()}")
            pv_payload = session_cbytes + key_bytes + padding_bytes + val_bytes
            pv_size = struct.pack('<H', 2 + len(pv_payload))
            packet_with_session = magic + pv_size + ptype + pv_payload
            
            print(f"Sending PV with session CBytes: {packet_with_session.hex()}")
            sock.sendall(packet_with_session)
            time.sleep(0.5)
        
        # Also try with UC's CBytes (r e)
        print(f"\nSending PV with UC CBytes (r e): {packet.hex()}")
        sock.sendall(packet)
        
        # Wait for response
        time.sleep(1.0)
        sock.settimeout(1.0)
        try:
            resp = sock.recv(4096)
            print(f"Received {len(resp)} bytes after PV")
            # Check for any PV echo or error
            if b'PV' in resp or b'\x50\x56' in resp:
                print("  Response contains PV packet")
        except socket.timeout:
            print("No response to PV (timeout)")
        
        print("\nDid the fader move?")
        
    except Exception as e:
        print(f"Error: {e}")
    finally:
        sock.close()

if __name__ == "__main__":
    main()
