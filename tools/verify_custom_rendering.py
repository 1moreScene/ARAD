"""Check live ARAD evidence and our actual native linear scaling against the logical frame."""
import json
import subprocess
from pathlib import Path
from PIL import Image,ImageChops,ImageStat,ImageDraw

root=Path(__file__).resolve().parents[1];cwd=root/'custom-dnf';out=cwd/'out/rendering-verified';out.mkdir(parents=True,exist_ok=True)
exe=cwd/'target/movement-verified/release/dnf-app.exe'
evidence=json.loads((root/'work/rendering_evidence/report.json').read_text(encoding='utf-8'))
assert evidence['backbuffer']==[640,480,22]
assert evidence['sprite_begin']=='0x67f1c6'
assert evidence['a1r5g5b5_comparison']['mean_absolute_channel_error']['bit_replication']==0
metrics=evidence['dpi_scale_comparison']['mean_absolute_channel_error']
assert metrics['linear']<metrics['point']/10
def run(args,name):
    p=subprocess.run([str(exe),*map(str,args)],cwd=cwd,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=30)
    (out/f'{name}.txt').write_text(p.stdout+p.stderr,encoding='utf-8');assert p.returncode==0,p.stdout+p.stderr
run(['--headless','--panel','--script','idle','--frames',1,'--out',out/'logical'],'logical')
run(['--window-replay','--panel','--script','idle','--shot-after',8,'--shot-out',out/'native.png'],'native')
logical=Image.open(out/'logical/frame_last.png').convert('RGB');native=Image.open(out/'native.png').convert('RGB')
linear=logical.resize(native.size,Image.Resampling.BILINEAR);point=logical.resize(native.size,Image.Resampling.NEAREST)
def error(a,b):return sum(ImageStat.Stat(ImageChops.difference(a,b)).mean)/3
scores={'linear':error(native,linear),'point':error(native,point)}
assert scores['linear']<0.4 and scores['point']>5,scores
run(['--window-replay','--panel','--script','idle','--scale',1,'--shot-after',8,'--shot-out',out/'native-one.png'],'native-one')
one=Image.open(out/'native-one.png').convert('RGB');assert error(one,logical)==0
# Same source and physical size, magnified only for inspection of the differing sampling.
region=(530,82,653,224)
panels=[point.crop(region).resize((246,284),Image.Resampling.NEAREST),native.crop(region).resize((246,284),Image.Resampling.NEAREST)]
sheet=Image.new('RGB',(492,308),'#181a20');draw=ImageDraw.Draw(sheet)
draw.text((12,6),'BEFORE: POINT',fill='white');draw.text((258,6),'AFTER: LINEAR',fill='white')
for i,img in enumerate(panels):sheet.paste(img,(246*i,24))
sheet.save(out/'sampling-comparison.png')
report={'official':evidence,'custom_native':{'size':native.size,'reference_mae':scores,'scale_one_identity':True},
        'comparison':str(out/'sampling-comparison.png')}
(out/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(report['custom_native'],indent=2))
