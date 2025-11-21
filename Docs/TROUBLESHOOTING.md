# FaderBridge Troubleshooting Guide

## Device Discovery Issues

### MIDI Devices Change to "Generic" After UCNet Refresh (KNOWN ISSUE)
**Problem:** When clicking "Refresh" on the Mixers & Interfaces panel, MIDI controller names change to generic names like "MIDI Input".

**Temporary Workaround:**
1. Click "Refresh" on the Controllers panel to restore proper names
2. Avoid clicking UCNet refresh unless necessary
3. Restart app if names don't restore

**Status:** Under investigation - appears to be a state management or caching issue

**Technical Notes:**
- MIDI and UCNet discovery should be independent
- Issue may be related to device enumeration timing
- Debug with: `RUST_LOG=debug npm run tauri:dev`

---

### FaderPort Shows Up as UCNet Device (FIXED)
**Problem:** FaderPort (or other MIDI controllers) appear in the UCNet device list instead of MIDI device list.

**Solution:** This has been fixed in the latest version. FaderPort devices are now correctly filtered out of UCNet discovery and should only appear in the MIDI device list.

**Technical Details:**
- FaderPort devices use PreSonus Vendor ID (0x194f) but are MIDI controllers, not UCNet devices
- UCNet discovery now filters by product ID to only include actual UCNet-capable devices:
  - Quantum HD8 (0x8186)
  - Quantum HD4 (0x8187)
  - Quantum 26x32 (0x8188)
  - Quantum 4848 (0x8189)

---

## Series III Mixer Not Discovered

### Network Discovery Requirements
Series III mixers (32SX, 24R, 16R, etc.) use **network-based UCNet**, not USB. To discover your mixer:

#### 1. **Network Connection**
- Mixer must be on the same local network as your computer
- Connect via Ethernet (recommended) or WiFi
- Check that both devices can ping each other

#### 2. **Mixer Network Settings**
On your Series III mixer:
1. Go to **Setup > Network**
2. Verify network settings:
   - **DHCP enabled** (recommended) OR
   - **Static IP** in the same subnet as your computer
3. Note the mixer's IP address

#### 3. **Firewall Settings**
FaderBridge uses UDP port **47809** for device discovery:
- **macOS:** System Settings > Network > Firewall
  - Allow incoming connections for FaderBridge
  - Or temporarily disable firewall for testing
- **Router:** Ensure UDP broadcast is not blocked

#### 4. **Test Network Connectivity**
```bash
# From Terminal, ping your mixer (replace with your mixer's IP)
ping 192.168.1.100

# Test if UCNet port is reachable (requires netcat)
nc -u -v 192.168.1.100 47809
```

#### 5. **Manual Connection (Workaround)**
If discovery fails but you know the mixer's IP:
1. Note the mixer's IP address from its network settings
2. In FaderBridge, use "Connect by IP" option (if available)
3. Enter the IP address manually

---

## MIDI Device Issues

### FaderPort Not Showing in MIDI List
**Check:**
1. FaderPort is connected via USB
2. FaderPort is powered on
3. macOS recognizes the device:
   - Open **Audio MIDI Setup** (Applications > Utilities)
   - Check if FaderPort appears in MIDI devices
4. Restart FaderBridge if device was connected after app launch

### MIDI Messages Not Working
**Check:**
1. FaderPort is in the correct mode:
   - Some FaderPort models have multiple modes (Native, HUI, MCU)
   - Use **Native mode** for best compatibility
2. MIDI Learn is enabled in FaderBridge
3. Check MIDI activity indicator in FaderBridge UI

---

## Connection Issues

### "Already Connected" Error
**Solution:** Disconnect the device first, then reconnect.

### Keep-Alive Timeout
**Symptoms:** Device disconnects after a few seconds

**Causes:**
- Network instability
- Mixer is sleeping/standby
- Firewall blocking keep-alive packets

**Solutions:**
1. Check network stability
2. Disable mixer sleep mode
3. Ensure firewall allows bidirectional UDP on port 47809

---

## Performance Issues

### High Latency (>10ms)
**Check:**
1. Network connection quality (use Ethernet, not WiFi)
2. Other network traffic (close bandwidth-heavy apps)
3. CPU usage (close unnecessary applications)
4. USB connection quality (try different USB port/cable)

### Missed MIDI Messages
**Check:**
1. MIDI buffer size in system settings
2. USB connection quality
3. CPU usage during peak activity

---

## Debug Mode

### Enable Verbose Logging
To get detailed logs for troubleshooting:

1. **macOS Terminal:**
```bash
# Set environment variable before launching
RUST_LOG=debug npm run tauri:dev
```

2. **Check Logs:**
- Look for discovery messages: `"Starting network device discovery"`
- Check for responses: `"Received X bytes from Y"`
- Verify device parsing: `"Discovered device: ..."`

### Common Log Messages

**Good:**
```
INFO  Starting network device discovery on port 47809
DEBUG Sending discovery broadcast to 255.255.255.255:47809
DEBUG Received 128 bytes from 192.168.1.100
INFO  Discovered device: StudioLive 32SX at 192.168.1.100
```

**Problems:**
```
DEBUG Discovery timeout reached  // No devices responded
WARN  Failed to parse discovery response  // Invalid response format
ERROR Socket error during discovery  // Network/firewall issue
```

---

## Known Limitations (MVP Phase)

1. **UCNet Protocol:** Currently using placeholder implementation
   - Discovery packet format is simplified
   - Response parsing is basic
   - May not work with all Series III firmware versions

2. **USB UCNet:** Only Quantum interfaces supported
   - Series III mixers require network connection
   - No USB UCNet support for mixers

3. **Auto-Discovery:** May not find all devices
   - Use manual IP connection as workaround
   - Check mixer firmware is up to date

---

## Getting Help

### Before Reporting Issues
1. Check this troubleshooting guide
2. Enable debug logging
3. Test network connectivity
4. Verify device compatibility

### What to Include in Bug Reports
1. Device models (MIDI controller + UCNet device)
2. Network setup (Ethernet/WiFi, router model)
3. macOS version
4. FaderBridge version
5. Debug logs showing the issue
6. Steps to reproduce

### Contact
- GitHub Issues: [Project Repository]
- Documentation: See `Docs/` folder
- Project Charter: `Docs/PROJECT_CHARTER.md`
