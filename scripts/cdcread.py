#!/usr/bin/env python3
"""Read a USB CDC-ACM device directly via libusb.

This workstation's kernel (CachyOS 7.1.4) has no cdc_acm driver, so no
/dev/ttyACM* ever appears. Since no driver is bound, the interface is free for
userspace to claim. We assert DTR on the control interface -- TinyGo's CDC
implementation will not transmit until the host does -- then drain the bulk IN
endpoint.
"""
import ctypes
import sys
import time

VID, PID = 0x2E8A, 0x000A
DATA_IFACE = 1
EP_IN = 0x83
SET_CONTROL_LINE_STATE = 0x22
DTR_RTS = 0x03

lib = ctypes.CDLL("libusb-1.0.so.0")


class Ctx(ctypes.Structure):
    pass


lib.libusb_open_device_with_vid_pid.restype = ctypes.c_void_p
lib.libusb_open_device_with_vid_pid.argtypes = [
    ctypes.c_void_p, ctypes.c_uint16, ctypes.c_uint16]
lib.libusb_claim_interface.argtypes = [ctypes.c_void_p, ctypes.c_int]
lib.libusb_detach_kernel_driver.argtypes = [ctypes.c_void_p, ctypes.c_int]
lib.libusb_control_transfer.argtypes = [
    ctypes.c_void_p, ctypes.c_uint8, ctypes.c_uint8, ctypes.c_uint16,
    ctypes.c_uint16, ctypes.c_void_p, ctypes.c_uint16, ctypes.c_uint]
lib.libusb_bulk_transfer.argtypes = [
    ctypes.c_void_p, ctypes.c_ubyte, ctypes.c_void_p, ctypes.c_int,
    ctypes.POINTER(ctypes.c_int), ctypes.c_uint]
# These two MUST have argtypes as well: without them ctypes passes the 64-bit
# handle as a 32-bit int, truncating the pointer and segfaulting.
lib.libusb_release_interface.argtypes = [ctypes.c_void_p, ctypes.c_int]
lib.libusb_close.argtypes = [ctypes.c_void_p]
lib.libusb_init.argtypes = [ctypes.c_void_p]

ctx = ctypes.c_void_p()
if lib.libusb_init(ctypes.byref(ctx)) != 0:
    sys.exit("libusb_init failed")

h = lib.libusb_open_device_with_vid_pid(ctx, VID, PID)
if not h:
    sys.exit(f"device {VID:04x}:{PID:04x} not found (or no permission)")

# Free the interface if anything holds it (harmless if nothing does).
for i in (0, DATA_IFACE):
    lib.libusb_detach_kernel_driver(h, i)

rc = lib.libusb_claim_interface(h, DATA_IFACE)
if rc != 0:
    sys.exit(f"claim_interface({DATA_IFACE}) failed rc={rc} "
             "(try running as root, or check no other process holds it)")

# Assert DTR|RTS on the CONTROL interface (0). TinyGo gates transmission on this.
rc = lib.libusb_control_transfer(h, 0x21, SET_CONTROL_LINE_STATE,
                                 DTR_RTS, 0, None, 0, 1000)
print(f"[dtr asserted rc={rc}]", file=sys.stderr)

buf = (ctypes.c_ubyte * 512)()
n = ctypes.c_int()
deadline = time.time() + float(sys.argv[1] if len(sys.argv) > 1 else 30)
out = bytearray()
while time.time() < deadline:
    rc = lib.libusb_bulk_transfer(h, EP_IN, buf, 512, ctypes.byref(n), 500)
    if rc == 0 and n.value:
        chunk = bytes(buf[:n.value])
        out += chunk
        sys.stdout.write(chunk.decode("utf-8", "replace"))
        sys.stdout.flush()
        if b"=== END ===" in out:
            break
    elif rc not in (0, -7):  # -7 = LIBUSB_ERROR_TIMEOUT
        print(f"\n[bulk_transfer rc={rc}]", file=sys.stderr)
        break

lib.libusb_release_interface(h, DATA_IFACE)
lib.libusb_close(h)
print(f"\n[read {len(out)} bytes]", file=sys.stderr)
