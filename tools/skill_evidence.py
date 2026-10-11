"""Save original skill, attack and animation scripts used for Ghost Slash/cancel research."""
import hashlib,json
from pathlib import Path
from pvf_reader import PvfReader
root=Path(__file__).resolve().parents[1];out=root/'work/skill_evidence';out.mkdir(parents=True,exist_ok=True)
pvf=PvfReader(str(root/'40jp-client/40jp单机版/Script.pvf'))
paths=['skill/swordmanskill.lst','skill/swordman/hardattack.skl','skill/swordman/upperslash.skl',
       'skill/swordman/hardattackcharge.skl','character/swordman/swordman.chr',
       'character/swordman/animation/hardattack.ani','character/swordman/attackinfo/hardattack.atk',
       'character/swordman/effect/animation/hardattack1.ani','character/swordman/effect/animation/hardattack2.ani']
paths += [e[4] for e in pvf.entries if e[4].startswith('skill/') and 'cancel' in e[4] and e[4].endswith('.skl')]
report={}
for path in paths:
    try:raw=pvf.read(path)
    except KeyError:continue
    name=path.replace('/','__');(out/name).write_text(raw.decode('cp932',errors='replace'),encoding='utf-8')
    report[path]={'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
(out/'scripts.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('\n'.join(report))
import movement_evidence as pe
for name,start,end in [('skill_parser',0x502F00,0x504960),
    ('runtime_tables',0x482A90,0x483C60),('runtime_getters',0x47EA80,0x47EB10),
    ('level_values',0x47EF20,0x47F190),('cancel_passive_gate',0x5EE1E0,0x5EE390),
    ('skill_admission_mp',0x4B49B0,0x4B4EE0),('ghost_state',0x5F39F0,0x5F48A0)]:
    (out/f'{name}.txt').write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}'
        for i in pe.md.disasm(pe.read(start,end-start),start)),encoding='utf-8')
(out/'exe.json').write_text(json.dumps({'sha256':hashlib.sha256(pe.blob).hexdigest(),
    'white_afterimage_timing':'not yet confirmed; local hold120ms + fade480ms = 600ms'},indent=2),encoding='utf-8')
