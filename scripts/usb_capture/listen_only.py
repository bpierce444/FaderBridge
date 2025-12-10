#!/usr/bin/env python3
"""
listen_only.py - Just listen for any data from the Quantum HD 2

Sometimes devices send data on their own (status updates, etc.)
This script just listens on all possible endpoints.
"""

import usb.core
import usb.util
import sys
import time
import threading

VENDOR_ID = 0x1ed8
PRODUCT_ID = 0x020e


def listen_endpoint(dev, intf_num, ep, duration=30):
    """Listen on a specific endpoint."""
    name = f"Interface {intf_num}, EP 0x{ep.bEndpointAddress:02x}"
    print(f"[{name}] Listening...")
    
    start = time.time()
    count = 0
    
    while time.time() - start < duration:
        try:
            data = dev.read(ep.bEndpointAddress, ep.wMaxPacketSize, timeout=100)
            count += 1
            elapsed = time.time() - start
            print(f"[{name}] {elapsed:.1f}s: {bytes(data).hex()}")
        except usb.core.USBError:
            pass  # Timeout
    
    print(f"[{name}] Done - received {count} packets")


def main():
    print("=" * 60)
    print("Quantum HD 2 - Listen Only Mode")
    print("=" * 60)
    
    dev = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if dev is None:
        print("Device not found")
        sys.exit(1)
    
    print(f"Found: {dev.product}")
    
    # Detach all kernel drivers
    print("\nDetaching kernel drivers...")
    for cfg in dev:
        for intf in cfg:
            try:
                if dev.is_kernel_driver_active(intf.bInterfaceNumber):
                    dev.detach_kernel_driver(intf.bInterfaceNumber)
            except:
                pass
    
    # Find all IN endpoints
    print("\nFinding IN endpoints...")
    in_endpoints = []
    
    cfg = dev.get_active_configuration()
    for intf in cfg:
        intf_num = intf.bInterfaceNumber
        alt = intf.bAlternateSetting
        
        # Claim interface
        try:
            usb.util.claim_interface(dev, intf_num)
            if alt > 0:
                dev.set_interface_altsetting(interface=intf_num, alternate_setting=alt)
        except:
            pass
        
        for ep in intf:
            if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                ep_type = usb.util.endpoint_type(ep.bmAttributes)
                type_name = {0: "Ctrl", 1: "Iso", 2: "Bulk", 3: "Intr"}[ep_type]
                print(f"  Interface {intf_num}.{alt} EP 0x{ep.bEndpointAddress:02x} ({type_name}, max {ep.wMaxPacketSize})")
                in_endpoints.append((intf_num, alt, ep))
    
    print(f"\nFound {len(in_endpoints)} IN endpoints")
    print("\n" + "=" * 60)
    print("Listening for 30 seconds...")
    print("Try moving faders or changing settings on the Quantum")
    print("=" * 60 + "\n")
    
    # Listen on key endpoints (not isochronous - those are audio)
    endpoints_to_monitor = []
    for intf_num, alt, ep in in_endpoints:
        ep_type = usb.util.endpoint_type(ep.bmAttributes)
        if ep_type != 1:  # Skip isochronous (audio streams)
            endpoints_to_monitor.append((intf_num, ep))
    
    # Start listener threads
    threads = []
    for intf_num, ep in endpoints_to_monitor:
        t = threading.Thread(target=listen_endpoint, args=(dev, intf_num, ep, 30))
        t.daemon = True
        t.start()
        threads.append(t)
    
    # Wait for all threads
    for t in threads:
        t.join()
    
    print("\n" + "=" * 60)
    print("Listening complete")
    print("=" * 60)


if __name__ == "__main__":
    main()
