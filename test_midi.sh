#!/bin/bash
# Quick test to see MIDI devices on macOS

echo "=== MIDI Devices via ioreg ==="
ioreg -c IOUSBDevice -r -l | grep -i "midi\|presonus\|behringer" -A 5

echo ""
echo "=== MIDI Devices via system_profiler ==="
system_profiler SPUSBDataType | grep -i "midi\|presonus\|behringer" -A 10
