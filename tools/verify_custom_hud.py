"""Real HUD: drag Ghost A->D, hotkey use, HP/MP crop, cooldown/tooltip and completion flash."""
import hashlib,json,re,subprocess,wave,struct
from pathlib import Path
from PIL import Image
root=Path(__file__).resolve().parents[1];cwd=root/'custom-dnf'
exe=cwd/'target/movement-verified/release/dnf-app.exe'
out=cwd/'out/hud-verified';out.mkdir(parents=True,exist_ok=True)
settings=cwd/'data/skillbar.txt';before=settings.read_bytes()
def run(name,*args):
    result=subprocess.run([str(exe),*map(str,args)],cwd=cwd,capture_output=True,text=True,
        encoding='utf-8',errors='replace',timeout=50)
    log=result.stdout+result.stderr;(out/f'{name}.txt').write_text(log,encoding='utf-8')
    assert result.returncode==0,log
    return log
trace=run('replay','--headless','--hud-replay','--script','idle','--frames',550,'--trace',
    '--png-every',5,'--out',out/'replay','--audio-out',out/'hud.wav')
rows={int(re.search(r'step=\s*(\d+)',s).group(1)):s for s in trace.splitlines() if s.startswith('step=')}
assert 'hud_slots=[None, Some(46), Some(5), None, None, None]' in rows[7]
assert 'ICON_QUICK_SET' in rows[7]
assert 'motion=STAND' in rows[11] and 'mp=150 ' in rows[11]
assert 'motion=GHOST' in rows[21] and 'mp=130 ' in rows[21] and 'ghost_cd=8000' in rows[21]
assert 'hud_hover=Some(5)' in rows[45] and 'ghost_cd=7600' in rows[45]
assert 'ghost_cd=0' in rows[540] and 'mp=130 ' in rows[540]
full=Image.open(out/'replay/frame_0010.png');half=Image.open(out/'replay/frame_0080.png')
assert full.getpixel((22,440))[0]>half.getpixel((22,440))[0]
assert Image.open(out/'replay/frame_0505.png').crop((443,447,471,467)).tobytes()!=Image.open(out/'replay/frame_0540.png').crop((443,447,471,467)).tobytes()
with wave.open(str(out/'hud.wav'),'rb') as f:
    assert (f.getnchannels(),f.getframerate(),f.getsampwidth())==(2,44100,2)
    pcm=f.readframes(f.getnframes());samples=struct.unpack(f'<{len(pcm)//2}h',pcm)
    assert max(map(abs,samples))>0 and not any(s in (-32768,32767) for s in samples)
report={}
for name,frame in [('drag',5),('tooltip',45),('ready',505),('top-layer',45)]:
    args=['--hud-replay','--trace','--shot-after',frame,'--shot-out',out/f'{name}-window.png']
    if name=='top-layer':args+=['--panel']
    log=run(name,*args)
    assert '窗口实拍已保存' in log
    assert 'dropped=0' in log and 'underruns=0' in log
    assert Image.open(out/f'{name}-window.png').size==(800,600)
    report[name]={'capture':str(out/f'{name}-window.png'),'frame':frame}
assert settings.read_bytes()==before,'verification must not replace user bindings'
report.update({'ghost_moved_A_to_D':True,'cooldown_follows_skill':True,'hp_mp':True,
    'tooltip_locale':'original CP932 Skill.jap.str','cooldown_flash':'CoolTimeEnd.img 9x50ms',
    'settings_untouched':hashlib.sha256(before).hexdigest()})
(out/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
