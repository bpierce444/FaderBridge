# FaderBridge Setup Guide

## Quick Start: FaderPort → Series III Mixer

This guide will help you set up a PreSonus FaderPort to control a PreSonus Series III mixer.

---

## Prerequisites

### Hardware
- **MIDI Controller:** PreSonus FaderPort (any model)
- **UCNet Device:** PreSonus Series III Mixer (32SX, 24R, 16R, etc.)
- **Computer:** Mac with macOS 10.15+
- **Network:** Ethernet cable (recommended) or WiFi router

### Software
- FaderBridge (this application)
- Latest firmware on both devices

---

## Step 1: Connect Your Devices

### FaderPort (MIDI Controller)
1. Connect FaderPort to your Mac via **USB**
2. Power on the FaderPort
3. **Set FaderPort to MCU mode** (recommended):
   - Hold **Shift** + **User** buttons on power-up
   - Or use PreSonus Universal Control software
   - MCU mode provides best compatibility and motorized fader feedback
   - Alternative: Native mode also works but with limited features
4. Verify it appears in **Audio MIDI Setup**:
   - Open Applications > Utilities > Audio MIDI Setup
   - Window > Show MIDI Studio
   - FaderPort should appear in the device list

### Series III Mixer (UCNet Device)
1. Connect mixer to your network:
   - **Recommended:** Ethernet cable from mixer to router
   - **Alternative:** WiFi (if mixer supports it)
2. Configure mixer network settings:
   - On mixer: Setup > Network
   - Enable DHCP (easiest) or set static IP
   - Note the mixer's IP address (e.g., 192.168.1.100)
3. Verify network connection:
   - Open Terminal on Mac
   - Ping the mixer: `ping 192.168.1.100` (use your mixer's IP)
   - Should see replies (Ctrl+C to stop)

---

## Step 2: Launch FaderBridge

1. Open FaderBridge application
2. Wait for the main window to appear

---

## Step 3: Discover Devices

### Discover MIDI Devices
1. Click **"Discover MIDI Devices"** button
2. FaderPort should appear in the MIDI device list
3. Select your FaderPort from the list

**Troubleshooting:**
- If FaderPort doesn't appear, check USB connection
- Try unplugging and reconnecting the FaderPort
- Restart FaderBridge

### Discover UCNet Devices
1. Click **"Discover UCNet Devices"** button
2. Wait 2-3 seconds for network discovery
3. Your Series III mixer should appear in the UCNet device list

**Troubleshooting:**
- If mixer doesn't appear, see "Network Discovery Issues" below
- Verify mixer is on the same network
- Check firewall settings (UDP port 47809)
- Try manual connection by IP address

---

## Step 4: Connect to Mixer

1. Select your Series III mixer from the UCNet device list
2. Click **"Connect"** button
3. Status should change to "Connected"
4. Keep-alive task will start automatically

---

## Step 5: Create Parameter Mappings

### Using MIDI Learn (Recommended)
1. Click **"MIDI Learn"** button to enable learning mode
2. On the mixer, select the parameter you want to control (e.g., Channel 1 Fader)
3. Move a control on your FaderPort (e.g., Fader 1)
4. FaderBridge will automatically create the mapping
5. Repeat for other controls

### Manual Mapping
1. Click **"Add Mapping"** button
2. Select MIDI CC number (e.g., CC 7 for fader)
3. Enter UCNet parameter path (e.g., `line/ch1/vol`)
4. Choose taper curve (Linear for faders, Logarithmic for volume)
5. Click **"Save"**

---

## Step 6: Test Your Setup

1. Move a fader on your FaderPort
2. Watch the corresponding fader on the mixer move
3. Move the fader on the mixer
4. Watch the FaderPort fader move (bidirectional sync)
5. Check the on-screen visual feedback in FaderBridge

**Expected Behavior:**
- Latency should be < 10ms (nearly instant)
- Faders should move smoothly
- Activity lights should flash when parameters change

---

## Step 7: Save Your Project

1. Click **"Save Project"** button
2. Enter a project name (e.g., "Studio Setup")
3. Optional: Add description
4. Click **"Save"**

**Auto-Save:**
- FaderBridge auto-saves every 30 seconds
- Changes are saved automatically
- No need to manually save frequently

---

## Network Discovery Issues

### Mixer Not Appearing in UCNet List

#### Check Network Connection
```bash
# Terminal commands to diagnose

# 1. Check your Mac's IP address
ifconfig | grep "inet "

# 2. Ping the mixer (replace with your mixer's IP)
ping 192.168.1.100

# 3. Check if you're on the same subnet
# Your Mac: 192.168.1.50
# Mixer:    192.168.1.100
# ✅ Same subnet (192.168.1.x)

# Your Mac: 192.168.1.50
# Mixer:    10.0.0.100
# ❌ Different subnet - won't work
```

#### Check Firewall
1. System Settings > Network > Firewall
2. If enabled, click "Firewall Options"
3. Add FaderBridge to allowed apps
4. Or temporarily disable firewall for testing

#### Manual Connection (Workaround)
If discovery fails but you know the mixer's IP:
1. In FaderBridge, look for "Connect by IP" option
2. Enter mixer's IP address: `192.168.1.100`
3. Click "Connect"

---

## Controller Modes & Compatibility

### Recommended: MCU (Mackie Control Universal) Mode

**Why MCU Mode?**
- ✅ Industry standard protocol (supported by most DAW controllers)
- ✅ 14-bit fader resolution (smoother, more precise)
- ✅ Motorized fader feedback (faders move automatically)
- ✅ LED feedback for buttons and V-Pots
- ✅ Works with ANY MCU-compatible controller

**Supported Controllers (MCU Mode):**
- PreSonus FaderPort (all models)
- Behringer X-Touch (all models)
- Icon Platform M/M+/X/X+
- Mackie MCU Pro
- SSL Nucleus
- And many others...

### How to Enable MCU Mode

#### PreSonus FaderPort
1. **Power-up method:**
   - Hold **Shift** + **User** buttons while powering on
   - Release when LEDs flash
2. **Software method:**
   - Open PreSonus Universal Control
   - Select FaderPort
   - Choose "MCU" mode
   - Apply and reconnect

#### Behringer X-Touch
1. Press and hold **MC** button
2. Select "Mackie Control" mode
3. Press **Enter**
4. Device will reboot in MCU mode

#### Icon Platform Series
1. Press **Setup** button
2. Navigate to "MIDI Mode"
3. Select "Mackie Control"
4. Save and exit

### Alternative Modes

#### Native/Proprietary Mode
- ✅ Device-specific features may work
- ❌ No standardized protocol
- ❌ Limited motorized fader support
- ❌ Requires custom mapping per device
- **Use only if MCU mode unavailable**

#### HUI Mode (Older Standard)
- ⚠️ Legacy protocol (less common)
- ⚠️ 7-bit fader resolution (less smooth)
- ❌ Not recommended for FaderBridge
- **Use MCU mode instead**

### MCU Protocol Benefits

#### 14-bit Fader Resolution
```
Standard MIDI CC: 0-127 (128 steps)
MCU Pitch Bend:   0-16383 (16,384 steps)
Result: 128x smoother fader movement
```

#### Bidirectional Communication
- **To Mixer:** Fader movements, button presses
- **From Mixer:** Fader positions, LED states, display updates
- **Result:** Perfect sync between controller and mixer

#### LED Feedback
- Mute/Solo/Select button LEDs
- V-Pot LED rings (12-segment)
- Transport button LEDs
- **Result:** Visual confirmation of mixer state

---

## Common Parameter Paths

### Series III Mixer UCNet Parameters

#### Channel Strip
```
line/ch1/vol      - Channel 1 Volume
line/ch1/pan      - Channel 1 Pan
line/ch1/mute     - Channel 1 Mute
line/ch1/solo     - Channel 1 Solo
line/ch1/gain     - Channel 1 Preamp Gain
```

#### Master Section
```
main/lr/vol       - Main LR Fader
main/lr/mute      - Main LR Mute
```

#### Aux Sends
```
line/ch1/aux1     - Channel 1 Aux 1 Send
line/ch1/aux2     - Channel 1 Aux 2 Send
```

**Note:** Exact parameter paths may vary by mixer model. Use MIDI Learn to discover correct paths automatically.

---

## MIDI Protocol Reference

### MCU Protocol (Recommended)

#### Faders (Channels 0-7)
```
Pitch Bend Ch 0-7: Faders 1-8 (14-bit, 0-16383)
  - Much smoother than standard MIDI CC
  - Motorized fader feedback supported
  - 128x more resolution than 7-bit CC
```

#### V-Pots / Rotary Encoders (Channels 0-7)
```
CC 16-23: V-Pot rotation (relative encoding)
  - Clockwise: 0x01-0x0F (1-15 steps)
  - Counter-clockwise: 0x41-0x4F (-1 to -15 steps)
CC 48-55: V-Pot LED ring control
  - Bits 4-5: Mode (Single/BoostCut/Wrap/Spread)
  - Bits 0-3: Position (0-11)
```

#### Buttons (Note On/Off)
```
Note 0-7:   Record Ready buttons
Note 8-15:  Solo buttons
Note 16-23: Mute buttons
Note 24-31: Select buttons
Note 91:    Rewind
Note 92:    Fast Forward
Note 93:    Stop
Note 94:    Play
Note 95:    Record
```

### Standard MIDI CC Mode (Fallback)

#### FaderPort Classic / FaderPort 8
```
CC 0-7:   Faders 1-8 (7-bit, 0-127)
CC 16-23: Rotary Encoders 1-8
CC 32-39: Mute Buttons 1-8 (toggle)
CC 40-47: Solo Buttons 1-8 (toggle)
```

#### FaderPort 16
```
CC 0-15:  Faders 1-16 (7-bit, 0-127)
CC 16-31: Rotary Encoders 1-16
```

**Note:** Use MIDI Learn for automatic detection of your controller's protocol and CC assignments.

---

## Best Practices

### Network Setup
- ✅ **Use Ethernet** for lowest latency and most reliable connection
- ✅ **Static IP** on mixer prevents IP changes
- ✅ **Dedicated network** for audio devices (if possible)
- ❌ Avoid WiFi for critical live applications
- ❌ Don't use network switches with high latency

### Mapping Strategy
- Start with essential controls (channel faders, mute, pan)
- Group similar controls together
- Use consistent CC assignments across projects
- Save multiple projects for different scenarios

### Performance
- Close unnecessary applications
- Disable WiFi if using Ethernet
- Monitor CPU usage during use
- Test thoroughly before live use

---

## Next Steps

1. **Explore Advanced Features:**
   - Custom taper curves
   - Min/max value ranges
   - Multiple device control

2. **Create Multiple Projects:**
   - Studio Setup
   - Live Show
   - Recording Session

3. **Backup Your Projects:**
   - File > Export Project
   - Save JSON file to safe location
   - Import on other computers

4. **Read Full Documentation:**
   - `PRD_FaderBridge.md` - Product overview
   - `TROUBLESHOOTING.md` - Common issues
   - `PROJECT_CHARTER.md` - Feature roadmap

---

## Support

**Issues?** See `TROUBLESHOOTING.md`

**Feature Requests?** Check `PROJECT_CHARTER.md` for roadmap

**Questions?** Enable debug logging: `RUST_LOG=debug npm run tauri:dev`
