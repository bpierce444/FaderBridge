#!/bin/bash
# setup_vm.sh - Run this inside the Linux VM to set up capture environment

set -e

echo "=== Setting up USB capture environment ==="

# Update system
echo "[1/5] Updating system..."
sudo apt update && sudo apt upgrade -y

# Install required packages
echo "[2/5] Installing packages..."
sudo apt install -y \
    wireshark \
    tshark \
    usbutils \
    python3-pip \
    python3-usb \
    libusb-1.0-0-dev

# Add user to wireshark group
echo "[3/5] Configuring permissions..."
sudo usermod -aG wireshark $USER

# Load usbmon module
echo "[4/5] Loading usbmon kernel module..."
sudo modprobe usbmon
echo "usbmon" | sudo tee -a /etc/modules

# Set permissions on usbmon devices
sudo chmod 666 /dev/usbmon* 2>/dev/null || true

# Install Python requirements
echo "[5/5] Installing Python packages..."
pip3 install pyusb

echo ""
echo "=== Setup complete ==="
echo ""
echo "IMPORTANT: Log out and back in for group changes to take effect!"
echo ""
echo "Next steps:"
echo "  1. Log out and back in"
echo "  2. Pass Quantum HD 2 USB device to this VM"
echo "  3. Run: lsusb | grep -i fender"
echo "  4. Run: sudo python3 test_quantum_usb.py"
