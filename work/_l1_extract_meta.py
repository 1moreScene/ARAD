# -*- coding: utf-8 -*-
"""L1 精修辅助：提取逐函数元数据（字符串引用 / 调用 / 行数），产出精修证据表。"""
import re, io, sys
sys.stdout.reconfigure(encoding='utf-8')

BASE = r'E:\DOF LX\【40JP逆向源代码工程】'

def analyze(path, out):
    lines = io.open(path, encoding='utf-8', errors='replace').read().splitlines()
    funcs = []
    for i, l in enumerate(lines):
        m = re.match(r'^// 0x([0-9A-Fa-f]{6,})\s+(.*)$', l)
        if m:
            funcs.append((i, m.group(1).upper(), m.group(2).strip()))
    report = []
    for idx, (i, va, name) in enumerate(funcs):
        end = funcs[idx + 1][0] if idx + 1 < len(funcs) else len(lines)
        body = '\n'.join(lines[i:end])
        strs = re.findall(r'"([^"\n]{3,90})"', body)
        seen = set(); ss = []
        for s in strs:
            if s not in seen:
                seen.add(s); ss.append(s)
        calls = re.findall(r'\b((?:j__)?(?:sub_[0-9A-Fa-f]{6}|[A-Za-z_][A-Za-z0-9_:<>~]*))\s*\(', body)
        drop = ('if','for','while','switch','return','sizeof','uint','int','float','double',
                'char','void','unsigned','signed','qmemcpy','memcmp','memset','long','short')
        seen = set(); cc = []
        for c in calls:
            if c.startswith(drop) or c[0].isdigit():
                continue
            if c not in seen:
                seen.add(c); cc.append(c)
        nlines = end - i
        report.append((va, name, nlines, ss, cc))
    with io.open(out, 'w', encoding='utf-8') as f:
        for va, name, n, ss, cc in report:
            f.write('0x%s %s [%dL]\n' % (va, name, n))
            if ss: f.write('  STR: ' + ' | '.join(ss[:8]) + '\n')
            if cc: f.write('  CALL: ' + ', '.join(cc[:14]) + '\n')
    print(out, len(report), 'functions')

analyze(BASE + r'\src\nengine\log_dump.cpp',     BASE + r'\work\_l1_meta_log.txt')
analyze(BASE + r'\src\nengine\gfx_texture.cpp', BASE + r'\work\_l1_meta_gfx.txt')
analyze(BASE + r'\src\nengine\zlib_inflate.cpp',BASE + r'\work\_l1_meta_zlib.txt')
analyze(BASE + r'\src\nengine\sound_script.cpp',BASE + r'\work\_l1_meta_snd.txt')
