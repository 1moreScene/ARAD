"""Extract original raw IMG UI frames and save primary PE evidence; no client writes."""
import hashlib
import json
import struct
import sys
from pathlib import Path
from PIL import Image, ImageDraw
from npk_reader import NpkReader
import movement_evidence as pe

root = Path(__file__).resolve().parents[1]
hud = '--hud' in sys.argv
out = root / ('work/hud_evidence' if hud else 'work/ui_evidence')
out.mkdir(parents=True, exist_ok=True)
names = ['sprite/interface/statuswindow.img', 'sprite/interface/statusillust.img',
         'sprite/interface/iteminventorywindow.img', 'sprite/interface/skillbuywindow.img',
         'sprite/common/etc/combo.img', 'sprite/item/weapon/swordman/sswd.img']
if hud:
    names = ['sprite/interface/base.img','sprite/interface/skillinventorywindow.img',
             'sprite/interface/popup.img','sprite/item/attachicon.img','sprite/common/etc/cooltimeend.img']
report = {'exe_sha256': hashlib.sha256(pe.blob).hexdigest(), 'assets': {}}
for path in (root / '40jp-client/40jp单机版/ImagePacks2').iterdir():
    if path.suffix.lower() != '.npk':
        continue
    pack = NpkReader(path)
    for _, _, name in pack.entries:
        if name.lower() not in names:
            continue
        raw = pack.read(name)
        count = struct.unpack_from('<I', raw, 28)[0]
        frames = []
        metadata = []
        off = 32
        for i in range(count):
            tag, = struct.unpack_from('<I', raw, off)
            off += 4
            if tag == 17:
                link, = struct.unpack_from('<I', raw, off)
                off += 4
                frames.append(frames[link].copy())
                metadata.append({'index': i, 'link': link})
                continue
            sub, w, h, size, x, y, cw, ch = struct.unpack_from('<IIIIiiII', raw, off)
            off += 32
            assert tag in (14, 15, 16), tag
            if tag == 16:
                rgba = bytes(c for p in range(w*h) for c in
                             (raw[off+4*p+2], raw[off+4*p+1], raw[off+4*p], raw[off+4*p+3]))
                off += w*h*4
            else:
                pixels = struct.unpack_from('<' + 'H' * (w*h), raw, off)
                off += w*h*2
                rgba = bytes(c for p in pixels for c in
                             (((((p >> 10) & 31) << 3) | (((p >> 10) & 31) >> 2),
                               (((p >> 5) & 31) << 3) | (((p >> 5) & 31) >> 2),
                               ((p & 31) << 3) | ((p & 31) >> 2), 255 if p & 32768 else 0) if tag == 14 else
                              (((p >> 8) & 15)*17, ((p >> 4) & 15)*17, (p & 15)*17, (p >> 12)*17)))
            frames.append(Image.frombytes('RGBA', (w, h), rgba))
            metadata.append({'index': i, 'tag': tag, 'width': w, 'height': h, 'x': x, 'y': y,
                             'canvas': [cw, ch]})
        assert off == len(raw)
        stem = Path(name).stem
        cell_w = max(max(f.width for f in frames)+8, 60)
        cell_h = max(f.height for f in frames)+22
        cols = 6 if cell_w < 140 else 3
        sheet = Image.new('RGBA', (cols*cell_w, ((count+cols-1)//cols)*cell_h), '#303038')
        draw = ImageDraw.Draw(sheet)
        for i, frame in enumerate(frames):
            x, y = (i % cols)*cell_w, (i // cols)*cell_h
            draw.text((x+4, y+2), str(i), fill='white')
            sheet.alpha_composite(frame, (x+4, y+18))
            if hud or i == 0 or stem == 'combo' and (44 <= i < 98 or 143 <= i < 153):
                frame.save(out / f'{stem}-{i}.png')
        sheet.save(out / f'{stem}-sheet.png')
        report['assets'][name] = {'pack': path.name, 'sha256': hashlib.sha256(raw).hexdigest(),
                                 'frames': metadata}
if hud:
    from pvf_reader import PvfReader
    pvf = PvfReader(str(root/'40jp-client/40jp单机版/Script.pvf'))
    for path in ['string.lst','skill/skill.jap.str','skill/swordman/hardattack.skl','skill/swordman/upperslash.skl']:
        (out/path.replace('/','__')).write_text(pvf.read(path).decode('cp932',errors='replace'),encoding='utf-8')
for label, start, end in [('registry', 0x41F2C0, 0x41F690),
                           ('damage_numbers', 0x4D7960, 0x4D7C60),
                           ('damage_call_flags', 0x448C50, 0x448D70),
                           ('status_window', 0x54B8B0, 0x54B9F0),
                           ('interpolation', 0x4919D0, 0x491C70)]:
    lines = [f'{i.address:08X} {i.mnemonic:8} {i.op_str}'
             for i in pe.md.disasm(pe.read(start, end-start), start)]
    (out / f'{label}.txt').write_text('\n'.join(lines), encoding='utf-8')
(out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
if hud:
    for label,start,end in [('hud',0x469170,0x469DD0),('slot_positions',0x469DD0,0x46ABB0),
        ('drag',0x4633D0,0x463C90),('gauges',0x460920,0x460DC0),('skill_draw',0x47DF50,0x47E170),
        ('cooldown',0x4F2E80,0x4F2F70),('cooldown_resources',0x4F3960,0x4F3B10),
        ('skill_tooltip',0x570FD0,0x571AF0),('popup_skin',0x554510,0x5547D0),
        ('level_property_format',0x47F310,0x47F6B0)]:
        (out/f'{label}.txt').write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}'
            for i in pe.md.disasm(pe.read(start,end-start),start)),encoding='utf-8')
print({name: len(entry['frames']) for name, entry in report['assets'].items()})
