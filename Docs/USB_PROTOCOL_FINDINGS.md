# USB Protocol Findings: Quantum HD 2

**Date:** 2024-12-03 (Updated)  
**Device:** PreSonus/Fender Quantum HD 2  
**Firmware:** 3.01 (bcdDevice 0x0301)  
**USB IDs:** VID 0x1ed8, PID 0x020e

---

## USB Interface Structure

| Interface | Alt | Class | Endpoints | Purpose |
|-----------|-----|-------|-----------|---------|
| 0 | 0 | Audio (1) | EP 0x84 IN Interrupt | **Status notifications** |
| 1 | 0,1 | Audio (1) | EP 0x83 IN Isochronous | Audio input stream |
| 2 | 0,1 | Audio (1) | EP 0x03 OUT, 0x85 IN Iso | Audio output stream |
| 3 | 0 | Audio (1) | - | Audio control |
| 4 | 0 | Audio (1) | EP 0x02 OUT, 0x82 IN Bulk | **MIDI** |
| 5 | 0,1 | Vendor (255) | EP 0x01 OUT, 0x81 IN Bulk | **Control protocol** |
| 6 | 0 | DFU (254) | - | Firmware update |

---

## Interface 0: Status Notifications (EP 0x84)

### Message Format (6 bytes)

```
Byte 0: 0x00 (header)
Byte 1: 0x01 (header)
Byte 2: Channel (0x01 = Ch1, 0x02 = Ch2)
Byte 3: Parameter ID
Byte 4: 0x00 or 0x02
Byte 5: 0x03 or 0x04
```

### Known Parameter IDs (Byte 3)

| Value | Hex | Parameter |
|-------|-----|-----------|
| 2 | 0x02 | Input Gain |
| 226 | 0xE2 | 48V Phantom Power |
| 227 | 0xE3 | High Pass Filter (HPF) |
| 228 | 0xE4 | -20dB Pad |

### Pattern Analysis

- **Continuous parameters** (Gain): Low values (0x02)
- **Toggle parameters** (48V, HPF, Pad): High values (0xE2-0xE4)
- The 0xE0 offset (224) may indicate toggle vs continuous

### Not Captured on Interface 0

- Main output level
- Headphone output level
- Sample rate changes

These may be:
1. Analog-only (no digital feedback)
2. Reported on Interface 5 (control interface)
3. Using a different message format

---

## Interface 4: MIDI (EP 0x82 IN, EP 0x02 OUT)

Standard USB MIDI interface. No traffic observed during hardware adjustments.
May be used for:
- External MIDI I/O
- DAW control surface features (if any)

---

## Interface 5: Control Protocol (EP 0x81 IN, EP 0x01 OUT)

**Status:** Controlled via ucdaemon which exposes UCNet over TCP.

### Key Discovery

`ucdaemon` acts as a bridge:
- USB ↔ UCNet translation
- Listens on TCP port **51801** for local connections
- Uses standard UCNet protocol format

### ucdaemon Architecture

```
┌─────────────────┐     USB      ┌──────────────────┐
│  Quantum HD 2   │◄────────────►│    ucdaemon      │
│  (Hardware)     │              │  (TCP :51801)    │
└─────────────────┘              └────────┬─────────┘
                                          │ UCNet/TCP
                                          ▼
                                 ┌──────────────────┐
                                 │ Universal Control │
                                 │   (or FaderBridge)│
                                 └──────────────────┘
```

### Loaded Device Drivers

- `quantumusbdevice.dylib` (12MB) - Quantum HD 2 USB driver
- `quantumdevice.dylib` - Quantum network driver
- `studiolive3device.dylib` - StudioLive Series III
- `faderportdevice.dylib` - FaderPort controllers

---

## UCNet Protocol over TCP (Port 51801)

### Packet Format

```
Bytes 0-3:  Magic "UC\x00\x01" (0x55 0x43 0x00 0x01)
Bytes 4-5:  Payload size (little-endian)
Bytes 6-7:  Payload type code
Bytes 8+:   Payload data
```

### Payload Types Observed

| Code | ASCII | Description |
|------|-------|-------------|
| `4B 41` | KA | KeepAlive |
| `50 56` | PV | ParameterValue |

### ParameterValue Format (PV)

```
Bytes 0-1:  Direction indicator
            - "h\x00u\x00" = host → unit (command)
            - "u\x00h\x00" = unit → host (response)
Bytes 2+:   Parameter path (null-terminated UTF-16LE string)
Last 4:     Value (IEEE 754 float, little-endian)
```

### Known Parameters

| Parameter Path | Description | Value Range |
|----------------|-------------|-------------|
| `global/mainOutVolume` | Main output level | 0.0 - 1.0 (float) |
| `global/phones1_volume` | Headphone 1 level | 0.0 - 1.0 (float) |

### Example Packets

**Main Output Volume (set to ~0.33):**
```
55 43 00 01              # Magic
21 00                    # Size: 33 bytes
50 56                    # Type: PV (ParameterValue)
68 00 75 00              # Direction: h.u. (host to unit)
67 6c 6f 62 61 6c 2f     # "global/"
6d 61 69 6e 4f 75 74     # "mainOut"
56 6f 6c 75 6d 65 00     # "Volume\0"
00 d5 f5 a8 3e           # Float value ~0.33
```

**Headphone Volume (set to ~0.775):**
```
55 43 00 01              # Magic
22 00                    # Size: 34 bytes  
50 56                    # Type: PV (ParameterValue)
75 00 68 00              # Direction: u.h. (unit to host)
67 6c 6f 62 61 6c 2f     # "global/"
70 68 6f 6e 65 73 31     # "phones1"
5f 76 6f 6c 75 6d 65 00  # "_volume\0"
00 68 66 46 3f           # Float value ~0.775
```

---

## Connection to ucdaemon

To control the Quantum HD 2:

1. Connect to `localhost:51801` via TCP
2. Send UCNet Hello packet to establish session
3. Send/receive ParameterValue packets
4. Maintain connection with KeepAlive packets (~1.5s interval)

---

## Raw Captures

### Sample Status Messages

```
# Input gain adjustments
000101020004  - Channel 1 gain
000102020004  - Channel 2 gain

# Toggle parameters
000101e20003  - Channel 1 48V phantom
000102e20003  - Channel 2 48V phantom
000101e30003  - Channel 1 HPF
000102e30003  - Channel 2 HPF
000101e40003  - Channel 1 Pad
000102e40003  - Channel 2 Pad
```

---

## Implications for FaderBridge

### What We Can Do Now

1. **Full bidirectional control** via UCNet to ucdaemon (TCP :51801)
2. **Monitor input channel status** via USB Interface 0
3. **Send/receive MIDI** via USB Interface 4
4. **Control Main/Headphone levels** via UCNet ParameterValue packets

### Recommended Architecture

```
┌─────────────────────────────────────────────────────┐
│                    FaderBridge                       │
├─────────────────────────────────────────────────────┤
│  UCNet Transport (Primary)                           │
│  ├── TCP localhost:51801 (via ucdaemon)             │
│  ├── TCP device:53000 (direct network connection)   │
│  └── Full parameter control                          │
├─────────────────────────────────────────────────────┤
│  USB Transport (Supplementary)                       │
│  ├── Interface 0: Real-time status monitoring       │
│  └── Interface 4: MIDI I/O                          │
└─────────────────────────────────────────────────────┘
```

### Implementation Strategy

1. **Primary control**: Connect to ucdaemon via TCP :51801
   - Use existing UCNet protocol implementation
   - Works for all USB-connected PreSonus devices
   
2. **Real-time monitoring**: USB Interface 0 for low-latency status
   - 6-byte status messages for input parameters
   - Faster than polling via UCNet

3. **MIDI**: USB Interface 4 for MIDI I/O
   - Standard USB MIDI class
   - No special protocol needed
