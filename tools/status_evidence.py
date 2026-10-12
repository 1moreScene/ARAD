"""Read-only 40JP PVF/PE evidence for armor, invulnerability and outlines."""
import hashlib
import os
import json
import re
import struct
from pathlib import Path
from pvf_reader import PvfReader, decrypt_blob

ROOT = Path(__file__).resolve().parents[1]
CLIENT = Path(os.environ.get('DNF_CLIENT_DIR', ROOT / '40jp-client/Arad40'))
OUT = ROOT / 'work/status_evidence'
OUT.mkdir(parents=True, exist_ok=True)
pvf = PvfReader(str(CLIENT / 'Script.pvf'))
report = {'pvf_sha256': hashlib.sha256(pvf.data).hexdigest(), 'scripts': {}, 'damage_types': {}}
for _, offset, size, key, path in pvf.entries:
    seek = offset + 27 + pvf.header['S']
    raw, _ = decrypt_blob(pvf.data[seek:seek + ((size + 3) & ~3)], offset, key)
    raw = raw[:size]
    types = re.findall(rb'\[DAMAGE TYPE\]\s*`?([A-Za-z]+)', raw, re.I)
    for kind in set(types):
        label = kind.decode().upper()
        report['damage_types'].setdefault(label, []).append(path)
    interesting = ('superarmor' in path or 'invins' in path or 'invinc' in path
                   or path in ['character/swordman/animation/attack3.ani',
                               'character/swordman/animation/jump.ani',
                               'character/swordman/animation/hardattack.ani',
                               'monster/spider/spiderbossani/burrowstart.ani'])
    if interesting:
        (OUT / path.replace('/', '__')).write_text(raw.decode('cp932', errors='replace'), encoding='utf-8')
        report['scripts'][path] = {'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
print('damage types:', {k: len(v) for k, v in report['damage_types'].items()})
report['damage_type_entry_counts'] = {k: len(v) for k, v in report['damage_types'].items()}
# Preserve non-normal examples; don't fill the report with 12,774 unrelated paths.
report['damage_types']['NORMAL'] = report['damage_types']['NORMAL'][:3]
(OUT / 'pvf.json').write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding='utf-8')
print('selected scripts:', '\n'.join(report['scripts']))

try:
    import capstone
except ImportError:
    print('capstone unavailable; PVF evidence saved')
else:
    blob = (CLIENT / 'ARAD.exe').read_bytes()
    header = struct.unpack_from('<I', blob, 0x3C)[0]
    base = struct.unpack_from('<I', blob, header + 52)[0]
    sections = []
    for i in range(struct.unpack_from('<H', blob, header + 6)[0]):
        pos = header + 24 + struct.unpack_from('<H', blob, header + 20)[0] + 40*i
        _, rva, size, offset = struct.unpack_from('<IIII', blob, pos + 8)
        sections.append((base + rva, size, offset))
    def read(address, count):
        for start, size, offset in sections:
            if start <= address and address + count <= start + size:
                return blob[offset + address-start:offset + address-start + count]
        raise ValueError(hex(address))
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    for name, start, end in [('armor_appendage', 0x605920, 0x6059E0),
                             ('outline_setter', 0x42A8D0, 0x42A930),
                             ('armor_gate', 0x4D5B30, 0x4D5C10),
                             ('frame_type_query', 0x4A2C40, 0x4A2C70),
                             ('frame_apply', 0x493400, 0x493480),
                             ('frame_copy', 0x493120, 0x4931A0),
                             ('hurt_gate', 0x4A2BB0, 0x4A2C40),
                             ('frame_override', 0x4E5C10, 0x4E5CE0),
                             ('outline_render', 0x499C70, 0x499FF0),
                             ('outline_color', 0x4917A0, 0x4918E0),
                             ('upper_armor', 0x5F58C0, 0x5F5A10),
                             ('upper_tick', 0x5F10C0, 0x5F1600),
                             ('frame_sync', 0x4919D0, 0x491D00)]:
        (OUT / (name + '.txt')).write_text('\n'.join(f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in md.disasm(read(start, end-start), start)), encoding='utf-8')
    (OUT / 'exe.json').write_text(json.dumps({'sha256': hashlib.sha256(blob).hexdigest()}, indent=2), encoding='utf-8')
    instructions = list(md.disasm(read(0x5F39F0, 0x5F6000-0x5F39F0), 0x5F39F0))
    excerpts = []
    for i, instruction in enumerate(instructions):
        if instruction.mnemonic == 'call' and '0x264' in instruction.op_str:
            excerpts.extend(instructions[max(0, i-8):i+3])
    (OUT / 'swordman_frame_overrides.txt').write_text('\n'.join(
        f'{i.address:08X} {i.mnemonic:8} {i.op_str}' for i in excerpts), encoding='utf-8')
