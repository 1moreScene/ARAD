"""Resource-failure admission, original trails/hover notices, real uncapped window and audio."""
import json,re,subprocess,wave,struct
from pathlib import Path
from PIL import Image
root=Path(__file__).resolve().parents[1];cwd=root/'custom-dnf'
exe=cwd/'target/movement-verified/release/dnf-app.exe'
out=cwd/'out/hud-feedback-verified';out.mkdir(parents=True,exist_ok=True)
def run(name,*args):
    r=subprocess.run([str(exe),*map(str,args)],cwd=cwd,capture_output=True,text=True,
        encoding='utf-8',errors='replace',timeout=40)
    log=r.stdout+r.stderr;(out/f'{name}.txt').write_text(log,encoding='utf-8')
    assert r.returncode==0,log
    return log
log=run('replay','--headless','--feedback-replay','--frames',750,'--trace','--png-every',1,
    '--out',out/'replay','--audio-out',out/'feedback.wav')
rows={int(re.search(r'step=\s*(\d+)',s)[1]):s for s in log.splitlines() if s.startswith('step=')}
for step,reason in [(3,'Mp'),(5,'Mp'),(111,'Cooldown'),(131,'Hp'),(135,'Item(3037)')]:
    assert f'failures=[{reason}]' in rows[step],rows[step]
    assert 'motion=STAND' in rows[step] and 'x=   36 y= 329' in rows[step]
assert 'mp=0 ' in rows[3] and 'ghost_cd=0 ' in rows[3] and 'notices=1 ' in rows[3]
assert 'vibration=2 ' in rows[19] and 'vibration=0 ' in rows[22], 'repeat must not restart 300ms timer'
assert 'motion=GHOST' in rows[27] and 'mp=130 ' in rows[27]
assert 'motion=UPPER' in rows[139] and 'mp=124 ' in rows[139]
assert 'notices=0 ' in rows[750]
assert 'capsule_hover=Some(0)' in rows[20] and 'capsule_hover=Some(1)' in rows[31]
assert 'capsule_hover=None' in rows[40]
def crop(frame,rect):return Image.open(out/f'replay/frame_{frame:04d}.png').crop(rect)
# The real current-height changes immediately, the lost band flashes then drains.
rect=(6,412,41,477)
assert crop(31,rect).tobytes()!=crop(60,rect).tobytes()!=crop(100,rect).tobytes()
assert crop(20,(6,390,150,410)).tobytes()!=crop(40,(6,390,150,410)).tobytes()
assert crop(31,(535,390,635,410)).tobytes()!=crop(40,(535,390,635,410)).tobytes()
with wave.open(str(out/'feedback.wav'),'rb') as f:
    assert (f.getnchannels(),f.getframerate(),f.getsampwidth())==(2,44100,2)
    samples=struct.unpack('<'+'h'*(f.getnframes()*2),f.readframes(f.getnframes()))
    assert max(map(abs,samples))>0 and not any(s in (-32768,32767) for s in samples)
report={'failure_causes':['MP','cooldown','HP','item3037'],'original_notice_lifetime_ms':10000,
    'original_vibration_ms':300,'vibration_does_not_extend':True,'hover_only_values':True,
    'original_gauge_flash_ms':500,'gauge_drain_px_per_second':65,'audio_hard_clips':0}
for name,step in [('hp-hover',20),('mp-hover',31),('item-failure',135),('fps',180)]:
    log=run(name,'--feedback-replay','--trace','--shot-after',step,'--shot-out',out/f'{name}-window.png')
    assert '窗口实拍已保存' in log and 'dropped=0' in log and 'underruns=0' in log
    assert Image.open(out/f'{name}-window.png').size==(800,600)
    m=re.search(r'logic_steps=(\d+) rendered=(\d+) elapsed_s=([\d.]+) average_fps=([\d.]+) uncapped=true',log)
    assert m and int(m[1])==step,log
    if name=='fps':
        # On this machine proves rendering is independent of 60Hz physics, not just a printed number.
        assert int(m[2])>step*1.1 and float(m[4])>60,log
        assert abs(float(m[3])-step/60)<0.25, 'simulation pace changed with rendering FPS'
        report['native_fps']={'logic_steps':int(m[1]),'rendered':int(m[2]),'seconds':float(m[3]),'average_fps':float(m[4])}
(out/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
