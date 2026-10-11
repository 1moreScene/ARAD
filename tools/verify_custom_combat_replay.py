"""Verify playable attack actions through the real client input/render pipeline."""
import json
import re
import subprocess
from pathlib import Path

root=Path(__file__).resolve().parents[1]
cwd=root/"custom-dnf"
exe=cwd/"target/movement-verified/release/dnf-app.exe"
out=cwd/"out/combat-verified"
out.mkdir(parents=True,exist_ok=True)
report={}
for scenario in ["attack","airattack"]:
    result=subprocess.run([str(exe),"--headless","--script",scenario,"--frames","120","--trace","--png-every","6","--out",str(out/scenario)],
        cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace")
    log=result.stdout+result.stderr
    (out/f"{scenario}.txt").write_text(log,encoding="utf-8")
    assert result.returncode==0,log
    motions=re.findall(r"motion=(\w+)",log)
    stages={int(x) for x in re.findall(r"attack_stage=(\d+)",log)}
    budgets={int(x) for x in re.findall(r"air_attacks=(\d+)",log)}
    assert len(motions)==120
    if scenario=="attack":
        assert {1,2,3}<=stages,stages
        assert "ATTACK" in motions and motions[-1]=="STAND"
    else:
        assert "AIRATK" in motions and "JUMP" in motions and motions[-1]=="STAND"
        assert budgets=={0,1},budgets
    # Script drives the same Game.set_key -> Actor -> official ANI renderer used
    # by keyboard input, in an actual native 640x480 game window.
    result=subprocess.run([str(exe),"--window-replay","--script",scenario,"--trace","--shot-after","25",
        "--shot-out",str(out/f"{scenario}-window.png")],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace")
    native=result.stdout+result.stderr
    (out/f"{scenario}-window.txt").write_text(native,encoding="utf-8")
    expected="ATTACK" if scenario=="attack" else "AIRATK"
    assert result.returncode==0 and f"motion={expected}" in native,native
    assert "窗口实拍已保存" in native,native
    report[scenario]={"normal_stages":sorted(stages),"air_attack_budget":sorted(budgets),
        "headless_frames":len(motions),"native_action_seen":expected,"native_capture":str(out/f"{scenario}-window.png")}
(out/"verification.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))
