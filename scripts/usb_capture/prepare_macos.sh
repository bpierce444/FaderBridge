#!/bin/bash
# prepare_macos.sh - Prepare macOS for USB passthrough to VM
#
# Run this on the macOS host BEFORE passing the Quantum to the VM

set -e

echo "=== Preparing macOS for USB Capture ==="
echo ""

# Check if ucdaemon is running
UCDAEMON_PID=$(pgrep ucdaemon 2>/dev/null || echo "")

if [ -n "$UCDAEMON_PID" ]; then
    echo "ucdaemon is running (PID: $UCDAEMON_PID)"
    echo ""
    echo "To pass the Quantum HD 2 to a VM, we need to stop ucdaemon."
    echo "This will disconnect Universal Control from the device."
    echo ""
    read -p "Stop ucdaemon? [y/N] " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Stopping ucdaemon..."
        sudo launchctl unload /Library/LaunchDaemons/com.presonus.ucdaemon.plist
        sleep 1
        
        # Verify it stopped
        if pgrep ucdaemon > /dev/null 2>&1; then
            echo "WARNING: ucdaemon still running. Try killing it:"
            echo "  sudo kill $(pgrep ucdaemon)"
        else
            echo "ucdaemon stopped successfully"
        fi
    else
        echo "Aborted. ucdaemon still running."
        exit 1
    fi
else
    echo "ucdaemon is not running - good!"
fi

echo ""
echo "=== Ready for VM USB Passthrough ==="
echo ""
echo "Next steps:"
echo "  1. Open UTM or VMware Fusion"
echo "  2. Start your Linux VM"
echo "  3. Pass 'Quantum HD 2' USB device to the VM"
echo "  4. In the VM, run: lsusb | grep -i fender"
echo ""
echo "To restart ucdaemon later:"
echo "  sudo launchctl load /Library/LaunchDaemons/com.presonus.ucdaemon.plist"
