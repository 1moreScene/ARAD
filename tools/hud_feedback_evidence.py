"""Read-only original gauge trails, failed-cast vibration and transient notices."""
import hashlib,json,struct
from pathlib import Path
import movement_evidence as pe
root=Path(__file__).resolve().parents[1];out=root/'work/hud_feedback_evidence'
out.mkdir(parents=True,exist_ok=True)
b=pe.blob
# PE resource directory: File / 115 / language. sub421300 extracts exactly this.
rva=struct.unpack_from('<I',b,pe.pe+24+96+16)[0];base=pe.base+rva
def directory(offset):
    h=pe.read(base+offset,16);n=sum(struct.unpack_from('<HH',h,12))
    for i in range(n):
        name,target=struct.unpack('<II',pe.read(base+offset+16+i*8,8))
        if name&0x80000000:
            p=name&0x7fffffff;length=struct.unpack('<H',pe.read(base+p,2))[0]
            name=pe.read(base+p+2,length*2).decode('utf-16le')
        yield name,target
file_dir=next(t&0x7fffffff for n,t in directory(0) if str(n).lower()=='file')
lang_dir=next(t&0x7fffffff for n,t in directory(file_dir) if n==115)
language,target=next(directory(lang_dir))
data_rva,size,page,_=struct.unpack('<IIII',pe.read(base+target,16))
raw=pe.read(pe.base+data_rva,size);text=raw.decode('cp932')
(out/'exe-string-table.txt').write_text(text,encoding='utf-8')
selected=[s for s in text.splitlines() if s.split('>')[0] in ('853','854','855')]
spans=[('gauge_trails',0x4688A0,0x468BA0),('gauge_crop',0x460920,0x460DC0),
    ('admission',0x4B49B0,0x4B4EE0),('vibration_update',0x4B7430,0x4B89D0),
    ('notice_insert',0x43BD50,0x43BF40),('notice_draw',0x43CC10,0x43D900),
    ('capsule_hover',0x464C60,0x465110),
    ('notice_init',0x43E150,0x43ECC0),('notice_expiry',0x441750,0x442280),('strings',0x421300,0x4213A0)]
for label,start,end in spans:
    (out/f'{label}.txt').write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}'
        for i in pe.md.disasm(pe.read(start,end-start),start)),encoding='utf-8')
report={'exe_sha256':hashlib.sha256(b).hexdigest(),'resource':{'type':'File','id':115,
    'language':language,'codepage':page,'size':size,'sha256':hashlib.sha256(raw).hexdigest()},
    'messages':selected,'gauge_tint_argb':hex((-6908266)&0xffffffff),
    'notice_argb':hex((-6567706)&0xffffffff)}
(out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
