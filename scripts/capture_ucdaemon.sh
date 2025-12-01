#!/bin/bash
# capture_ucdaemon.sh - Capture ucdaemon USB traffic
#
# This script uses multiple approaches to capture the USB protocol
# used by PreSonus Universal Control to communicate with interfaces.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/../captures"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$OUTPUT_DIR"

echo "=== PreSonus USB Protocol Capture ==="
echo ""
echo "Target: ucdaemon (pid $(pgrep ucdaemon || echo 'not running'))"
echo "Output: $OUTPUT_DIR"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges for dtrace."
    echo "Run with: sudo $0"
    exit 1
fi

# Method 1: Basic syscall tracing
echo "Starting syscall trace..."
echo "Move faders in Universal Control, then press Ctrl+C to stop."
echo ""

dtrace -n '
syscall::write:entry /execname == "ucdaemon"/ { 
    self->buf = arg1; 
    self->len = arg2; 
}
syscall::write:return /execname == "ucdaemon" && self->len > 0 && self->len <= 512/ { 
    printf("W[%d]: ", self->len);
    tracemem(copyin(self->buf, self->len < 128 ? self->len : 128), 128);
}
syscall::read:return /execname == "ucdaemon" && arg1 > 0 && arg1 <= 512/ { 
    printf("R[%d]: ", arg1);
}
' 2>&1 | tee "$OUTPUT_DIR/ucdaemon_trace_$TIMESTAMP.log"

echo ""
echo "Capture saved to: $OUTPUT_DIR/ucdaemon_trace_$TIMESTAMP.log"
