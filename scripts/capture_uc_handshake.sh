#!/bin/bash
# Capture UC's handshake with the mixer
# Usage: 
#   1. Close Universal Control
#   2. Run this script: sudo ./scripts/capture_uc_handshake.sh
#   3. Open Universal Control
#   4. Move the main fader
#   5. Press Ctrl+C to stop capture
#   6. Check capture_uc.txt for the output

MIXER_IP="192.168.1.209"
IFACE="en1"

echo "Capturing UCNet traffic to/from $MIXER_IP on $IFACE..."
echo "1. Make sure Universal Control is CLOSED"
echo "2. Press Enter to start capture"
read

tcpdump -i $IFACE -X host $MIXER_IP and port 53000 -c 150 2>&1 | tee /tmp/capture_uc.txt

echo ""
echo "Capture saved to /tmp/capture_uc.txt"
echo "Looking for Hello (UM), Subscribe (JM), and PV packets..."
echo ""

# Extract key packets
grep -A5 "UM\|JM\|PV" /tmp/capture_uc.txt | head -50
