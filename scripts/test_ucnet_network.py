#!/usr/bin/env python3
"""\
Simple UCNet test client for network-connected StudioLive mixers.

This talks directly to a mixer over TCP UCNet (port 53000), using the
same packet formats as the Rust UCNet implementation in this project.

Usage:
    python3 scripts/test_ucnet_network.py <mixer_ip>

Then use commands like:
    m 0.5   -> set main mix fader to 0.5 (0.0 - 1.0)
    q       -> quit
"""

import socket
import struct
import sys
import json
import threading
import time
from typing import Optional

UCNET_MAGIC = b"\x55\x43\x00\x01"  # "UC\x00\x01"
UCNET_PORT = 53000

TYPE_HELLO = b"\x55\x4d"      # "UM"
TYPE_JSON = b"\x4a\x4d"       # "JM"
TYPE_KEEPALIVE = b"\x4b\x41"  # "KA"
TYPE_PARAM_SET = b"\x50\x53"  # "PS"
TYPE_PARAM_VALUE = b"\x50\x56"  # "PV" - what UC actually uses for fader changes!


def build_packet(payload_type: bytes, payload: bytes) -> bytes:
    """Build a UCNet packet.

    Header layout (8 bytes):
      0-3: MAGIC (55 43 00 01)
      4-5: size (little-endian u16) = len(type) + len(payload) = 2 + len(payload)
      6-7: payload type (e.g., "UM", "JM", "PV")
    
    From tcpdump: UC uses little-endian size that INCLUDES the 2-byte type.
    """
    # Size includes the 2-byte type + payload
    size_le = struct.pack("<H", 2 + len(payload))
    return UCNET_MAGIC + size_le + payload_type + payload


def build_hello_packet() -> bytes:
    """Build a Hello (UM) packet.

    From protocol.rs:
      payload = [0x00, 0x00, 0x65, 0x00, 0x15, 0xFA]
    Full example:
      55 43 00 01 08 00 55 4d 00 00 65 00 15 fa
    """
    payload = b"\x00\x00\x65\x00\x15\xFA"
    return build_packet(TYPE_HELLO, payload)


def build_subscribe_packet() -> bytes:
    """Build a Subscribe (JM) packet using the Rust default SubscribeRequest."""
    # Matches SubscribeRequest::default() in protocol.rs
    req = {
        "id": "Subscribe",
        "clientName": "FaderBridge",
        "clientInternalName": "faderbridge",
        "clientType": "PC",
        "clientDescription": "FaderBridge MIDI Controller",
        "clientIdentifier": "FaderBridge",
        # perm=permissions, users=user list, levl=levels, redu=redux, rtan=real-time analysis
        "clientOptions": "perm users levl redu rtan",
        "clientEncoding": 23106,
    }

    json_bytes = json.dumps(req).encode("utf-8")

    # CBytes::new() => 'j' 'e' with nulls in between: 6a 00 65 00
    cbytes = b"\x6a\x00\x65\x00"
    json_len_le = struct.pack("<I", len(json_bytes))

    payload = cbytes + json_len_le + json_bytes
    return build_packet(TYPE_JSON, payload)


def build_keepalive_packet() -> bytes:
    """Build a KeepAlive (KA) packet.

    For network UCNet, Rust uses an empty payload for KA.
    """
    return build_packet(TYPE_KEEPALIVE, b"")


def build_parameter_set_packet(key: str, value: float) -> bytes:
    """Build a ParameterSet (PS) packet for a float parameter.

    Mirrors ParameterValue::to_set_payload in protocol.rs:
      payload = CBytes + key + 00 + 00 00 + value_le

    CBytes here is the default 'j' 'e' pattern.
    """
    cbytes = b"\x6a\x00\x65\x00"  # CBytes::new()
    key_bytes = key.encode("ascii") + b"\x00"  # null-terminated
    part_a = b"\x00\x00"  # normal parameter (not filter group)
    value_bytes = struct.pack("<f", value)  # little-endian f32

    payload = cbytes + key_bytes + part_a + value_bytes
    return build_packet(TYPE_PARAM_SET, payload)


def build_parameter_value_packet(key: str, value: float) -> bytes:
    """Build a ParameterValue (PV) packet - what UC actually sends for fader changes.

    From tcpdump capture of UC talking to mixer (fresh capture after UC restart):
      55 43 00 01 1c 00 50 56 72 00 65 00 6d 61 69 6e 2f 63 68 31 2f 76 6f 6c 75 6d 65 00 00 00 XX XX XX XX
      
    Structure:
      - Magic: 55 43 00 01
      - Size: 1c 00 (little-endian, 28 bytes)
      - Type: 50 56 (PV)
      - CBytes: 72 00 65 00 ('r' 'e' with nulls) - NOT 't' 'e'!
      - Key: main/ch1/volume + null (uses forward slash!)
      - Padding: 00 00
      - Value: 4 bytes little-endian float
    """
    cbytes = b"\x72\x00\x65\x00"  # 'r' 'e' - from fresh UC capture
    key_bytes = key.encode("ascii") + b"\x00"  # null-terminated
    padding = b"\x00\x00"
    value_bytes = struct.pack("<f", value)  # little-endian f32

    payload = cbytes + key_bytes + padding + value_bytes
    return build_packet(TYPE_PARAM_VALUE, payload)


def recv_loop(sock: socket.socket, stop_flag) -> None:
    """Background receiver to show any UCNet packets returned by the mixer."""
    sock.settimeout(0.5)
    buffer = b""
    while not stop_flag[0]:
        try:
            data = sock.recv(4096)
            if not data:
                break
            buffer += data
            # Just show raw hex for now, don't fully parse
            print(f"[RX] {len(data)} bytes: "
                  f"{' '.join(f'{b:02x}' for b in data[:64])}" +
                  (" ..." if len(data) > 64 else ""))
        except socket.timeout:
            continue
        except OSError:
            break


def do_handshake(sock: socket.socket) -> None:
    """Perform Hello + Subscribe + initial KeepAlive, then read a bit of state."""
    # Send Hello
    hello = build_hello_packet()
    sock.sendall(hello)
    print("Sent: Hello (UM)")
    time.sleep(0.1)

    # Send Subscribe
    sub = build_subscribe_packet()
    sock.sendall(sub)
    print("Sent: Subscribe (JM)")
    time.sleep(0.2)

    # Initial KeepAlive
    ka = build_keepalive_packet()
    sock.sendall(ka)
    print("Sent: KeepAlive (KA)")


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: python3 scripts/test_ucnet_network.py <mixer_ip>")
        print("Example: python3 scripts/test_ucnet_network.py 192.168.1.209")
        return

    host = sys.argv[1]
    addr = (host, UCNET_PORT)

    print("============================================================")
    print("UCNet Network Device Test")
    print("============================================================")
    print(f"Target mixer: {host}:{UCNET_PORT}")
    print()

    sock: Optional[socket.socket] = None

    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3.0)
        sock.connect(addr)
        print("Connected to mixer")

        # Start background receiver
        stop_flag = [False]
        t = threading.Thread(target=recv_loop, args=(sock, stop_flag))
        t.daemon = True
        t.start()

        # Handshake
        do_handshake(sock)

        print("\nCommands:")
        print("  m <value>  - Set main volume (0.0 - 1.0)")
        print("  q          - Quit")
        print()

        while True:
            try:
                line = input("> ").strip()
            except (EOFError, KeyboardInterrupt):
                break

            if not line:
                continue

            parts = line.split()
            cmd = parts[0].lower()

            if cmd == "q":
                break
            elif cmd == "m" and len(parts) >= 2:
                try:
                    value = float(parts[1])
                except ValueError:
                    print("Invalid value; expected a float between 0.0 and 1.0")
                    continue

                # Clamp to 0..1 as the Rust API expects
                value = max(0.0, min(1.0, value))
                # UC uses forward slashes, not dots!
                key = "main/ch1/volume"

                # Use PV packet (what UC actually sends), not PS
                packet = build_parameter_value_packet(key, value)
                print(f"[TX] {key} = {value:.4f} ({len(packet)} bytes)")
                print("      " + " ".join(f"{b:02x}" for b in packet))
                try:
                    sock.sendall(packet)
                except OSError as e:
                    print(f"Send failed: {e}")
                    break
            else:
                print("Unknown command. Use 'm <value>' or 'q'.")

    except Exception as e:
        print(f"Error: {e}")

    finally:
        if sock is not None:
            try:
                sock.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            sock.close()
        print("Disconnected")


if __name__ == "__main__":
    main()
