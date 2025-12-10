#!/usr/bin/env python3
"""
test_ucdaemon_control.py - Test UCNet control via ucdaemon

Connects to ucdaemon on localhost:51801 and sends parameter control commands
to the Quantum HD 2 (or any USB-connected PreSonus device).

Usage:
    python3 test_ucdaemon_control.py
"""

import socket
import struct
import time
import threading
import sys

# UCNet constants
UCNET_MAGIC = b'\x55\x43\x00\x01'  # "UC\x00\x01"
UCDAEMON_HOST = 'localhost'
UCDAEMON_PORT = 51801

# Payload type codes
TYPE_HELLO = b'\x55\x4d'      # "UM" - Hello (Universal Message)
TYPE_KEEPALIVE = b'\x4b\x41'  # "KA" - KeepAlive
TYPE_PARAM_VALUE = b'\x50\x56'  # "PV" - ParameterValue
TYPE_PARAM_SET = b'\x50\x53'  # "PS" - ParameterSet (for setting values)

# Direction indicators (with null bytes between chars)
# Based on tcpdump capture:
# - Universal Control sends with "u\0h\0" (unit to host - requesting change)
# - ucdaemon responds with "h\0u\0" (host to unit - confirming change)
DIR_CLIENT_TO_DAEMON = b'\x75\x00\x68\x00'  # "u\0h\0" - client sending to daemon
DIR_DAEMON_TO_CLIENT = b'\x68\x00\x75\x00'  # "h\0u\0" - daemon sending to client


def build_ucnet_packet(payload_type: bytes, payload: bytes) -> bytes:
    """Build a UCNet packet with magic, size, type, and payload."""
    # Size includes the payload type (2 bytes) + payload
    size = len(payload_type) + len(payload)
    return UCNET_MAGIC + struct.pack('<H', size) + payload_type + payload


def build_hello_packet() -> bytes:
    """Build a UCNet Hello packet."""
    # Hello packet format from protocol.rs: 00 00 65 00 15 FA
    hello_payload = b'\x00\x00\x65\x00\x15\xFA'
    return build_ucnet_packet(TYPE_HELLO, hello_payload)


def build_keepalive_packet() -> bytes:
    """Build a UCNet KeepAlive packet."""
    # Observed format: KA with "j\0e\0" payload
    ka_payload = b'\x6a\x00\x65\x00'  # "j\0e\0"
    return build_ucnet_packet(TYPE_KEEPALIVE, ka_payload)


def build_parameter_packet(param_path: str, value: float) -> bytes:
    """
    Build a ParameterValue packet to set a parameter.
    
    Observed packet format from tcpdump (for "global/mainOutVolume"):
    55 43 00 01              # Magic
    21 00                    # Size (little-endian): 33 bytes  
    50 56                    # Type: PV
    68 00 75 00              # Direction: h\0u\0 (host to unit) - 4 bytes
    67 6c 6f 62 61 6c 2f...  # Path: "global/mainOutVolume" - 20 bytes
    00 00 00                 # Null terminator + 2 padding bytes - 3 bytes
    XX XX XX XX              # Float value (little-endian) - 4 bytes
    
    Total payload = 4 + 20 + 3 + 4 = 31 bytes
    Size field = 2 (type) + 31 = 33 bytes
    """
    # Direction: client to daemon (u\0h\0)
    direction = DIR_CLIENT_TO_DAEMON  # 4 bytes
    
    # Parameter path as null-terminated ASCII string
    path_bytes = param_path.encode('ascii') + b'\x00'
    
    # Padding: add bytes so (path + null + padding) aligns to leave room for 4-byte value
    # Observed: path ends with 00 00 00 (null + 2 padding)
    # This seems to be: pad to make (direction + path_with_null + padding) % 4 == 0
    # Then the 4-byte float follows
    current_len = len(direction) + len(path_bytes)
    padding_needed = (4 - (current_len % 4)) % 4
    if padding_needed == 0:
        padding_needed = 4  # Always have some padding before value
    padding = b'\x00' * padding_needed
    
    # Actually, looking at the capture more carefully:
    # "global/mainOutVolume" = 20 chars + null = 21 bytes
    # direction = 4 bytes
    # Total = 25 bytes, need 2 more to get to 27, then +4 for float = 31
    # Let's just add 2 null bytes after the null terminator
    padding = b'\x00\x00'
    
    # Value as IEEE 754 float (little-endian)
    value_bytes = struct.pack('<f', value)
    
    payload = direction + path_bytes + padding + value_bytes
    return build_ucnet_packet(TYPE_PARAM_VALUE, payload)


def parse_ucnet_packet(data: bytes) -> dict:
    """Parse a UCNet packet."""
    if len(data) < 8:
        return None
    
    if data[:4] != UCNET_MAGIC:
        return None
    
    size = struct.unpack('<H', data[4:6])[0]
    ptype = data[6:8]
    payload = data[8:8+size-2] if len(data) >= 8 + size - 2 else b''
    
    return {
        'size': size,
        'type': ptype,
        'type_str': ptype.decode('ascii', errors='replace'),
        'payload': payload
    }


class UCDaemonClient:
    """Client for communicating with ucdaemon via UCNet."""
    
    def __init__(self, host=UCDAEMON_HOST, port=UCDAEMON_PORT):
        self.host = host
        self.port = port
        self.sock = None
        self.running = False
        self.recv_thread = None
    
    def connect(self) -> bool:
        """Connect to ucdaemon."""
        try:
            self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.sock.connect((self.host, self.port))
            self.sock.settimeout(0.1)
            print(f"Connected to ucdaemon at {self.host}:{self.port}")
            
            # Start receive thread
            self.running = True
            self.recv_thread = threading.Thread(target=self._receive_loop, daemon=True)
            self.recv_thread.start()
            
            # Send hello
            self._send_hello()
            
            return True
        except Exception as e:
            print(f"Connection failed: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from ucdaemon."""
        self.running = False
        if self.sock:
            self.sock.close()
            self.sock = None
        print("Disconnected")
    
    def _send_hello(self):
        """Send hello and subscribe packets to establish session."""
        # 1. Send Hello (UM) packet
        hello = build_hello_packet()
        self.sock.send(hello)
        print(f"Sent: Hello (UM)")
        
        time.sleep(0.1)
        
        # 2. Send Subscribe (JM) packet with JSON payload
        subscribe = self._build_subscribe_packet()
        self.sock.send(subscribe)
        print(f"Sent: Subscribe (JM)")
        
        time.sleep(0.1)
        
        # 3. Send initial KeepAlive
        ka = build_keepalive_packet()
        self.sock.send(ka)
        print(f"Sent: KeepAlive (KA)")
    
    def _build_subscribe_packet(self) -> bytes:
        """Build a Subscribe JSON packet."""
        import json
        
        subscribe_data = {
            "id": "Subscribe",
            "clientName": "FaderBridge",
            "clientInternalName": "faderbridge", 
            "clientType": "PC",
            "clientDescription": "FaderBridge MIDI Controller",
            "clientIdentifier": "FaderBridge-Test",
            "clientOptions": "perm users levl redu rtan",
            "clientEncoding": 23106
        }
        
        json_bytes = json.dumps(subscribe_data).encode('utf-8')
        
        # JM packet format: C-Bytes (4) + JSON length (4) + JSON data
        cbytes = b'\x6a\x00\x65\x00'  # j\0e\0
        json_len = struct.pack('<I', len(json_bytes))
        
        payload = cbytes + json_len + json_bytes
        
        # Build packet with JM type
        TYPE_JSON = b'\x4a\x4d'  # "JM"
        return UCNET_MAGIC + struct.pack('<H', len(TYPE_JSON) + len(payload)) + TYPE_JSON + payload
    
    def _receive_loop(self):
        """Background thread to receive packets."""
        buffer = b''
        while self.running:
            try:
                data = self.sock.recv(1024)
                if data:
                    buffer += data
                    # Try to parse complete packets
                    while len(buffer) >= 8:
                        if buffer[:4] != UCNET_MAGIC:
                            # Find next magic
                            idx = buffer.find(UCNET_MAGIC, 1)
                            if idx > 0:
                                buffer = buffer[idx:]
                            else:
                                buffer = b''
                            continue
                        
                        size = struct.unpack('<H', buffer[4:6])[0]
                        packet_len = 6 + size
                        
                        if len(buffer) >= packet_len:
                            packet_data = buffer[:packet_len]
                            buffer = buffer[packet_len:]
                            
                            pkt = parse_ucnet_packet(packet_data)
                            if pkt:
                                self._handle_packet(pkt)
                        else:
                            break
            except socket.timeout:
                pass
            except Exception as e:
                if self.running:
                    print(f"Receive error: {e}")
                break
    
    def _handle_packet(self, pkt: dict):
        """Handle received packet."""
        ptype = pkt['type_str']
        
        if ptype == 'KA':
            # KeepAlive - ignore
            pass
        elif ptype == 'PV':
            # ParameterValue
            payload = pkt['payload']
            if len(payload) > 4:
                direction = payload[:4]
                rest = payload[4:]
                # Find null terminator for path
                null_idx = rest.find(b'\x00')
                if null_idx > 0:
                    path = rest[:null_idx].decode('utf-8', errors='replace')
                    # Value is last 4 bytes
                    if len(rest) >= null_idx + 5:
                        value_bytes = rest[-4:]
                        value = struct.unpack('<f', value_bytes)[0]
                        dir_str = 'daemon→client' if direction == DIR_DAEMON_TO_CLIENT else 'client→daemon'
                        print(f"  [{dir_str}] {path} = {value:.4f}")
        else:
            print(f"  Received: {ptype} ({len(pkt['payload'])} bytes)")
    
    def set_parameter(self, path: str, value: float, debug: bool = False):
        """Set a parameter value."""
        if not self.sock:
            print("Not connected")
            return
        
        packet = build_parameter_packet(path, value)
        if debug:
            print(f"DEBUG: Sending {len(packet)} bytes:")
            print(f"  {' '.join(f'{b:02x}' for b in packet)}")
        self.sock.send(packet)
        print(f"Sent: {path} = {value:.4f}")
    
    def set_main_volume(self, value: float, debug: bool = False):
        """Set main output volume (0.0 - 1.0)."""
        self.set_parameter('global/mainOutVolume', max(0.0, min(1.0, value)), debug)
    
    def set_headphone_volume(self, value: float, debug: bool = False):
        """Set headphone 1 volume (0.0 - 1.0)."""
        self.set_parameter('global/phones1_volume', max(0.0, min(1.0, value)), debug)
    
    def send_keepalive(self):
        """Send keepalive packet."""
        if self.sock:
            packet = build_keepalive_packet()
            self.sock.send(packet)


def main():
    print("=" * 60)
    print("UCDaemon Control Test")
    print("=" * 60)
    print()
    print("This script connects to ucdaemon and controls the Quantum HD 2")
    print()
    
    client = UCDaemonClient()
    
    if not client.connect():
        print("Failed to connect to ucdaemon")
        print("Make sure Universal Control is running and the Quantum is connected")
        sys.exit(1)
    
    print()
    print("Commands:")
    print("  m <value>  - Set main volume (0.0-1.0)")
    print("  h <value>  - Set headphone volume (0.0-1.0)")
    print("  p <path> <value> - Set arbitrary parameter")
    print("  k          - Send keepalive")
    print("  q          - Quit")
    print()
    
    try:
        while True:
            try:
                cmd = input("> ").strip()
            except EOFError:
                break
            
            if not cmd:
                continue
            
            parts = cmd.split()
            
            if parts[0] == 'q':
                break
            elif parts[0] == 'k':
                client.send_keepalive()
                print("Sent keepalive")
            elif parts[0] == 'm' and len(parts) >= 2:
                try:
                    value = float(parts[1])
                    client.set_main_volume(value, debug=True)
                except ValueError:
                    print("Invalid value")
            elif parts[0] == 'h' and len(parts) >= 2:
                try:
                    value = float(parts[1])
                    client.set_headphone_volume(value, debug=True)
                except ValueError:
                    print("Invalid value")
            elif parts[0] == 'p' and len(parts) >= 3:
                try:
                    path = parts[1]
                    value = float(parts[2])
                    client.set_parameter(path, value)
                except ValueError:
                    print("Invalid value")
            else:
                print("Unknown command")
    
    except KeyboardInterrupt:
        print()
    
    finally:
        client.disconnect()
    
    print("Done")


if __name__ == "__main__":
    main()
