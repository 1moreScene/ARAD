"""Read-only 40JP keyboard slots, logical bindings and CNCommandChecker evidence."""
import hashlib,json,struct
from pathlib import Path
from pvf_reader import PvfReader
import movement_evidence as pe
import capstone

root=Path(__file__).resolve().parents[1];out=root/'work/key_manager_evidence';out.mkdir(parents=True,exist_ok=True)
slots=[None]*107
registers={}
for ins in pe.md.disasm(pe.read(0x415240,0x415660-0x415240),0x415240):
    ops=ins.operands
    if ins.mnemonic=='mov' and len(ops)==2:
        value=ops[1].imm if ops[1].type==capstone.x86.X86_OP_IMM else registers.get(ops[1].reg) if ops[1].type==capstone.x86.X86_OP_REG else None
        if ops[0].type==capstone.x86.X86_OP_REG:registers[ops[0].reg]=value
        if ops[0].type==capstone.x86.X86_OP_MEM and ops[0].mem.base==capstone.x86.X86_REG_ECX:
            offset=ops[0].mem.disp
            if 784<=offset<=1208 and offset%4==0:slots[(offset-784)//4]=value
assert all(x is not None for x in slots)
logical_slots=[2,3,0,1,80,78,57,76,13]
names=['Up','Down','Left','Right','Skill','Attack','Jump','Creature','Buff']
report={'exe_sha256':hashlib.sha256(pe.blob).hexdigest(),'device_slots':slots,
        'logical_bindings':{name:{'slot':slot,'dik':slots[slot]} for name,slot in zip(names,logical_slots)},
        'skill_shortcuts':[slots[x] for x in [55,73,58,60,61,62]],'item_shortcuts':[slots[x] for x in range(46,52)]}
for name,start,end in [('keyboard_slots',0x415240,0x415660),('keyboard_sample',0x415770,0x4157E0),
                       ('registration',0x4C0090,0x4C0140),('records',0x4C0140,0x4C0360),
                       ('matcher',0x4BF540,0x4BF910),('priority',0x4BF910,0x4BFBC0),
                       ('default_commands',0x4B5B80,0x4B5FD0),('command_constructor',0x436F50,0x436FC0),
                       ('global_bindings',0x439AB0,0x43B4C0)]:
    (out/f'{name}.txt').write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in pe.md.disasm(pe.read(start,end-start),start)),encoding='utf-8')
pvf=PvfReader(str(root/'40jp-client/40jp单机版/Script.pvf'))
blocks={}
for _,_,_,_,path in pvf.entries:
    if not path.startswith('skill/') or not path.endswith('.skl'):continue
    text=pvf.read(path).decode('cp932',errors='replace');lower=text.lower();at=lower.find('[command]')
    if at>=0:
        end=lower.find('[/command]',at)
        if end>=0:blocks[path]=text[at:end+10]
(out/'skill_commands.json').write_text(json.dumps(blocks,ensure_ascii=False,indent=2),encoding='utf-8')
report['skill_command_blocks']=len(blocks)
(out/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps({k:report[k] for k in ['logical_bindings','skill_shortcuts','item_shortcuts','skill_command_blocks']},indent=2))
