"""Read only ARAD renderer fields and capture its logical DC/physical client for cross checks."""
import ctypes as c
import json
import struct
import subprocess
import sys
from pathlib import Path
from PIL import Image,ImageGrab,ImageChops,ImageStat
import numpy as np
import movement_evidence as pe

root=Path(__file__).resolve().parents[1];out=root/'work/rendering_evidence';out.mkdir(parents=True,exist_ok=True)
u=c.WinDLL('user32',use_last_error=True);g=c.WinDLL('gdi32',use_last_error=True);k=c.WinDLL('kernel32',use_last_error=True)
u.SetProcessDPIAware()
callback=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_ssize_t)
u.EnumWindows.argtypes=[callback,c.c_ssize_t];u.GetWindowThreadProcessId.argtypes=[c.c_void_p,c.POINTER(c.c_uint32)]
u.GetDC.argtypes=[c.c_void_p];u.GetDC.restype=c.c_void_p;u.ReleaseDC.argtypes=[c.c_void_p,c.c_void_p]
u.SetThreadDpiAwarenessContext.argtypes=[c.c_void_p];u.SetThreadDpiAwarenessContext.restype=c.c_void_p
u.GetWindowDpiAwarenessContext.argtypes=[c.c_void_p];u.GetWindowDpiAwarenessContext.restype=c.c_void_p
u.GetAwarenessFromDpiAwarenessContext.argtypes=[c.c_void_p]
class Rect(c.Structure):_fields_=[('left',c.c_long),('top',c.c_long),('right',c.c_long),('bottom',c.c_long)]
class Point(c.Structure):_fields_=[('x',c.c_long),('y',c.c_long)]
u.GetClientRect.argtypes=[c.c_void_p,c.POINTER(Rect)];u.ClientToScreen.argtypes=[c.c_void_p,c.POINTER(Point)]
raw=subprocess.check_output(['powershell','-NoProfile','-Command','[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); @(Get-Process ARAD -ErrorAction SilentlyContinue | Select-Object Id,Path) | ConvertTo-Json -Compress'],encoding='utf-8').strip()
rows=json.loads(raw) if raw else [];rows=[rows] if isinstance(rows,dict) else rows
allowed=(root/'40jp-client/40jp单机版/ARAD.exe').resolve().as_posix().lower()
pids={p['Id'] for p in rows if Path(p['Path']).resolve().as_posix().lower()==allowed}
windows=[]
@callback
def scan(hwnd,_):
    pid=c.c_uint32();u.GetWindowThreadProcessId(hwnd,c.byref(pid));r=Rect();u.GetClientRect(hwnd,c.byref(r))
    if pid.value in pids and r.right-r.left>=640 and r.bottom-r.top>=480:windows.append((hwnd,pid.value))
    return 1
u.EnumWindows(scan,0)
assert windows,'official main window not running'
hwnd,pid=windows[0]
k.OpenProcess.argtypes=[c.c_uint32,c.c_int,c.c_uint32];k.OpenProcess.restype=c.c_void_p
k.ReadProcessMemory.argtypes=[c.c_void_p,c.c_void_p,c.c_void_p,c.c_size_t,c.POINTER(c.c_size_t)]
k.CloseHandle.argtypes=[c.c_void_p]
h=k.OpenProcess(0x410,0,pid)
def read(address,count):
    b=c.create_string_buffer(count);n=c.c_size_t()
    assert k.ReadProcessMemory(h,c.c_void_p(address),b,count,c.byref(n)) and n.value==count
    return b.raw
def uint(address):return struct.unpack('<I',read(address,4))[0]
renderer=uint(0x79C0A4);sprite=uint(renderer+8256);sprite_table=uint(sprite)
report={'pid':pid,'renderer':hex(renderer),'sprite':hex(sprite),'sprite_begin':hex(uint(sprite_table+16)),
        'requested_depth':uint(renderer+24),'backbuffer':list(struct.unpack('<III',read(renderer+8276,12))),
        'awareness':u.GetAwarenessFromDpiAwarenessContext(u.GetWindowDpiAwarenessContext(hwnd))}
k.CloseHandle(h)
r=Rect();u.GetClientRect(hwnd,c.byref(r));p=Point();u.ClientToScreen(hwnd,c.byref(p))
physical=ImageGrab.grab(bbox=(p.x,p.y,p.x+r.right,p.y+r.bottom)).convert('RGB')
physical.save(out/'official-physical.png');report['physical_size']=physical.size
previous=u.SetThreadDpiAwarenessContext(c.c_void_p(-1))
r=Rect();u.GetClientRect(hwnd,c.byref(r));report['logical_size']=[r.right,r.bottom]
# Capture the source DC without changing either application or resource files.
g.CreateCompatibleDC.argtypes=[c.c_void_p];g.CreateCompatibleDC.restype=c.c_void_p
g.CreateCompatibleBitmap.argtypes=[c.c_void_p,c.c_int,c.c_int];g.CreateCompatibleBitmap.restype=c.c_void_p
g.SelectObject.argtypes=[c.c_void_p,c.c_void_p];g.SelectObject.restype=c.c_void_p
g.BitBlt.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_int,c.c_int,c.c_void_p,c.c_int,c.c_int,c.c_uint32]
g.DeleteObject.argtypes=[c.c_void_p];g.DeleteDC.argtypes=[c.c_void_p]
class Header(c.Structure):
    _fields_=[('size',c.c_uint32),('w',c.c_int32),('h',c.c_int32),('planes',c.c_uint16),('bits',c.c_uint16),
              ('compression',c.c_uint32),('image_size',c.c_uint32),('xppm',c.c_int32),('yppm',c.c_int32),
              ('used',c.c_uint32),('important',c.c_uint32)]
g.GetDIBits.argtypes=[c.c_void_p,c.c_void_p,c.c_uint,c.c_uint,c.c_void_p,c.c_void_p,c.c_uint]
dc=u.GetDC(hwnd);mem=g.CreateCompatibleDC(dc);bmp=g.CreateCompatibleBitmap(dc,r.right,r.bottom);old=g.SelectObject(mem,bmp)
g.BitBlt(mem,0,0,r.right,r.bottom,dc,0,0,0xCC0020);g.SelectObject(mem,old)
header=Header(c.sizeof(Header),r.right,-r.bottom,1,32,0,0,0,0,0,0)
b=c.create_string_buffer(r.right*r.bottom*4)
assert g.GetDIBits(mem,bmp,0,r.bottom,b,c.byref(header),0)
logical=Image.frombytes('RGB',(r.right,r.bottom),b.raw,'raw','BGRX')
logical.save(out/'official-dc.png')
roi=(36,84,328,548)
report['dpi_scale_comparison']={'static_visible_roi':roi,
    'mean_absolute_channel_error':{name:sum(ImageStat.Stat(ImageChops.difference(physical.crop(roi),logical.resize(physical.size,method).crop(roi))).mean)/3
      for name,method in [('point',Image.Resampling.NEAREST),('linear',Image.Resampling.BILINEAR),('cubic',Image.Resampling.BICUBIC)]}}
atlas=root/'work/ui_evidence/statuswindow-0.png'
if atlas.exists():
    f=np.array(Image.open(atlas));display=np.array(logical)[63:443,29:320].astype(np.int32)
    y,x=np.indices(f.shape[:2]);mask=(f[:,:,3]==255)&((x<16)|(x>273)|((y>335)&(y<348)))
    values=f[:,:,:3].astype(np.int32)//8
    report['a1r5g5b5_comparison']={'samples':int(mask.sum()),'frame_origin':[29,63],
       'mean_absolute_channel_error':{name:float(np.abs(display-pixels)[mask].mean()) for name,pixels in
          [('shift',values<<3),('unorm_round',(values*255+15)//31),('bit_replication',(values<<3)|(values>>2))]}}
logical.crop((29,63,320,443)).save(out/'official-status-logical.png')
g.DeleteObject(bmp);g.DeleteDC(mem);u.ReleaseDC(hwnd,dc);u.SetThreadDpiAwarenessContext(previous)
for name,begin,end in [('sprite_begin',0x67F1C6,0x67F602),('texture_upload',0x40A130,0x40A550),
                       ('alpha_pixel_pack',0x412C20,0x412C80)]:
    (out/f'{name}.txt').write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in pe.md.disasm(pe.read(begin,end-begin),begin)),encoding='utf-8')
(out/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report,indent=2))
if '--close' in sys.argv:
    u.SendMessageTimeoutW.argtypes=[c.c_void_p,c.c_uint32,c.c_size_t,c.c_ssize_t,c.c_uint32,c.c_uint32,c.POINTER(c.c_size_t)]
    result=c.c_size_t();u.SendMessageTimeoutW(hwnd,0x10,0,0,2,3000,c.byref(result))
    k.WaitForSingleObject.argtypes=[c.c_void_p,c.c_uint32]
    h=k.OpenProcess(0x100000,0,pid)
    if h:
        status=k.WaitForSingleObject(h,5000);k.CloseHandle(h);assert status==0,'official client did not close normally'
    print(f'gracefully closed authorized official client {pid}')
