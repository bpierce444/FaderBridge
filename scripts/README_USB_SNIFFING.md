# USB Protocol Sniffing for PreSonus Interfaces

## Discovery Summary

### Quantum HD 2 USB Interfaces

| Interface # | Name | Class | Owner | Purpose |
|-------------|------|-------|-------|---------|
| 0-3 | Audio Streaming | 1 (Audio) | TUsbAudioDriver | Audio I/O |
| 4 | MIDI | 1.3 (MIDI) | MIDIServer | Standard USB MIDI |
| **5** | **Quantum HD 2 Control** | **255 (Vendor)** | **ucdaemon** | **Control protocol** |
| 6 | Quantum HD 2 DFU | 254 (DFU) | (none) | Firmware update |

### Key Finding
The control protocol uses **Interface #5** with:
- **Vendor ID:** `0x1ed8` (PreSonus/Fender)
- **Product ID:** `0x020e` (Quantum HD 2)
- **Interface Class:** 255 (Vendor-specific)
- **Endpoints:** 2 (likely IN and OUT bulk/interrupt)

### Current Owner
`ucdaemon` (PreSonus Universal Control daemon) has exclusive access to this interface.

---

## Sniffing Options

### Option 1: Disable SIP (Not Recommended for Production)
```bash
# Boot to Recovery Mode (Cmd+R on Intel, hold power on Apple Silicon)
# Open Terminal and run:
csrutil disable

# After reboot, dtrace will work fully
sudo dtrace -s scripts/sniff_ucdaemon.d
```

### Option 2: Use a VM with USB Passthrough
1. Install VMware Fusion, Parallels, or UTM
2. Install Windows or Linux in the VM
3. Pass through the Quantum HD 2 USB device
4. Use Wireshark with USBPcap (Windows) or usbmon (Linux)

### Option 3: Hardware USB Analyzer
- Total Phase Beagle USB 480 (~$1,200)
- Inline capture without software limitations

### Option 4: Stop ucdaemon and Write Our Own Client
```bash
# Stop the daemon (will disconnect Universal Control)
sudo launchctl unload /Library/LaunchDaemons/com.presonus.ucdaemon.plist

# Now we can claim Interface #5 ourselves
# Use libusb or IOKit to communicate directly
```

### Option 5: Reverse Engineer ucdaemon
The daemon links against `ucnet.framework` which contains:
- `UCGetNetworkManager`
- `UCGetMIDINetworkManager`
- `UCGetFileTransferManager`

These suggest the USB protocol may be similar to the network UCNet protocol.

---

## Hypothesis: USB Protocol = UCNet over USB

Based on the framework names and the fact that both USB and network devices use the same
Universal Control app, the USB protocol is likely:

1. **Same message format** as network UCNet (magic bytes, payload types, JSON/ZLIB)
2. **Transported over USB bulk endpoints** instead of TCP
3. **No discovery phase** (device is directly connected)

### Test Plan
1. Stop `ucdaemon`
2. Open Interface #5 with libusb
3. Send a UCNet "Hello" packet (UM message)
4. See if the device responds

---

## Files Created

- `scripts/sniff_ucdaemon.d` - DTrace script (requires SIP disabled)
- `scripts/sniff_usb_iokit.d` - IOKit-specific DTrace script
- `scripts/capture_ucdaemon.sh` - Shell wrapper for capture

## Next Steps

1. **Test the hypothesis** that USB uses UCNet protocol format
2. **Create a Rust USB client** using `rusb` crate to communicate with Interface #5
3. **Document the protocol** differences between USB and network transport
