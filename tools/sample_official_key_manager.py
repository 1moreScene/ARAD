"""Read only the official checker/binding fields; never sends input or writes process memory."""
import ctypes as c,json,struct,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1]
raw=subprocess.check_output(['powershell','-NoProfile','-Command','[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); @(Get-Process ARAD -ErrorAction SilentlyContinue | Select-Object Id,Path) | ConvertTo-Json -Compress'],encoding='utf-8').strip()
rows=json.loads(raw) if raw else [];rows=[rows] if isinstance(rows,dict) else rows
target=(root/'40jp-client/40jp单机版/ARAD.exe').resolve().as_posix().lower()
row=next(p for p in rows if Path(p['Path']).resolve().as_posix().lower()==target)
k=c.WinDLL('kernel32');k.OpenProcess.argtypes=[c.c_uint32,c.c_int,c.c_uint32];k.OpenProcess.restype=c.c_void_p
k.ReadProcessMemory.argtypes=[c.c_void_p,c.c_void_p,c.c_void_p,c.c_size_t,c.POINTER(c.c_size_t)];k.CloseHandle.argtypes=[c.c_void_p]
h=k.OpenProcess(0x410,0,row['Id'])
def read(at,n):
    b=c.create_string_buffer(n);got=c.c_size_t();assert k.ReadProcessMemory(h,c.c_void_p(at),b,n,c.byref(got)) and got.value==n;return b.raw
def u32(at):return struct.unpack('<I',read(at,4))[0]
checker=0x8f8558
assert u32(checker)==0x6c490c
head=u32(checker+36);bindings=[];visited=set()
def visit(node):
    if node==head or node in visited:return
    visited.add(node);assert len(visited)<=64
    visit(u32(node));logical,device,slot=struct.unpack('<III',read(node+12,12));bindings.append({'id':logical,'device':device,'slot':slot});visit(u32(node+8))
visit(u32(head+4))
keyboard=u32(0x79c0b4)
for b in bindings:
    if b['device']==0 and b['slot']<107:b['dik']=u32(keyboard+784+4*b['slot'])
first,last=u32(checker+48),u32(checker+52);assert 0<=last-first<=65536
records=[]
for at in range(first,last,16):
    logical,press,hold,held,changed=struct.unpack('<IIIBB',read(at,14));records.append({'id':logical,'press_ms':press,'hold_ms':hold,'held':held,'changed':changed})
commands={}
for name,at in [('attack',0x9a0330),('jump',0x9a035c),('right_right',0x9a0388),('left_left',0x9a03b4)]:
    mode,state,enabled=struct.unpack('<IIB',read(at+32,9));commands[name]={'mode':mode,'state':state,'enabled':enabled}
report={'pid':row['Id'],'checker_vtable':hex(u32(checker)),'bindings':bindings,'records':records,'commands':commands,'retention_ms':u32(checker+60)}
k.CloseHandle(h)
out=root/'work/key_manager_evidence/live.json';out.write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report,indent=2))
