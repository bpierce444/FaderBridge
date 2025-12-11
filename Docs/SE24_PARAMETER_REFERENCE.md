# StudioLive SE24 UCNet Parameter Reference

**Date:** 2024-12-11  
**Device:** PreSonus StudioLive 24R (SE24)  
**Connection:** TCP port 53000

---

## Parameter Path Format

The SE24 uses hierarchical parameter paths in the format:
```
<category>/<channel>/<parameter>
<category>/<channel>/<subsystem>/<parameter>
```

### Confirmed Working Parameters

| Path | Type | Values | Description | Verified |
|------|------|--------|-------------|----------|
| `line/ch{N}/select` | float | 0.0 = off, 1.0 = on | Channel select | Yes |
| `line/ch{N}/volume` | float | 0.0 - 1.0 | Channel volume/fader | Yes |
| `line/ch{N}/pan` | float | 0.0 (L) - 1.0 (R) | Channel pan | Yes |
| `line/ch{N}/mute` | float | 0.0 = off, 1.0 = on | Channel mute | Yes |
| `line/ch{N}/solo` | float | 0.0 = off, 1.0 = on | Channel solo | Yes |
| `line/ch{N}/filter/hpf` | float | 0.0 - 1.0 | High-pass filter frequency | Yes (observed) |
| `line/ch{N}/gate/threshold` | float | 0.0 - 1.0 | Gate threshold | Yes (observed) |

### Parameters Found in State (Need Testing)

| Path | Type | Values | Description |
|------|------|--------|-------------|
| `line/ch{N}/stereopan` | float | 0.0 - 1.0 | Stereo pan |
| `line/ch{N}/fader` | float | 0.0 - 1.0 | Fader position (may be alias for volume) |
| `line/ch{N}/gain` | float | 0.0 - 1.0 | Input gain |
| `line/ch{N}/aux1` - `aux32` | float | 0.0 - 1.0 | Aux send levels |

### Channel Numbers

- **Line inputs:** `line/ch1` through `line/ch24` (or higher depending on model)
- **Channels observed:** ch1-ch28 in state dump

---

## State Structure

The mixer state is sent as compressed (zlib) data in CK packets containing ZB payloads.

### Binary Format

Properties use a tagged format:
- `i` + length byte + property name = identifier
- `d` + 4 bytes = float value (little-endian IEEE 754)
- `{` / `}` = object start/end
- `S` + string = string value

### Hierarchy Example

```
line {
  children {
    ch1 {
      values {
        name: "Ch. 1"
        select: 0.0
        solo: 0.0
        volume: 0.75
        mute: 0.0
        pan: 0.5
        stereopan: 0.5
      }
      children {
        filter { hpf: 0.0 }
        gate { threshold: 0.5 }
        eq { ... }
        opt { ... }
      }
    }
    ch2 { ... }
  }
}
```

---

## Other Categories Found

| Category | Description |
|----------|-------------|
| `permissions/` | User permission flags |
| `mastersection/` | Master section (anysolo, etc.) |
| `global/` | Global settings |
| `aux/` | Aux bus settings |
| `fxreturn/` | FX return channels |

---

## Protocol Notes

### Subscription

```json
{
  "id": "Subscribe",
  "clientName": "Universal Control",
  "clientInternalName": "ucapp",
  "clientType": "Mac",
  "clientOptions": "perm users levl redu rtan",
  "clientEncoding": 23106
}
```

### PV Packet Format

```
Magic:    55 43 00 01
Size:     little-endian u16 (includes 2-byte type)
Type:     50 56 (PV)
CBytes:   72 00 65 00 (r\0e\0)
Key:      ASCII null-terminated string
Padding:  00 00
Value:    little-endian f32
```

### Key Differences from Quantum HD 2

| Aspect | Quantum HD 2 | SE24 |
|--------|--------------|------|
| Path prefix | `global/` | `line/` |
| Volume path | `global/mainOutVolume` | `line/ch{N}/volume` |
| Connection | localhost:51801 (via ucdaemon) | device:53000 (direct) |

---

## Testing Scripts

- `discover_parameters.py` - Dump and analyze mixer state
- `listen_params.py` - Monitor real-time parameter changes
- `test_mixer_control.py` - Send parameter commands

### Quick Test

```bash
# Listen for changes (move faders, press buttons on mixer)
python3 listen_params.py 192.168.1.209 60

# Test mute control
python3 -c "
import socket, struct, json, time
# ... (see test_mixer_control.py for full implementation)
"
```

---

## Next Steps

1. **Verify volume/pan control** - These parameters exist in state but need physical verification
2. **Discover main output path** - Find the main fader parameter
3. **Map aux sends** - Test `line/ch{N}/aux{M}` paths
4. **EQ parameters** - Explore `line/ch{N}/eq/` structure
5. **Compressor/Gate** - Map dynamics parameters

---

## Changelog

- 2024-12-11: Confirmed all core channel parameters working: select, volume, pan, mute, solo, filter/hpf, gate/threshold
