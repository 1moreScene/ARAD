"""Native background progression and command-manager replay; no global input injection."""
import ctypes as c
import json,re,subprocess,time
from pathlib import Path
root=Path(__file__).resolve().parents[1];cwd=root/'custom-dnf';out=cwd/'out/key-manager-verified';out.mkdir(parents=True,exist_ok=True)
exe=cwd/'target/movement-verified/release/dnf-app.exe'
u=c.WinDLL('user32',use_last_error=True)
callback=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_ssize_t)
u.EnumWindows.argtypes=[callback,c.c_ssize_t];u.GetWindowThreadProcessId.argtypes=[c.c_void_p,c.POINTER(c.c_uint32)]
u.ShowWindow.argtypes=[c.c_void_p,c.c_int];u.SetForegroundWindow.argtypes=[c.c_void_p];u.GetForegroundWindow.restype=c.c_void_p
u.SendMessageW.argtypes=[c.c_void_p,c.c_uint32,c.c_size_t,c.c_ssize_t];u.SendMessageW.restype=c.c_ssize_t
u.GetWindowThreadProcessId.restype=c.c_uint32;u.AttachThreadInput.argtypes=[c.c_uint32,c.c_uint32,c.c_int]
k=c.WinDLL('kernel32');k.GetCurrentThreadId.restype=c.c_uint32
report={}
device=subprocess.run([str(exe),'--mute','--shot-after','20','--shot-out',str(out/'device.png')],cwd=cwd,
                      capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=30)
device_log=device.stdout+device.stderr;(out/'device.txt').write_text(device_log,encoding='utf-8')
assert device.returncode==0 and '键盘后端: DirectInput8' in device_log,device_log
sampler=re.search(r'keyboard samples=(\d+) failed=(\d+)',device_log)
assert sampler and int(sampler[1])>=15 and int(sampler[2])==0
report['device']={'backend':'DirectInput8','data_bytes':256,'samples':int(sampler[1]),'failures':0}
for scenario in ['jump','upper']:
    logfile=out/f'{scenario}-background.txt'
    with logfile.open('wb') as stream:
        process=subprocess.Popen([str(exe),'--window-replay','--show-keys','--script',scenario,'--trace',
                                  '--shot-after','210','--shot-out',str(out/f'{scenario}-window.png')],cwd=cwd,
                                  stdout=stream,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
        windows=[]
        @callback
        def find(hwnd,_):
            pid=c.c_uint32();u.GetWindowThreadProcessId(hwnd,c.byref(pid))
            if pid.value==process.pid:windows.append(hwnd)
            return 1
        def lines():return [s for s in logfile.read_text(encoding='utf-8',errors='replace').splitlines() if s.startswith('step=')]
        deadline=time.monotonic()+8
        while not windows and time.monotonic()<deadline:
            u.EnumWindows(find,0);time.sleep(.01)
        assert windows,'native window missing'
        hwnd=windows[0]
        while time.monotonic()<deadline:
            states=lines()
            zname='z=' if scenario=='jump' else 'goblin_z='
            if states and float(re.search(r'(?<!\w)'+zname+r'\s*([\d.]+)',states[-1]).group(1))>5:break
            time.sleep(.01)
        else:raise AssertionError('fixture did not become airborne')
        u.SendMessageW(hwnd,0x104,0x79,1);u.SendMessageW(hwnd,0x105,0x79,0) # Own F10 handler unpins.
        u.ShowWindow(hwnd,6) # Minimize only our fixture. Its physics must keep running.
        deadline=time.monotonic()+5
        while time.monotonic()<deadline:
            states=lines();inactive=[s for s in states if 'input_active=false' in s]
            if len(inactive)>=125:break
            assert process.poll() is None,'fixture exited before background check'
            time.sleep(.01)
        else:raise AssertionError('game did not advance at least 125 ticks while unfocused')
        assert not any('audio_tags=["JUMP_START"' in s or 'audio_tags=["SM_UPPER_SLASH"]' in s for s in inactive)
        assert all('key_records=0' in s for s in inactive),'background command input leaked'
        if scenario=='jump':assert any('phase=Ground' in s and re.search(r'(?<!\w)z=\s*0.0',s) for s in inactive)
        else:
            assert any('upper_cd=0 ' in s for s in inactive),'cooldown froze'
            assert any('goblin_motion=Sit' in s for s in inactive),'reaction froze'
            assert all('mp=144 ' in s for s in inactive),'background Z was accepted'
        clocks=[int(re.search(r'key_clock=(\d+)',s).group(1)) for s in inactive]
        assert clocks[-1]-clocks[0]>2000
        u.ShowWindow(hwnd,9);u.SetForegroundWindow(hwnd)
        if u.GetForegroundWindow()!=hwnd:
            foreground=u.GetForegroundWindow();thread=u.GetWindowThreadProcessId(foreground,None);own=k.GetCurrentThreadId()
            if thread and thread!=own:
                u.AttachThreadInput(own,thread,1)
                try:u.SetForegroundWindow(hwnd)
                finally:u.AttachThreadInput(own,thread,0)
        process.wait(timeout=8)
    assert process.returncode==0
    log=logfile.read_text(encoding='utf-8',errors='replace')
    assert '窗口实拍已保存' in log and 'underruns=0' in log
    assert any('input_active=true' in s for s in lines()[len(states):]),'foreground input did not recover'
    report[scenario]={'inactive_ticks':len(inactive),'background_clock_ms':clocks[-1]-clocks[0],
                      'background_input_ignored':True,'landing_or_recovery_finished':True,'foreground_recovered':True}
for scenario in ['mashtap','run','attack']:
    p=subprocess.run([str(exe),'--headless','--show-keys','--script',scenario,'--frames','120','--trace','--out',str(out/scenario)],
                     cwd=cwd,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=30)
    log=p.stdout+p.stderr;(out/f'{scenario}.txt').write_text(log,encoding='utf-8');assert p.returncode==0,log
    assert 'key_records=' in log and 'key_clock=2000' in log
    if scenario=='mashtap':assert 'motion=RUN' not in log
    elif scenario=='run':assert 'motion=RUN' in log
    else:assert 'goblin_hp=246 hits=3' in log
    report[scenario]={'passed':True}
(out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report,indent=2))
