"""Exercise movement loops, running attacks, upper slash and goblin feedback in native windows."""
import json
import re
import struct
import subprocess
import wave
from pathlib import Path

root=Path(__file__).resolve().parents[1]
cwd=root/"custom-dnf"
exe=cwd/"target/movement-verified/release/dnf-app.exe"
out=cwd/"out/battle-verified"
out.mkdir(parents=True,exist_ok=True)
report={}

def run(name,args):
    p=subprocess.run([str(exe),*map(str,args)],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace",timeout=45)
    log=p.stdout+p.stderr
    (out/f"{name}.txt").write_text(log,encoding="utf-8")
    assert p.returncode==0,log
    assert "音效启用失败" not in log and "缺少可播放音效" not in log,log
    return log

for scenario,frames,mode in [("walk",120,"WALK"),("run",120,"RUN"),("dashattack",120,"DASHATK"),("upper",180,"UPPER")]:
    wav=out/f"{scenario}.wav"
    log=run(scenario,["--headless","--script",scenario,"--frames",frames,"--trace","--png-every",8,
                      "--out",out/scenario,"--audio-out",wav])
    assert f"motion={mode}" in log,log
    with wave.open(str(wav),"rb") as w:
        raw=w.readframes(w.getnframes());samples=struct.unpack(f"<{len(raw)//2}h",raw)
    assert any(samples) and not any(s in (-32768,32767) for s in samples)
    max_hits=max(map(int,re.findall(r"hits=(\d+)",log)))
    max_z=max(map(float,re.findall(r"goblin_z=([\d.]+)",log)))
    if scenario in ("walk","run"):
        tag="WALK_SOIL_LOOP" if scenario=="walk" else "RUN_SOIL_LOOP"
        assert f'loop=Some("{tag}")' in log
    elif scenario=="dashattack":
        assert 'audio_tags=["SM_THRUST"]' in log and max_hits==1
        assert "R_MINERALSWDA_HIT" in log and "R_GOBLIN_DMG" in log
    else:
        assert log.count('audio_tags=["SM_UPPER_SLASH"]')==2
        assert "mp=138" in log and max_hits==2 and max_z>70
    # Stop the shot before motion completion to visually inspect the actual action.
    shot=24 if scenario=="upper" else 26 if scenario=="dashattack" else 60
    native=run(f"{scenario}-window",["--window-replay","--script",scenario,"--trace","--shot-after",shot,
                                   "--shot-out",out/f"{scenario}-window.png"])
    assert f"motion={mode}" in native and "窗口实拍已保存" in native
    assert int(re.findall(r"audible_completed=(\d+)",native)[-1])>0
    assert int(re.findall(r"dropped=(\d+)",native)[-1])==0
    if scenario in ("walk","run"):
        assert int(re.findall(r"loops_started=(\d+)",native)[-1])>=1
    else:
        assert max(map(int,re.findall(r"hits=(\d+)",native)))>=1
    report[scenario]={"headless_frames":frames,"max_hits":max_hits,"goblin_apex":max_z,
                      "audio_peak":max(map(abs,samples)),"native_capture":str(out/f"{scenario}-window.png"),
                      "native_underruns":int(re.findall(r"underruns=(\d+)",native)[-1])}

(out/"verification.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))
