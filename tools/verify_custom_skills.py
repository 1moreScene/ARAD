"""Real PVF/NPK/PCM Ghost Slash and both forced-cancel replays, then native capture."""
import json,re,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1];cwd=root/'custom-dnf'
exe=cwd/'target/movement-verified/release/dnf-app.exe'
out=cwd/'out/skill-verified';out.mkdir(parents=True,exist_ok=True)
report={}
info=subprocess.run([str(exe),'--info'],cwd=cwd,capture_output=True,text=True,encoding='utf-8',timeout=30)
assert info.returncode==0 and 'MP20 CD8000ms' in info.stdout and 'MP6 CD2000ms' in info.stdout,info.stdout+info.stderr
(out/'info.txt').write_text(info.stdout,encoding='utf-8')
for name in ['ghost','cancelghost','cancelupper']:
    result=subprocess.run([str(exe),'--headless','--script',name,'--frames','120','--trace',
        '--png-every','4','--out',str(out/name),'--audio-out',str(out/f'{name}.wav')],
        cwd=cwd,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=45)
    log=result.stdout+result.stderr;(out/f'{name}.txt').write_text(log,encoding='utf-8')
    assert result.returncode==0,log
    assert ('R_SM_GUE' if name!='cancelupper' else 'SM_UPPER_SLASH') in log
    assert max(map(int,re.findall(r'hits=(\d+)',log)))==1
    assert f'mp={144 if name=="cancelupper" else 130}' in log
    if name!='ghost':
        assert 'afterimages=1' in log
        assert 'afterimages=0' in log.split('afterimages=1',1)[1]
    else:
        assert 'goblin_hp=242' in log and 'ghost_fx=2' in log
    assert 'goblin_motion=Down' in log
    shot=15 if name=='ghost' else 10
    native=subprocess.run([str(exe),'--window-replay','--script',name,'--trace',
        '--shot-after',str(shot),'--shot-out',str(out/f'{name}-window.png')],cwd=cwd,
        capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=45)
    text=native.stdout+native.stderr;(out/f'{name}-window.txt').write_text(text,encoding='utf-8')
    assert native.returncode==0 and '窗口实拍已保存' in text,text
    assert int(re.findall(r'dropped=(\d+)',text)[-1])==0,text
    assert int(re.findall(r'audible_completed=(\d+)',text)[-1])>0,text
    report[name]={'headless_hits':1,'mp':144 if name=='cancelupper' else 130,
        'native_capture':str(out/f'{name}-window.png'),'audio':str(out/f'{name}.wav')}
(out/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
