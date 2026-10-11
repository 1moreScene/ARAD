"""Verify official sound decoding, real device playback and native action integration."""
import hashlib
import json
import re
import struct
import subprocess
import wave
from collections import Counter
from pathlib import Path

root = Path(__file__).resolve().parents[1]
cwd = root / "custom-dnf"
exe = cwd / "target/movement-verified/release/dnf-app.exe"
out = cwd / "out/audio-verified"
out.mkdir(parents=True, exist_ok=True)

def run(name, *args):
    result = subprocess.run([str(exe), *map(str, args)], cwd=cwd, capture_output=True,
                            text=True, encoding="utf-8", errors="replace", timeout=60)
    log = result.stdout + result.stderr
    (out / f"{name}.txt").write_text(log, encoding="utf-8")
    assert result.returncode == 0, log
    return log

def inspect(path, audible=True):
    with wave.open(str(path), "rb") as stream:
        assert (stream.getnchannels(), stream.getframerate(), stream.getsampwidth()) == (2, 44100, 2)
        frames = stream.getnframes()
        raw = stream.readframes(frames)
    samples = struct.unpack(f"<{len(raw)//2}h", raw)
    peak = max(map(abs, samples))
    assert (peak > 0) == audible
    rail_samples = sum(x in (-32768, 32767) for x in samples)
    assert rail_samples == 0, f"hard clipping in {path}: {rail_samples} samples"
    return {"frames": frames, "peak": peak,
            "rail_samples": rail_samples,
            "nonzero_samples": sum(x != 0 for x in samples),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}

device = run("device", "--audio-selftest", "--audio-out", out / "overlap.wav")
assert "音频自检通过" in device
assert "audio focus_stop_resume=passed" in device
stats = dict(zip(["requests", "submitted", "completed", "audible_completed", "peak_voices", "dropped"],
                 map(int, re.findall(r"audio requests=(\d+) submitted=(\d+) completed=(\d+) audible_completed=(\d+) peak_voices=(\d+) dropped=(\d+)", device)[-1])))
assert stats["audible_completed"] > 0 and stats["peak_voices"] >= 2 and stats["dropped"] == 0
stats["underruns"] = int(re.findall(r"underruns=(\d+)",device)[-1])
assert stats["underruns"] == 0, stats
report = {"device": stats, "overlap": inspect(out / "overlap.wav")}
report["focus_stop_resume"] = True
for scenario, expected in [
    ("attack", {"R_SM_ATK": 3, "R_MINERALSWDA": 2, "MINERALSWDB": 1}),
    ("airattack", {"JUMP_START": 1, "SM_JUMP": 1, "R_SM_JUMP_ATK": 1, "R_MINERALSWDA": 1, "JUMP_END": 1}),
]:
    log = run(scenario, "--headless", "--script", scenario, "--frames", 120, "--trace",
              "--out", out / scenario, "--audio-out", out / f"{scenario}.wav")
    tags = Counter(tag for group in re.findall(r"audio_tags=(\[.*?\])", log) for tag in json.loads(group))
    assert {tag:tags[tag] for tag in expected} == expected, tags
    audio = inspect(out / f"{scenario}.wav")
    assert audio["frames"] == 88200
    native = run(f"{scenario}-window", "--window-replay", "--script", scenario,
                 "--trace", "--shot-after", 120, "--shot-out", out / f"{scenario}-window.png")
    native_tags = Counter(tag for group in re.findall(r"audio_tags=(\[.*?\])", native) for tag in json.loads(group))
    assert {tag:native_tags[tag] for tag in expected} == expected, native_tags
    assert "音效已启用" in native and "窗口实拍已保存" in native
    match = re.search(r"audio requests=(\d+).*audible_completed=(\d+).*peak_voices=(\d+).*dropped=(\d+)", native)
    assert match, native
    requests, completed, overlap, dropped = map(int, match.groups())
    assert requests == sum(native_tags.values()) and completed > 0 and overlap >= 2 and dropped == 0
    report[scenario] = {"events": dict(tags), "export": audio, "native_requests": requests,
                        "native_audible_completed": completed, "native_peak_voices": overlap,
                        "native_underruns": int(re.findall(r"underruns=(\d+)",native)[-1])}
    assert report[scenario]["native_underruns"] == 0, report[scenario]

run("repeat", "--headless", "--script", "attack", "--frames", 120,
    "--out", out / "repeat", "--audio-out", out / "repeat.wav")
assert (out / "repeat.wav").read_bytes() == (out / "attack.wav").read_bytes()
run("mute-export", "--headless", "--script", "attack", "--frames", 120, "--mute",
    "--out", out / "mute", "--audio-out", out / "mute.wav")
report["mute"] = inspect(out / "mute.wav", False)
mute = run("mute-window", "--window-replay", "--script", "attack", "--mute",
           "--shot-after", 25, "--shot-out", out / "mute-window.png")
assert "音效已启用" not in mute and "audio requests=" not in mute
report["deterministic_export"] = True
report["mute_opens_no_device"] = True
(out / "verification.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps(report, ensure_ascii=False, indent=2))
