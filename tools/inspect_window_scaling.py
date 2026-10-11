"""Read-only physical/logical window and DPI inventory for ARAD and custom-dnf."""
import ctypes as c
import json
import subprocess
from pathlib import Path

user=c.WinDLL('user32',use_last_error=True)
user.SetProcessDPIAware()
user.GetDpiForSystem.restype=c.c_uint32
user.GetWindowDpiAwarenessContext.argtypes=[c.c_void_p]
user.GetWindowDpiAwarenessContext.restype=c.c_void_p
user.GetAwarenessFromDpiAwarenessContext.argtypes=[c.c_void_p]
user.GetAwarenessFromDpiAwarenessContext.restype=c.c_int
user.SetThreadDpiAwarenessContext.argtypes=[c.c_void_p]
user.SetThreadDpiAwarenessContext.restype=c.c_void_p
user.GetDpiForWindow.argtypes=[c.c_void_p]
user.GetDpiForWindow.restype=c.c_uint32
user.GetWindowThreadProcessId.argtypes=[c.c_void_p,c.POINTER(c.c_uint32)]
user.GetWindowTextW.argtypes=[c.c_void_p,c.c_void_p,c.c_int]
user.GetClassNameW.argtypes=[c.c_void_p,c.c_void_p,c.c_int]
class Rect(c.Structure):
    _fields_=[('left',c.c_long),('top',c.c_long),('right',c.c_long),('bottom',c.c_long)]
user.GetClientRect.argtypes=[c.c_void_p,c.POINTER(Rect)]
user.GetWindowRect.argtypes=[c.c_void_p,c.POINTER(Rect)]
user.ClientToScreen.argtypes=[c.c_void_p,c.c_void_p]
class Point(c.Structure):
    _fields_=[('x',c.c_long),('y',c.c_long)]
raw=subprocess.check_output(['powershell','-NoProfile','-Command',
    '@(Get-Process ARAD,dnf-app -ErrorAction SilentlyContinue | Select-Object Id,Path) | ConvertTo-Json -Compress'],text=True).strip()
processes=json.loads(raw) if raw else []
if isinstance(processes,dict): processes=[processes]
paths={p['Id']:p['Path'] for p in processes}
rows=[]
callback=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_ssize_t)
user.EnumWindows.argtypes=[callback,c.c_ssize_t]
@callback
def sample(hwnd,_):
    pid=c.c_uint32(); user.GetWindowThreadProcessId(hwnd,c.byref(pid))
    if pid.value not in paths: return 1
    title=c.create_unicode_buffer(256); name=c.create_unicode_buffer(256)
    user.GetWindowTextW(hwnd,title,256); user.GetClassNameW(hwnd,name,256)
    # Read screen coordinates once as per-monitor aware (physical), once as
    # DPI unaware (virtualized 96dpi); restore this tool's thread context each time.
    sizes={}
    for label,context in [('physical',-4),('virtual_96dpi',-1)]:
        previous=user.SetThreadDpiAwarenessContext(c.c_void_p(context))
        rect=Rect(); outer=Rect()
        user.GetClientRect(hwnd,c.byref(rect)); user.GetWindowRect(hwnd,c.byref(outer))
        first=Point(rect.left,rect.top); last=Point(rect.right,rect.bottom)
        user.ClientToScreen(hwnd,c.byref(first)); user.ClientToScreen(hwnd,c.byref(last))
        sizes[label]={'client_rect':[rect.right-rect.left,rect.bottom-rect.top],
            'client_screen_pixels':[last.x-first.x,last.y-first.y],
            'window_rect':[outer.right-outer.left,outer.bottom-outer.top]}
        user.SetThreadDpiAwarenessContext(previous)
    rows.append({'pid':pid.value,'path':paths[pid.value],'title':title.value,'class':name.value,
        'dpi':user.GetDpiForWindow(hwnd),'awareness':user.GetAwarenessFromDpiAwarenessContext(user.GetWindowDpiAwarenessContext(hwnd)),**sizes})
    return 1
user.EnumWindows(sample,0)
report={'system_dpi':user.GetDpiForSystem(),'windows':rows}
Path('work/window_scaling_observation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
