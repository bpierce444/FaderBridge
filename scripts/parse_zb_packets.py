#!/usr/bin/env python3
"""
Parse ZB (compressed) packets from SE24 UCNet state dump.

The mixer sends most of its state in zlib-compressed ZB packets.
This script decompresses and analyzes them to find parameter paths.
"""

import struct
import sys
import zlib
import re
from pathlib import Path

MAGIC = b'\x55\x43\x00\x01'


def find_packets(data: bytes) -> list:
    """Find all UCNet packets in raw data."""
    packets = []
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
        packets.append({
            'offset': idx,
            'size': size,
            'type': ptype,
            'type_str': ptype.decode('ascii', errors='replace'),
            'payload': payload
        })
        pos = idx + 6 + size
    
    return packets


def decompress_zb(payload: bytes) -> bytes:
    """Decompress ZB packet payload."""
    # ZB format: 4 bytes cbytes, then metadata, then zlib data
    # Look for zlib magic (78 01, 78 9c, 78 da)
    for i in range(len(payload) - 2):
        if payload[i] == 0x78 and payload[i+1] in (0x01, 0x9c, 0xda):
            try:
                decompressed = zlib.decompress(payload[i:])
                return decompressed
            except zlib.error:
                continue
    return b''


def extract_strings(data: bytes, min_len: int = 4) -> list:
    """Extract printable ASCII strings from binary data."""
    strings = []
    current = []
    
    for byte in data:
        if 32 <= byte < 127:
            current.append(chr(byte))
        else:
            if len(current) >= min_len:
                strings.append(''.join(current))
            current = []
    
    if len(current) >= min_len:
        strings.append(''.join(current))
    
    return strings


def find_parameter_paths(strings: list) -> list:
    """Find strings that look like parameter paths."""
    paths = []
    # Pattern: word/word or word/word/word etc
    path_pattern = re.compile(r'^[a-zA-Z][a-zA-Z0-9_]*(/[a-zA-Z][a-zA-Z0-9_]*)+$')
    
    for s in strings:
        if path_pattern.match(s):
            paths.append(s)
    
    return paths


def analyze_decompressed(data: bytes) -> dict:
    """Analyze decompressed data structure."""
    result = {
        'size': len(data),
        'strings': [],
        'paths': [],
        'floats': [],
    }
    
    # Extract strings
    strings = extract_strings(data, min_len=3)
    result['strings'] = strings
    
    # Find parameter paths
    paths = find_parameter_paths(strings)
    result['paths'] = paths
    
    # Look for float values (common mixer values 0.0-1.0)
    for i in range(0, len(data) - 4, 4):
        try:
            val = struct.unpack('<f', data[i:i+4])[0]
            if 0.0 <= val <= 1.0 and val not in (0.0, 1.0):
                result['floats'].append((i, val))
        except:
            pass
    
    return result


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 parse_zb_packets.py <raw_dump.bin>")
        print("\nLooks for .bin files in current directory:")
        for f in Path('.').glob('se24_raw_*.bin'):
            print(f"  {f}")
        return
    
    filename = sys.argv[1]
    print(f"Reading {filename}...")
    
    with open(filename, 'rb') as f:
        data = f.read()
    
    print(f"File size: {len(data)} bytes")
    
    # Find all packets
    packets = find_packets(data)
    print(f"Found {len(packets)} packets")
    
    # Count by type
    type_counts = {}
    for pkt in packets:
        t = pkt['type_str']
        type_counts[t] = type_counts.get(t, 0) + 1
    
    print("\nPacket types:")
    for t, count in sorted(type_counts.items()):
        print(f"  {t}: {count}")
    
    # Process ZB packets
    zb_packets = [p for p in packets if p['type_str'] == 'ZB']
    print(f"\n{'='*60}")
    print(f"Analyzing {len(zb_packets)} ZB (compressed) packets")
    print('='*60)
    
    all_paths = set()
    all_strings = set()
    
    for i, pkt in enumerate(zb_packets):
        print(f"\nZB packet {i+1} at offset 0x{pkt['offset']:x}, payload size {len(pkt['payload'])}")
        
        decompressed = decompress_zb(pkt['payload'])
        if decompressed:
            print(f"  Decompressed: {len(decompressed)} bytes")
            
            analysis = analyze_decompressed(decompressed)
            
            # Show paths found
            if analysis['paths']:
                print(f"  Found {len(analysis['paths'])} parameter paths:")
                for path in sorted(set(analysis['paths']))[:100]:
                    all_paths.add(path)
                    print(f"    {path}")
                if len(analysis['paths']) > 100:
                    print(f"    ... and {len(analysis['paths']) - 100} more")
            
            # Collect all strings
            all_strings.update(analysis['strings'])
            
            # Save decompressed data
            out_file = f"zb_decompressed_{i+1}.bin"
            with open(out_file, 'wb') as f:
                f.write(decompressed)
            print(f"  Saved to: {out_file}")
        else:
            print("  Failed to decompress")
    
    # Process BO packets
    bo_packets = [p for p in packets if p['type_str'] == 'BO']
    if bo_packets:
        print(f"\n{'='*60}")
        print(f"Analyzing {len(bo_packets)} BO (bulk object) packets")
        print('='*60)
        
        for i, pkt in enumerate(bo_packets):
            print(f"\nBO packet {i+1} at offset 0x{pkt['offset']:x}")
            strings = extract_strings(pkt['payload'], min_len=3)
            if strings:
                print(f"  Strings: {strings[:20]}")
    
    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print('='*60)
    
    print(f"\nUnique parameter paths found: {len(all_paths)}")
    if all_paths:
        # Categorize by prefix
        categories = {}
        for path in all_paths:
            prefix = path.split('/')[0]
            if prefix not in categories:
                categories[prefix] = []
            categories[prefix].append(path)
        
        for cat in sorted(categories.keys()):
            paths = categories[cat]
            print(f"\n[{cat}] ({len(paths)} paths)")
            for p in sorted(paths)[:30]:
                print(f"  {p}")
            if len(paths) > 30:
                print(f"  ... and {len(paths) - 30} more")
    
    # Save all paths to file
    if all_paths:
        with open('se24_paths.txt', 'w') as f:
            for path in sorted(all_paths):
                f.write(f"{path}\n")
        print(f"\nAll paths saved to: se24_paths.txt")
    
    # Look for interesting strings that might be parameter-related
    print(f"\n{'='*60}")
    print("Interesting strings (potential parameters)")
    print('='*60)
    
    interesting = []
    for s in all_strings:
        s_lower = s.lower()
        if any(kw in s_lower for kw in ['volume', 'pan', 'mute', 'solo', 'gain', 'fader', 
                                          'eq', 'comp', 'gate', 'aux', 'bus', 'main',
                                          'channel', 'input', 'output', 'mix', 'send']):
            interesting.append(s)
    
    for s in sorted(set(interesting))[:50]:
        print(f"  {s}")


if __name__ == "__main__":
    main()
