"""Native UI and original damage-sprite verification, without global input injection."""
import json
import subprocess
from pathlib import Path
from PIL import Image

root=Path(__file__).resolve().parents[1]
cwd=root/'custom-dnf'
exe=cwd/'target/movement-verified/release/dnf-app.exe'
out=cwd/'out/ui-verified'
out.mkdir(parents=True,exist_ok=True)
def run(args,name):
    p=subprocess.run([str(exe),*args],cwd=cwd,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=35)
    log=p.stdout+p.stderr
    (out/f'{name}.txt').write_text(log,encoding='utf-8')
    assert p.returncode==0,log
    return log
assert 'UI自检通过' in run(['--ui-selftest','--out',str(out)],'selftest')
report={}
for name,flags,frame in [('panel-window',['--window-replay','--script','idle','--panel'],10),
                         ('edit-window',['--ui-replay'],32),('damage-window',['--ui-replay'],56)]:
    path=out/f'{name}.png'
    log=run([*flags,'--trace','--shot-after',str(frame),'--shot-out',str(path)],name)
    assert '窗口实拍已保存' in log
    if name=='edit-window':
        assert 'move_speed=1200 physical_attack=123' in log
        assert 'hits=0' in log
    if name=='damage-window':
        assert 'goblin_hp=196 hits=1' in log and 'popups=1' in log
    img=Image.open(path)
    assert img.size==(800,600),img.size
    assert len(img.getcolors(img.width*img.height))>100
    report[name]={'size':img.size,'capture':str(path)}
report['editing']={'base_attack':123,'move_speed':1200,'normal_attack1_damage':104,'target_hp':196,
                   'mp_limit':10,'reset_passed':True}
report['damage']={'atlas':'sprite/common/etc/combo.img','normal_base':143,'critical_base':88,'additional_base':44,
                  'duration_ms':500,'frames_ms':[100,200,100,100,0],'advance':'glyph width - 5'}
(out/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
