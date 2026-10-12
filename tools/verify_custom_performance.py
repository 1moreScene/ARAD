"""Sequential release performance/quality gates. Never benchmark competing processes.

--baseline must support --performance-benchmark (a previously saved instrumented build).
CPU-only throughput is reported separately from real native presentation FPS.
"""
import argparse
import csv
import json
import os
import re
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[1]
cwd = root / 'custom-dnf'
parser = argparse.ArgumentParser()
parser.add_argument('--exe', type=Path, default=cwd/'target/active-status-verified/release/dnf-app.exe')
parser.add_argument('--baseline', type=Path)
parser.add_argument('--out', type=Path, default=cwd/'out/performance-verified')
parser.add_argument('--enforce', action='store_true')
parser.add_argument('--min-fps', type=float, default=300)
parser.add_argument('--max-p95-ms', type=float, default=5)
parser.add_argument('--quality-scope',choices=['all','status'],default='all',help='status excludes expected debug UI changes from old-build pixel comparison')
args = parser.parse_args()
args.out=args.out.resolve() # subprocesses run from custom-dnf, so destinations must be absolute
args.out.mkdir(parents=True, exist_ok=True)

def run(exe, arguments, log):
    with log.open('w', encoding='utf-8') as stream:
        subprocess.run([str(exe.resolve()), *map(str, arguments)], cwd=cwd,
                       stdout=stream, stderr=subprocess.STDOUT, timeout=120, check=True)

def benchmark(exe, name):
    destination=args.out/name
    run(exe, ['--performance-benchmark','--out',destination], args.out/(name+'.log'))
    with (destination/'performance.csv').open(encoding='utf-8') as stream:
        return {row['case']: {key:float(value) for key,value in row.items()
                              if key not in ['case','armor']} for row in csv.DictReader(stream)}

def native(exe, name, panel=False):
    log=args.out/(name+'.log')
    command=['--mute','--scale','1.25','--shot-after','240','--shot-out',args.out/(name+'.png')]
    if panel: command += ['--show-keys']
    run(exe,command,log)
    text=log.read_text(encoding='utf-8')
    fps=float(re.search(r'average_fps=([\d.]+)',text)[1])
    line=re.search(r'^PERF .*$',text,re.M)[0]
    metrics={key:float(value) for key,value in re.findall(r'(\w+)=([\d.]+)',line)}
    return {'fps':fps,**metrics}

report={'cpu':{},'native':{},'gates':{},'quality':{},'comparison':{}}
if args.baseline:
    report['cpu']['before']=benchmark(args.baseline,'before')
    if os.name=='nt': report['native']['before']=native(args.baseline,'native-before')
report['cpu']['after']=benchmark(args.exe,'after')
if os.name=='nt':
    report['native']['after']=native(args.exe,'native-after')
    report['native']['panel']=native(args.exe,'native-panel',panel=True)
    n=report['native']['after']
    report['gates']['native_fps']=n['fps']>=args.min_fps
    report['gates']['native_p95']=n['p95_frame_ms']<=args.max_p95_ms
    report['gates']['native_slow_frame_ratio']=n['slow_frames']/max(1,n['samples'])<=0.01
for name,row in report['cpu']['after'].items():
    # Warm CPU frame budgets (render + bilinear scaling to800x600).
    budget=8.333 if name in ['armor_32','grab_burst_32','active_status_32'] else 5.0 if name in ['debug_panel','active_status_panel'] else 4.0 if name=='normal_32' else 3.333
    report['gates'][name]=row['mean_render_ms']+row['mean_scale_ms']<=budget
    if args.baseline and name in report['cpu']['before']:
        old=report['cpu']['before'][name]
        report['comparison'][name]={'cpu_speedup':row['cpu_fps']/old['cpu_fps']}
if args.baseline and os.name=='nt':
    report['comparison']['native_speedup']=report['native']['after']['fps']/report['native']['before']['fps']

for kind,flag in [('debug','--debug-selftest'),('status','--status-selftest')]:
    after=args.out/'quality-after'/kind
    run(args.exe,[flag,'--out',after],args.out/(kind+'-after.log'))
    if args.baseline and (args.quality_scope=='all' or kind=='status'):
        before=args.out/'quality-before'/kind
        run(args.baseline,[flag,'--out',before],args.out/(kind+'-before.log'))
        for image in before.glob('*.png'):
            report['quality'][kind+'/'+image.name]=image.read_bytes()==(after/image.name).read_bytes()
if args.baseline:
    report['gates']['pixel_exact']=bool(report['quality']) and all(report['quality'].values())
report['quality_comparison_performed']=bool(args.baseline)
(args.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report,indent=2))
if args.enforce and not all(report['gates'].values()):
    raise SystemExit('performance/quality gates failed; inspect verification.json')
