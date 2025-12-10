#!/bin/bash
# capture_and_probe.sh - Capture USB traffic while probing the Quantum
#
# This captures ALL USB traffic on the bus, which we can analyze later
# to understand the protocol.

set -e

echo "=== USB Capture + Probe ==="
echo ""

# Find the USB bus
BUS=$(lsusb | grep -i "1ed8:020e" | awk '{print $2}' | sed 's/^0*//')
if [ -z "$BUS" ]; then
    echo "ERROR: Quantum HD 2 not found"
    exit 1
fi

echo "Found Quantum on bus $BUS"
INTERFACE="usbmon${BUS}"

# Check if usbmon is loaded
if ! lsmod | grep -q usbmon; then
    echo "Loading usbmon module..."
    sudo modprobe usbmon
fi

# Create output directory
OUTDIR="captures_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$OUTDIR"

echo ""
echo "Starting capture on $INTERFACE..."
echo "Output: $OUTDIR/"
echo ""

# Start tshark capture in background
CAPFILE="$OUTDIR/quantum_probe.pcapng"
sudo tshark -i "$INTERFACE" -w "$CAPFILE" &
TSHARK_PID=$!

sleep 2

echo "Capture running (PID: $TSHARK_PID)"
echo ""
echo "Now running probe script..."
echo ""

# Run the probe
sudo python3 test_quantum_usb_v2.py 2>&1 | tee "$OUTDIR/probe_output.txt"

echo ""
echo "Stopping capture..."
sudo kill $TSHARK_PID 2>/dev/null || true
sleep 1

echo ""
echo "=== Capture Complete ==="
echo ""
echo "Files:"
echo "  $CAPFILE"
echo "  $OUTDIR/probe_output.txt"
echo ""
echo "To analyze:"
echo "  wireshark $CAPFILE"
echo ""
echo "Or extract USB data:"
echo "  tshark -r $CAPFILE -Y 'usb.capdata' -T fields -e usb.capdata"
