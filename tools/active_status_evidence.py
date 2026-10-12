"""Read-only official ActiveStatus PE/PVF evidence; keep all file hashes."""
import struct,json,hashlib,re,os
from pathlib import Path
from pvf_reader import PvfReader,decrypt_blob
import capstone
root=Path(__file__).resolve().parents[1];client=Path(os.environ.get('DNF_CLIENT_DIR',root/'40jp-client/Arad40'));out=root/'work/active_status_evidence';out.mkdir(exist_ok=True)
pvf=PvfReader(str(client/'Script.pvf'));scripts={};examples=[]
for _,off,size,key,path in pvf.entries:
    if not path.startswith('common/activestatus/') and not path.endswith('.atk'):continue
    pos=off+27+pvf.header['S'];raw,_=decrypt_blob(pvf.data[pos:pos+((size+3)&~3)],off,key);raw=raw[:size]
    if path.startswith('common/activestatus/'):
        (out/path.replace('/','__')).write_text(raw.decode('cp932',errors='replace'),encoding='utf8');scripts[path]=hashlib.sha256(raw).hexdigest()
    elif len(examples)<20 and re.search(rb'\[active status\]\s*`\[(?!none)[a-z]+\]`',raw,re.I):examples.append({'path':path,'tags':[x.decode('cp932',errors='replace') for x in re.findall(rb'\[active status\][^\r\n]+',raw,re.I)]})
b=(client/'ARAD.exe').read_bytes();h=struct.unpack_from('<I',b,60)[0];base=struct.unpack_from('<I',b,h+52)[0];secs=[]
for i in range(struct.unpack_from('<H',b,h+6)[0]):
    q=h+24+struct.unpack_from('<H',b,h+20)[0]+i*40;_,v,n,o=struct.unpack_from('<IIII',b,q+8);secs.append((base+v,n,o))
def read(v,n):
    for s,l,o in secs:
        if s<=v and v+n<=s+l:return b[o+v-s:o+v-s+n]
    raise ValueError(hex(v))
labels=[]
for addr in struct.unpack('<18I',read(0x791F38,72)):labels.append(read(addr,80).split(b'\0')[0].decode('ascii'))
rows=[]
for i in range(17):rows.append({'id':i,'token':labels[i],'fixed_seconds':struct.unpack('<i',read(0x6CB220+i*4,4))[0],'level1_level40':list(struct.unpack('<ff',read(0x6CB268+i*8,8))),'secondary':list(struct.unpack('<ff',read(0x6CB2F0+i*8,8)))})
(out/'tables.json').write_text(json.dumps(rows,indent=2),encoding='utf8')
(out/'manifest.json').write_text(json.dumps({'exe_sha256':hashlib.sha256(b).hexdigest(),'pvf_sha256':hashlib.sha256(pvf.data).hexdigest(),'labels':labels,'scripts':scripts,'attack_examples':examples},indent=2),encoding='utf8')
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
for name,start,end in [('admission',0x4D6030,0x4D6130),('level_time',0x497120,0x497300),('apply',0x497300,0x4973B0),('modifiers',0x5FAD60,0x5FAFC0),('hit_observer',0x5FA9A0,0x5FABA0),('start',0x5FB830,0x5FBE20),('burn',0x5FC120,0x5FC2F0),('bleed',0x5FC610,0x5FC900),('poison',0x5FCA50,0x5FCE10),('tick_exit',0x5FD010,0x5FD5C0),('sizing',0x497560,0x497840),('assets',0x497840,0x497EA0)]:
    (out/(name+'.txt')).write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in md.disasm(read(start,end-start),start)),encoding='utf8')
print('ActiveStatus labels:',labels);print('scripts',len(scripts),'attack examples',len(examples))
