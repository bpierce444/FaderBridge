# USB Packet Capture via Linux VM

This guide sets up a Linux VM to capture USB traffic from the Quantum HD 2's control interface.

## Prerequisites

- **UTM** (free, recommended for Apple Silicon) or **VMware Fusion**
- ~20GB disk space
- Ubuntu 22.04 or 24.04 ISO

---

## Step 1: Install UTM

```bash
brew install --cask utm
```

Or download from: https://mac.getutm.app/

---

## Step 2: Create Ubuntu VM

1. Open UTM → **Create a New Virtual Machine**
2. Select **Virtualize** (faster on Apple Silicon)
3. Select **Linux**
4. Browse to Ubuntu ISO
5. Configure:
   - **RAM:** 4096 MB
   - **CPU:** 2 cores
   - **Storage:** 20 GB
6. **Important:** Check **"Enable USB Sharing"** in settings
7. Complete Ubuntu installation

---

## Step 3: Install Capture Tools in VM

After Ubuntu boots, open Terminal and run:

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Wireshark and USB tools
sudo apt install -y wireshark tshark usbutils

# Allow non-root USB capture
sudo usermod -aG wireshark $USER

# Enable usbmon kernel module
sudo modprobe usbmon

# Verify usbmon is loaded
lsmod | grep usbmon
```

Log out and back in for group changes to take effect.

---

## Step 4: Stop ucdaemon on macOS Host

Before passing the USB device to the VM, release it from macOS:

```bash
# On macOS host - stop Universal Control daemon
sudo launchctl unload /Library/LaunchDaemons/com.presonus.ucdaemon.plist

# Verify it's stopped
ps aux | grep ucdaemon
```

**Note:** This will disconnect Universal Control. Restart with:
```bash
sudo launchctl load /Library/LaunchDaemons/com.presonus.ucdaemon.plist
```

---

## Step 5: Pass USB Device to VM

### In UTM:
1. With VM running, click the **USB icon** in the toolbar
2. Select **"Quantum HD 2"** to pass it to the VM
3. The device will disconnect from macOS and appear in Linux

### Verify in Linux VM:
```bash
# List USB devices
lsusb

# Should show:
# Bus 00X Device 00Y: ID 1ed8:020e Fender Quantum HD 2

# Get detailed info
lsusb -v -d 1ed8:020e 2>/dev/null | head -100
```

---

## Step 6: Identify USB Bus for Capture

```bash
# Find which usbmon interface to capture
# The bus number from lsusb tells you which usbmonX to use
# e.g., Bus 001 → usbmon1

# List available capture interfaces
sudo tshark -D | grep usbmon
```

---

## Step 7: Start Capture

### Option A: Wireshark GUI
```bash
sudo wireshark &
# Select usbmon1 (or appropriate bus)
# Start capture
```

### Option B: Command Line (tshark)
```bash
# Capture to file
sudo tshark -i usbmon1 -w quantum_capture.pcapng

# Or capture with live display
sudo tshark -i usbmon1 -Y "usb.device_address == Y" 
# Replace Y with device address from lsusb
```

### Option C: Filtered Capture (Recommended)
```bash
# Capture only Quantum HD 2 traffic (vendor 0x1ed8)
sudo tshark -i usbmon1 \
  -f "host 1ed8:020e" \
  -w quantum_hd2_$(date +%Y%m%d_%H%M%S).pcapng
```

---

## Step 8: Generate Traffic to Capture

With capture running, you need to send commands to the Quantum. Options:

### Option A: Use Linux ALSA/PulseAudio
The audio interfaces should appear in Linux. Changing volume may generate control traffic.

### Option B: Write a Test Script
Create a simple libusb script to send UCNet-like packets:

```bash
# Install libusb development files
sudo apt install -y libusb-1.0-0-dev python3-usb

# Python test script
python3 << 'EOF'
import usb.core
import usb.util

# Find Quantum HD 2
dev = usb.core.find(idVendor=0x1ed8, idProduct=0x020e)
if dev is None:
    print("Device not found")
    exit(1)

print(f"Found: {dev}")

# List configurations
for cfg in dev:
    print(f"Config {cfg.bConfigurationValue}")
    for intf in cfg:
        print(f"  Interface {intf.bInterfaceNumber}, Class {intf.bInterfaceClass}")
        for ep in intf:
            print(f"    Endpoint {ep.bEndpointAddress:02x}")
EOF
```

### Option C: Send UCNet Hello Packet
```python
# test_ucnet_usb.py
import usb.core
import usb.util
import struct

# UCNet magic bytes
MAGIC = bytes([0x55, 0x43, 0x00, 0x01])  # "UC\x00\x01"

# Find device
dev = usb.core.find(idVendor=0x1ed8, idProduct=0x020e)
if dev is None:
    raise ValueError("Quantum HD 2 not found")

# Detach kernel driver if attached
if dev.is_kernel_driver_active(5):  # Interface 5 = Control
    dev.detach_kernel_driver(5)

# Set configuration
dev.set_configuration()

# Claim interface 5 (Control)
usb.util.claim_interface(dev, 5)

# Find endpoints on interface 5
cfg = dev.get_active_configuration()
intf = cfg[(5, 0)]  # Interface 5, alternate setting 0

ep_out = usb.util.find_descriptor(
    intf,
    custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress) == usb.util.ENDPOINT_OUT
)

ep_in = usb.util.find_descriptor(
    intf,
    custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress) == usb.util.ENDPOINT_IN
)

print(f"OUT endpoint: {ep_out.bEndpointAddress:02x}")
print(f"IN endpoint: {ep_in.bEndpointAddress:02x}")

# Build UCNet Hello packet (UM type)
# Format: MAGIC + payload_type + c_bytes + length + payload
payload_type = bytes([0x55, 0x4D])  # "UM" = Hello
c_bytes = bytes([0x6A, 0x00, 0x65, 0x00])  # Default c-bytes
payload = b'{"id":"FaderBridge","version":"1.0"}'
length = struct.pack('<H', len(payload))

packet = MAGIC + payload_type + c_bytes + length + payload

print(f"Sending: {packet.hex()}")

# Send packet
try:
    ep_out.write(packet)
    print("Sent successfully")
    
    # Try to read response
    response = ep_in.read(512, timeout=1000)
    print(f"Response: {bytes(response).hex()}")
except Exception as e:
    print(f"Error: {e}")

# Release interface
usb.util.release_interface(dev, 5)
```

---

## Step 9: Analyze Capture

### In Wireshark:
1. Open the `.pcapng` file
2. Filter: `usb.transfer_type == 0x03` (Interrupt) or `usb.transfer_type == 0x02` (Bulk)
3. Look for packets to/from the Control interface

### Key Fields:
- **usb.endpoint_address** - Which endpoint (IN/OUT)
- **usb.data_len** - Payload size
- **usb.capdata** - Raw data bytes

### Export Data:
```bash
# Extract just the USB data payloads
tshark -r quantum_capture.pcapng \
  -Y "usb.capdata" \
  -T fields -e usb.capdata > payloads.txt
```

---

## Step 10: Document Findings

After capturing traffic, look for:

1. **Magic bytes** - Does it start with `0x55 0x43 0x00 0x01` (UCNet)?
2. **Payload types** - `UM`, `JM`, `PV`, `KA`, etc.
3. **JSON content** - Parameter names, channel IDs
4. **Binary patterns** - Fixed-size control messages

Create a protocol document with:
- Message format
- Parameter IDs for faders, mutes, etc.
- Request/response patterns

---

## Troubleshooting

### Device not appearing in VM
- Ensure ucdaemon is stopped on macOS
- Try unplugging and replugging the USB cable
- Check UTM USB settings

### Permission denied on usbmon
```bash
sudo chmod 666 /dev/usbmon*
```

### No traffic captured
- Verify correct usbmon interface (matches USB bus)
- Check device address filter
- Try capturing without filters first

### Device resets when claimed
- The device may require specific initialization sequence
- Try claiming interface without setting configuration

---

## Quick Reference

```bash
# macOS: Stop ucdaemon
sudo launchctl unload /Library/LaunchDaemons/com.presonus.ucdaemon.plist

# macOS: Start ucdaemon
sudo launchctl load /Library/LaunchDaemons/com.presonus.ucdaemon.plist

# Linux: Quick capture
sudo tshark -i usbmon1 -w capture.pcapng

# Linux: View capture
wireshark capture.pcapng &
```
