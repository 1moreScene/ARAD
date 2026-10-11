"""Verify the built client, keeping logs and PNGs for the movement scenarios."""
import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path

root=Path(__file__).resolve().parents[1]
cwd=root/"custom-dnf"
exe=cwd/"target/movement-verified/release/dnf-app.exe"
output=cwd/"out/movement-verified"
output.mkdir(parents=True,exist_ok=True)
report={"executable":str(exe),"exe_sha256":hashlib.sha256(exe.read_bytes()).hexdigest(),"cases":{}}
for scenario,frames in [("mashtap",120),("turn",90),("airturn",90),("jump",90)]:
    case_out=output/scenario
    result=subprocess.run([str(exe),"--headless","--script",scenario,"--frames",str(frames),
        "--out",str(case_out),"--trace"],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace")
    log=result.stdout+result.stderr
    (output/f"{scenario}.txt").write_text(log,encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"{scenario} failed: {result.returncode}; see log")
    rows=[]
    for line in log.splitlines():
        match=re.search(r"step=\s*(\d+) motion=(\w+) .*?phase=(\w+) dash_candidate=([+-]\d+)",line)
        if match:
            rows.append((int(match[1]),match[2],match[3],int(match[4])))
    if len(rows)!=frames:
        raise AssertionError(f"{scenario}: expected {frames} trace rows, got {len(rows)}")
    counts=Counter(row[1] for row in rows)
    phases=Counter(row[2] for row in rows)
    if scenario=="mashtap":
        assert counts=={"WALK":frames},counts
        assert any(row[3]!=0 for row in rows),"input candidates must be distinct from sustained run"
    if scenario=="turn":
        assert counts["RUN"]>0
        assert all(row[1]=="WALK" for row in rows if row[0]>=41),"reverse did not cancel run"
    if scenario in ("airturn","jump"):
        assert all(phases[phase]>0 for phase in ("Preparing","Flying","Landing","Ground")),phases
        assert counts["RUN"]==0
    report["cases"][scenario]={"frames":frames,"motions":dict(counts),"phases":dict(phases),
        "dash_candidates":sum(row[3]!=0 for row in rows),"log":str(output/f"{scenario}.txt")}
(output/"verification.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))
