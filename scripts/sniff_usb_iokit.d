#!/usr/sbin/dtrace -s
/*
 * sniff_usb_iokit.d - Trace IOKit USB calls from ucdaemon
 * 
 * Usage: sudo dtrace -s sniff_usb_iokit.d
 * 
 * This traces the IOKit framework calls that ucdaemon uses to
 * communicate with the Quantum HD 2 Control interface.
 */

#pragma D option quiet
#pragma D option switchrate=10hz

dtrace:::BEGIN
{
    printf("=== Tracing ucdaemon IOKit USB calls ===\n");
    printf("Vendor ID: 0x1ed8 (PreSonus/Fender)\n");
    printf("Product ID: 0x020e (Quantum HD 2)\n");
    printf("Interface: #5 (Quantum HD 2 Control)\n\n");
    printf("Move faders in Universal Control to capture traffic...\n");
    printf("Press Ctrl+C to stop\n\n");
}

/* Trace IOKit user client calls */
pid$target::IOConnectCallMethod:entry
{
    printf("[%Y] IOConnectCallMethod selector=%d\n", walltimestamp, arg1);
    printf("  inputCnt=%d outputCnt=%d\n", arg4, arg7);
}

pid$target::IOConnectCallStructMethod:entry
{
    printf("[%Y] IOConnectCallStructMethod selector=%d inputSize=%d\n", 
           walltimestamp, arg1, arg3);
}

pid$target::IOConnectCallScalarMethod:entry
{
    printf("[%Y] IOConnectCallScalarMethod selector=%d\n", walltimestamp, arg1);
}

/* Trace USB pipe operations */
pid$target::*WritePipe*:entry
{
    printf("[%Y] WritePipe called\n", walltimestamp);
}

pid$target::*ReadPipe*:entry
{
    printf("[%Y] ReadPipe called\n", walltimestamp);
}

/* Trace USB control requests */
pid$target::*DeviceRequest*:entry
{
    printf("[%Y] DeviceRequest called\n", walltimestamp);
}

pid$target::*ControlRequest*:entry
{
    printf("[%Y] ControlRequest called\n", walltimestamp);
}

dtrace:::END
{
    printf("\n=== Tracing complete ===\n");
}
