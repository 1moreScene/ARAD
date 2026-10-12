"""Read-only 40JP grabability and skill31 PVF/PE evidence."""
import os,json,hashlib,re,struct
from pathlib import Path
from pvf_reader import PvfReader,decrypt_blob
root=Path(__file__).resolve().parents[1]
client=Path(os.environ.get('DNF_CLIENT_DIR',root/'40jp-client/Arad40'))
out=root/'work/grab_evidence';out.mkdir(parents=True,exist_ok=True)
pvf=PvfReader(str(client/'Script.pvf'));report={};tags={}
for _,offset,size,key,path in pvf.entries:
    interesting=('waveslash' in path or 'grabblastblood' in path or path in ['monster/goblin/goblin.mob','character/swordman/swordman.chr','skill/swordmanskill.lst','character/swordman/animation/grab.ani'])
    if not interesting and not path.endswith(('.mob','.ani','.atk','.chr')):continue
    seek=offset+27+pvf.header['S']
    raw,_=decrypt_blob(pvf.data[seek:seek+((size+3)&~3)],offset,key);raw=raw[:size]
    if path.endswith(('.mob','.ani','.atk','.chr')):
        found=re.findall(rb'\[grab[a-z ]*\]',raw,re.I)
        if found:tags[path]=[tag.decode() for tag in found]
    if not interesting:continue
    (out/path.replace('/','__')).write_text(raw.decode('cp932',errors='replace'),encoding='utf-8')
    report[path]={'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
(out/'scripts.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
(out/'grab_tags.json').write_text(json.dumps(tags,indent=2),encoding='utf-8')
(out/'summary.json').write_text(json.dumps({'pvf_sha256':hashlib.sha256(pvf.data).hexdigest(),'waveslash_paths':[e[4] for e in pvf.entries if 'waveslash' in e[4]],'grab_tag_entries':len(tags)},indent=2),encoding='utf-8')
import capstone
blob=(client/'ARAD.exe').read_bytes()
header=struct.unpack_from('<I',blob,0x3C)[0]
base=struct.unpack_from('<I',blob,header+52)[0]
sections=[]
for i in range(struct.unpack_from('<H',blob,header+6)[0]):
    pos=header+24+struct.unpack_from('<H',blob,header+20)[0]+40*i
    _,rva,size,offset=struct.unpack_from('<IIII',blob,pos+8)
    sections.append((base+rva,size,offset))
def read(address,count):
    for start,size,offset in sections:
        if start<=address and address+count<=start+size:
            return blob[offset+address-start:offset+address-start+count]
    raise ValueError(hex(address))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
for name,start,end in [('contact_gate',0x5EF0A0,0x5EF3F0),('grabable_gate',0x4D5C10,0x4D5C50),('damage_type',0x4AED20,0x4AED60),('release',0x5F0460,0x5F0660),('bind_tick',0x650D50,0x650E80),('bind_start',0x650E80,0x650F20),('swordman_tick',0x5F10C0,0x5F39F0),('swordman_enter',0x5F39F0,0x5F6000)]:
    (out/(name+'.txt')).write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in md.disasm(read(start,end-start),start)),encoding='utf-8')
(out/'exe.json').write_text(json.dumps({'sha256':hashlib.sha256(blob).hexdigest()},indent=2),encoding='utf-8')
print(f'grab evidence: {len(report)} scripts, {len(tags)} grab-tag entries; PVF+PE hashes saved')
