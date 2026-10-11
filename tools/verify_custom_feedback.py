"""Check scripted goblin hold gates, recovery and native rendering against recorded PE/PVF evidence."""
import json
import re
import subprocess
from pathlib import Path

root=Path(__file__).resolve().parents[1]
cwd=root/"custom-dnf"
exe=cwd/"target/movement-verified/release/dnf-app.exe"
out=cwd/"out/feedback-verified"
out.mkdir(parents=True,exist_ok=True)
report={}
for scenario in ["attack","upper","dashattack"]:
    result=subprocess.run([str(exe),"--headless","--script",scenario,"--frames","200","--trace","--png-every","8",
                           "--out",str(out/scenario),"--audio-out",str(out/f"{scenario}.wav")],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace",timeout=45)
    log=result.stdout+result.stderr
    (out/f"{scenario}.txt").write_text(log,encoding="utf-8")
    assert result.returncode==0,log
    assert 'actor_stop=100.0 target_stop=100.0' in log
    hits=max(map(int,re.findall(r"hits=(\d+)",log)))
    if scenario=="attack":
        assert hits==3 and "goblin_hp=246" in log
        assert "goblin_motion=Damage" in log
    elif scenario=="upper":
        assert hits==2 and "bounce=true" in log
        assert "goblin_motion=Down goblin_frame=2 bounce=true" in log
        assert "goblin_motion=Down goblin_frame=3 bounce=true" in log
        assert "goblin_motion=Down goblin_frame=4 bounce=false" in log
        assert "goblin_motion=Sit" in log
    else:
        assert hits==1
        assert max(map(float,re.findall(r"goblin_z=([\d.]+)",log)))==0.0
    shot=24 if scenario=="upper" else 26 if scenario=="dashattack" else 18
    native=subprocess.run([str(exe),"--window-replay","--script",scenario,"--trace","--shot-after",str(shot),
                           "--shot-out",str(out/f"{scenario}-window.png")],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace",timeout=30)
    text=native.stdout+native.stderr
    (out/f"{scenario}-window.txt").write_text(text,encoding="utf-8")
    assert native.returncode==0 and "窗口实拍已保存" in text,text
    assert "gcc1.equ" in text and "weapon01.equ" in text and "stop_ms: 100" in text
    assert int(re.findall(r"audible_completed=(\d+)",text)[-1])>0
    assert int(re.findall(r"dropped=(\d+)",text)[-1])==0
    report[scenario]={"hits":hits,"peak_height":max(map(float,re.findall(r"goblin_z=([\d.]+)",log))),
                      "scripted_layers":3,"actor_stop_ms":100,"target_stop_ms":100,
                      "native_capture":str(out/f"{scenario}-window.png")}
(out/"verification.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))
