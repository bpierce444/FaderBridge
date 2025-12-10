#!/bin/bash
# trace_ucdaemon_log.sh - Trace ucdaemon using macOS log system
#
# This works WITH SIP enabled but provides less detail than dtrace

echo "=== Tracing ucdaemon via log system ==="
echo "Move knobs on the Quantum HD 2..."
echo "Press Ctrl+C to stop"
echo ""

# Stream logs from ucdaemon process
log stream --process ucdaemon --level debug 2>&1 | while read line; do
    echo "[$(date +%H:%M:%S)] $line"
done
