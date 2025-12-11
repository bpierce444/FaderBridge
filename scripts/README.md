# UCNet Test Scripts

This directory contains test scripts for UCNet protocol communication with PreSonus devices.

## Working Scripts

### `test_mixer_control.py`

Controls a network-connected StudioLive mixer via UCNet over TCP.

**Usage:**
```bash
python3 test_mixer_control.py <mixer_ip> [value]

# Examples:
python3 test_mixer_control.py 192.168.1.209 0.5   # Set main fader to 50%
python3 test_mixer_control.py 192.168.1.209 0.1   # Set main fader to 10%
python3 test_mixer_control.py 192.168.1.209 0.9   # Set main fader to 90%
```

**Parameters:**
- `mixer_ip` - IP address of the StudioLive mixer
- `value` - Fader value from 0.0 to 1.0 (linear gain, not dB)

---

## UCNet Protocol Summary

### Key Findings (from reverse engineering)

1. **Connection**: TCP port 53000
2. **Packet format**: 
   - Magic bytes: `55 43 00 01`
   - Size: 2 bytes, **little-endian**, includes the 2-byte type
   - Type: 2 bytes (e.g., `55 4D` = Hello, `4A 4D` = JSON, `50 56` = ParameterValue)
   - Payload: variable

3. **Subscribe packet must use**:
   - `clientName`: "Universal Control"
   - `clientInternalName`: "ucapp"
   - `clientType`: "Mac"
   - CBytes: `72 00 65 00` ('r', 'e')

4. **Parameter Value (PV) packet format**:
   ```
   [Magic 4B] [Size 2B LE] [Type "PV"] [CBytes 4B] [Key + null] [Padding 2B] [Value 4B LE float]
   ```

5. **Parameter paths use forward slashes**: `main/ch1/volume`

6. **Values are linear gain** (0.0 to 1.0), not dB

### Handshake Sequence

1. Connect to mixer on TCP port 53000
2. Send Hello (UM) packet
3. Send Subscribe (JM) packet with correct fields
4. Wait for SubscriptionReply
5. Send ParameterValue (PV) packets to control parameters

---

## Future Work

- USB communication with Quantum HD 2 (via ucdaemon or direct USB)
- Additional parameter paths (channel faders, pan, mute, aux sends, etc.)
- Bidirectional sync (receive parameter changes from mixer)
