#!/usr/sbin/dtrace -s
/*
 * sniff_ucdaemon.d - Trace USB I/O from ucdaemon to Quantum HD 2
 * 
 * Usage: sudo dtrace -s sniff_ucdaemon.d
 * 
 * This script traces IOKit USB calls made by ucdaemon to capture
 * the vendor-specific control protocol used by PreSonus interfaces.
 */

#pragma D option quiet
#pragma D option switchrate=10hz

dtrace:::BEGIN
{
    printf("=== Tracing ucdaemon USB I/O ===\n");
    printf("Move faders in Universal Control to capture traffic...\n");
    printf("Press Ctrl+C to stop\n\n");
}

/* Trace all write syscalls from ucdaemon */
syscall::write:entry
/execname == "ucdaemon"/
{
    self->fd = arg0;
    self->buf = arg1;
    self->len = arg2;
}

syscall::write:return
/execname == "ucdaemon" && self->len > 0 && self->len < 1024/
{
    printf("[%Y] WRITE fd=%d len=%d\n", walltimestamp, self->fd, self->len);
    tracemem(copyin(self->buf, self->len < 64 ? self->len : 64), 64);
    printf("\n");
}

/* Trace all read syscalls from ucdaemon */
syscall::read:entry
/execname == "ucdaemon"/
{
    self->rfd = arg0;
    self->rbuf = arg1;
    self->rlen = arg2;
}

syscall::read:return
/execname == "ucdaemon" && arg1 > 0 && arg1 < 1024/
{
    printf("[%Y] READ fd=%d len=%d\n", walltimestamp, self->rfd, arg1);
    tracemem(copyin(self->rbuf, arg1 < 64 ? arg1 : 64), 64);
    printf("\n");
}

/* Trace ioctl calls (often used for USB control transfers) */
syscall::ioctl:entry
/execname == "ucdaemon"/
{
    printf("[%Y] IOCTL fd=%d request=0x%x\n", walltimestamp, arg0, arg1);
}

dtrace:::END
{
    printf("\n=== Tracing complete ===\n");
}
