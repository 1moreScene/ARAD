"""Launch only our built client; verify native window properties and F10 handler.

The game closes itself after its screenshot. No global keyboard injection or
changes to another application's input method are used.
"""
import ctypes as c
import json
import subprocess
import time
from pathlib import Path

root=Path(__file__).resolve().parents[1]
cwd=root/"custom-dnf"
output=cwd/"out/window-policy"
output.mkdir(parents=True,exist_ok=True)
exe=cwd/"target/movement-verified/release/dnf-app.exe"
user=c.WinDLL("user32",use_last_error=True)
user.SetProcessDPIAware()
user.GetDpiForSystem.restype=c.c_uint32
scale=user.GetDpiForSystem()/96.0
expected_size=[round(640*scale),round(480*scale)]
callback_type=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_ssize_t)
user.EnumWindows.argtypes=[callback_type,c.c_ssize_t]
user.GetWindowThreadProcessId.argtypes=[c.c_void_p,c.POINTER(c.c_uint32)]
user.GetWindowLongPtrW.argtypes=[c.c_void_p,c.c_int]
user.GetWindowLongPtrW.restype=c.c_ssize_t
user.GetForegroundWindow.restype=c.c_void_p
user.SendMessageW.argtypes=[c.c_void_p,c.c_uint32,c.c_size_t,c.c_ssize_t]
user.SendMessageW.restype=c.c_ssize_t

class Rect(c.Structure):
    _fields_=[("left",c.c_long),("top",c.c_long),("right",c.c_long),("bottom",c.c_long)]
user.GetClientRect.argtypes=[c.c_void_p,c.POINTER(Rect)]
user.GetWindowTextW.argtypes=[c.c_void_p,c.c_void_p,c.c_int]

process=subprocess.Popen([str(exe),"--shot-after","180","--shot-out",str(output/"window.png")],
                        cwd=cwd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
window=[]
@callback_type
def find(hwnd,_):
    pid=c.c_uint32()
    user.GetWindowThreadProcessId(hwnd,c.byref(pid))
    if pid.value==process.pid:
        window.append(hwnd)
    return 1
deadline=time.monotonic()+5
while not window and time.monotonic()<deadline:
    user.EnumWindows(find,0)
    if not window: time.sleep(0.02)
if not window:
    raise RuntimeError("our client did not create a window")
hwnd=window[0]
time.sleep(0.15)
deadline=time.monotonic()+1
while user.GetForegroundWindow()!=hwnd and time.monotonic()<deadline:
    time.sleep(0.02)
rect=Rect()
user.GetClientRect(hwnd,c.byref(rect))
report={"client":[rect.right-rect.left,rect.bottom-rect.top],
        "expected_display":expected_size,"logical_canvas":[640,480],
        "foreground":user.GetForegroundWindow()==hwnd,
        "resize_disabled":not bool(user.GetWindowLongPtrW(hwnd,-16)&0x00040000),
        "maximize_disabled":not bool(user.GetWindowLongPtrW(hwnd,-16)&0x00010000)}
def topmost():
    return bool(user.GetWindowLongPtrW(hwnd,-20)&0x8)
report["pinned_initially"]=topmost()
user.SendMessageW(hwnd,0x104,0x79,1)  # F10 system-keydown, to our window only
report["f10_unpins"]=not topmost()
user.SendMessageW(hwnd,0x104,0x79,(1<<30)|1)
report["repeat_does_not_toggle"]=not topmost()
user.SendMessageW(hwnd,0x105,0x79,0)
user.SendMessageW(hwnd,0x104,0x79,1)
user.SendMessageW(hwnd,0x105,0x79,0)
report["f10_repins"]=topmost()
log=process.communicate(timeout=10)[0].decode("utf-8",errors="replace")
(output/"window.log").write_text(log,encoding="utf-8")
report["exit_code"]=process.returncode
report["ime_context_detached"]="IME上下文=false" in log
report["thread_ime_disabled"]="IME线程禁用=true" in log
report["screenshot_saved"]=(output/"window.png").exists() and "窗口实拍已保存" in log
# Closing the caption button sends WM_CLOSE -> WM_DESTROY -> WM_QUIT. Verify
# that the loop ends instead of reclaiming focus for a destroyed window.
process=subprocess.Popen([str(exe)],cwd=cwd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
window=[]
deadline=time.monotonic()+5
while not window and time.monotonic()<deadline:
    user.EnumWindows(find,0)
    if not window: time.sleep(0.02)
if not window: raise RuntimeError("close-test window did not appear")
time.sleep(0.15)
user.SendMessageW(window[0],0x10,0,0)
close_log=process.communicate(timeout=3)[0].decode("utf-8",errors="replace")
(output/"close.log").write_text(close_log,encoding="utf-8")
report["close_stops_process"]=process.returncode==0
process=subprocess.Popen([str(exe),"--scale","1","--shot-after","15","--shot-out",str(output/"scale-one.png")],cwd=cwd,
    stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
window=[]
deadline=time.monotonic()+5
while not window and time.monotonic()<deadline:
    user.EnumWindows(find,0)
    if not window: time.sleep(0.02)
if not window: raise RuntimeError("scale-one window did not appear")
time.sleep(0.03)
rect=Rect(); user.GetClientRect(window[0],c.byref(rect))
report["scale_one_client"]=[rect.right-rect.left,rect.bottom-rect.top]
one_log=process.communicate(timeout=5)[0].decode("utf-8",errors="replace")
(output/"scale-one.log").write_text(one_log,encoding="utf-8")
assert report["scale_one_client"]==[640,480]
(output/"verification.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))
assert report["client"]==expected_size
assert report["foreground"],"foreground acquisition did not succeed on this desktop"
assert all(report[name] for name in ["resize_disabled","maximize_disabled","pinned_initially","f10_unpins",
    "repeat_does_not_toggle","f10_repins","ime_context_detached","thread_ime_disabled","screenshot_saved","close_stops_process"])
assert process.returncode==0
