"""Gracefully close only custom-dnf executables in this workspace before update."""
import ctypes as c
import json
import subprocess
from pathlib import Path

root=Path(__file__).resolve().parents[1]
allowed={(root/"custom-dnf"/name).resolve().as_posix().lower() for name in
         ["target/release/dnf-app.exe","target/debug/dnf-app.exe","target/movement-verified/release/dnf-app.exe"]}
raw=subprocess.check_output(["powershell","-NoProfile","-Command",
    "@(Get-Process dnf-app -ErrorAction SilentlyContinue | Select-Object Id,Path) | ConvertTo-Json -Compress"],text=True).strip()
processes=json.loads(raw) if raw else []
if isinstance(processes,dict): processes=[processes]
user=c.WinDLL("user32",use_last_error=True)
kernel=c.WinDLL("kernel32",use_last_error=True)
callback_type=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_ssize_t)
user.EnumWindows.argtypes=[callback_type,c.c_ssize_t]
user.GetWindowThreadProcessId.argtypes=[c.c_void_p,c.POINTER(c.c_uint32)]
user.SendMessageTimeoutW.argtypes=[c.c_void_p,c.c_uint32,c.c_size_t,c.c_ssize_t,c.c_uint32,c.c_uint32,c.POINTER(c.c_size_t)]
user.SendMessageTimeoutW.restype=c.c_ssize_t
kernel.OpenProcess.argtypes=[c.c_uint32,c.c_int,c.c_uint32]
kernel.OpenProcess.restype=c.c_void_p
kernel.WaitForSingleObject.argtypes=[c.c_void_p,c.c_uint32]
kernel.CloseHandle.argtypes=[c.c_void_p]
targets={p['Id'] for p in processes if p.get('Path') and Path(p['Path']).resolve().as_posix().lower() in allowed}
@callback_type
def close(hwnd,_):
    pid=c.c_uint32()
    user.GetWindowThreadProcessId(hwnd,c.byref(pid))
    if pid.value in targets:
        result=c.c_size_t()
        user.SendMessageTimeoutW(hwnd,0x10,0,0,2,2000,c.byref(result))
    return 1
handles=[(pid,kernel.OpenProcess(0x100000,0,pid)) for pid in targets]
user.EnumWindows(close,0)
for pid,handle in handles:
    if handle:
        result=kernel.WaitForSingleObject(handle,3000)
        kernel.CloseHandle(handle)
        if result!=0: raise RuntimeError(f"client {pid} did not close; no force termination attempted")
print(f"gracefully closed workspace clients: {sorted(targets)}")
