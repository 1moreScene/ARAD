"""Read-only sampling of the running 40JP client; never writes or injects code.

python tools/trace_official_movement.py --seconds 30 --out work/official_movement_trace.jsonl
Play the movement scenarios in ARAD while sampling. Addresses apply only to the
verified 3,948,544-byte 40JP image at ImageBase 0x400000.
"""
import argparse
import ctypes as c
import json
import struct
import subprocess
import time
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--pid", type=int)
    parser.add_argument("--seconds", type=float, default=1)
    parser.add_argument("--wait-client", type=float, default=0, help="seconds to wait for an already user-launched client")
    parser.add_argument("--out", type=Path, default=Path("work/official_movement_trace.jsonl"))
    args = parser.parse_args()
    if args.pid is None:
        deadline=time.monotonic()+args.wait_client
        while True:
            pid_text=subprocess.check_output(
                ["powershell", "-NoProfile", "-Command", "(Get-Process ARAD -ErrorAction SilentlyContinue | Select-Object -First 1).Id"],
                text=True).strip()
            if pid_text:
                args.pid=int(pid_text)
                break
            if time.monotonic()>=deadline:
                raise RuntimeError("ARAD is not running; open the official client before sampling")
            time.sleep(0.5)
    kernel = c.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [c.c_uint32, c.c_int, c.c_uint32]
    kernel.OpenProcess.restype = c.c_void_p
    kernel.ReadProcessMemory.argtypes = [c.c_void_p, c.c_void_p, c.c_void_p, c.c_size_t, c.POINTER(c.c_size_t)]
    kernel.CloseHandle.argtypes = [c.c_void_p]
    handle = kernel.OpenProcess(0x1010, 0, args.pid)  # QUERY_LIMITED_INFORMATION | VM_READ only
    if not handle:
        raise c.WinError(c.get_last_error())

    def read(addr, size):
        buf = c.create_string_buffer(size)
        count = c.c_size_t()
        if not kernel.ReadProcessMemory(handle, addr, buf, size, c.byref(count)) or count.value != size:
            raise c.WinError(c.get_last_error())
        return buf.raw

    def u32(addr):
        return struct.unpack("<I", read(addr, 4))[0]

    last = None
    start = time.monotonic()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    try:
        if read(0x400000, 2) != b"MZ" or read(0x4BF540, 3) != b"\x55\x8b\xec":
            raise RuntimeError("unexpected client image; refusing to use fixed addresses")
        with args.out.open("w", encoding="utf-8") as stream:
            while time.monotonic() - start < args.seconds:
                snapshot = {}
                actor = u32(0x8F7C04)
                snapshot["actor"] = hex(actor)
                if actor:
                    snapshot["vtable"] = hex(u32(actor))
                    snapshot["state"] = u32(actor+1639*4)
                    snapshot["jump_phase"] = u32(actor+1723*4)
                    snapshot["directions"] = list(struct.unpack("<II", read(actor+1659*4,8)))
                    snapshot["speeds"] = list(struct.unpack("<IIII", read(actor+1652*4,16)))
                    snapshot["rates"] = list(struct.unpack("<II", read(actor+1663*4,8)))
                    controller = u32(actor+112)
                    snapshot["facing"] = read(controller+52,1)[0]
                    snapshot["pending_state"] = struct.unpack("<i",read(actor+616,4))[0]
                    snapshot["position"] = list(struct.unpack("<fff", read(controller+4,12)))
                    snapshot["velocity"] = list(struct.unpack("<fff", read(controller+16,12)))
                    animation = u32(actor+120)
                    if animation:
                        snapshot["frame"] = u32(animation+148)
                begin, end = struct.unpack("<II", read(0x8F8558+48,8))
                if end < begin or end-begin > 16*10000 or (end-begin)%16:
                    raise RuntimeError("invalid input history bounds")
                records = []
                for i in range(max(begin, end-16*8), end, 16):
                    key, stamp, duration, held, edge = struct.unpack("<III BB", read(i,14))
                    records.append([key, stamp, duration, held, edge])
                snapshot["records"] = records
                snapshot["dash_commands"] = [u32(0x9A0388+36), u32(0x9A03B4+36)]
                snapshot["dash_enabled"] = [read(0x9A0388+40,1)[0],read(0x9A03B4+40,1)[0]]
                if snapshot != last:
                    last = snapshot
                    stream.write(json.dumps({"t_ms": round((time.monotonic()-start)*1000), **snapshot})+"\n")
                    stream.flush()
                time.sleep(0.01)
        print(f"saved read-only trace: {args.out}")
        print(json.dumps(last))
    finally:
        kernel.CloseHandle(handle)


if __name__ == "__main__":
    main()
