"""Save read-only 40JP PE and PVF evidence for goblin animation and hit reactions."""
import hashlib
import json
from pathlib import Path
from pvf_reader import PvfReader
import movement_evidence as pe

root=Path(__file__).resolve().parents[1]
out=root/"work/feedback_evidence"
out.mkdir(parents=True,exist_ok=True)
pvf=PvfReader(str(root/"40jp-client/40jp单机版/Script.pvf"))
report={"exe_sha256":hashlib.sha256(pe.blob).hexdigest(),"pvf_sha256":hashlib.sha256(pvf.data).hexdigest(),
        "scripts":{},"verified_functions":{}}
for name in ["monster/goblin/goblin.mob","equipment/monster/goblin/gcc1.equ",
             "equipment/monster/goblin/weapon01.equ","character/swordman/swordman.chr"]:
    raw=pvf.read(name)
    report["scripts"][name]={"sha256":hashlib.sha256(raw).hexdigest()}
    (out/Path(name).name).write_text(raw.decode("cp932",errors="replace"),encoding="utf-8")
for name,begin,end in [
    ("goblin_frame_setup",0x59A990,0x59AA1A),("hit_recovery_getter",0x58E6D0,0x58E710),
    ("weapon_stop_getter",0x4B1910,0x4B195A),("context_stop_scale",0x4AF8E0,0x4AF999),
    ("height_to_velocity",0x4A0550,0x4A05BD),("attribute_defaults",0x4A1210,0x4A1670),
]:
    lines=[f"{i.address:08X} {i.bytes.hex():20} {i.mnemonic:8} {i.op_str}"
           for i in pe.md.disasm(pe.read(begin,end-begin),begin)]
    (out/f"{name}.txt").write_text("\n".join(lines),encoding="utf-8")
    report["verified_functions"][name]={"begin":f"{begin:08X}","end":f"{end:08X}"}
report["key_results"]={"mob_hit_recovery":500,"mob_weight":45000,
    "short_sword_hit_info":["cut","blood",100,1.0],"normal_stun_ms":375,"goblin_down_gates":[2,2,2,3,4],
    "runtime_hit_stop_scale":"(2 - attribute_float_at_1292) * context_stop","default_attribute_float_at_1292":1.0,
    "default_effective_short_stop_ms":100,
    "ordinary_damage_ignores_lift_up":"4D71B0 selects state3 on ground; only state4 reads push/lift",
    "state4_first_landing":"height = recorded_peak/5; starts bounce; apex advances2->3; second landing4",
    "dead_fade_ms":[100,250]}
(out/"report.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report["key_results"],ensure_ascii=False,indent=2))
