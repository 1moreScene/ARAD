# -*- coding: utf-8 -*-
"""L1 精修自验：VA 计数不变 / 剥离注释逐行比对 / 括号配平 / 头渲染抽查"""
import io, re, sys
sys.stdout.reconfigure(encoding='utf-8')

BASE = r'E:\DOF LX\【40JP逆向源代码工程】'
BAK = BASE + r'\work\_l1_nengine_backup'
EXPECT = {'log_dump': 442, 'gfx_texture': 370, 'zlib_inflate': 153, 'sound_script': 9}

def code_lines(path):
    out = []
    for l in io.open(path, encoding='utf-8').read().split('\n'):
        s = l.strip()
        if s.startswith('//'):
            continue
        out.append(l.rstrip('\r'))
    return out

ok_all = True
for f, expect in EXPECT.items():
    old = BAK + '\\' + f + '.cpp'
    new = BASE + r'\src\nengine' + '\\' + f + '.cpp'
    otext = io.open(old, encoding='utf-8').read()
    ntext = io.open(new, encoding='utf-8').read()

    # 1) VA 计数
    n_old = len(re.findall(r'^// 0x[0-9A-Fa-f]{6}', otext, re.M))
    n_new = len(re.findall(r'^// 0x[0-9A-Fa-f]{6}', ntext, re.M))
    va_ok = (n_old == n_new == expect)

    # 2) 剥离注释逐行比对
    o, n = code_lines(old), code_lines(new)
    same = (o == n)
    diff = 0
    if not same:
        for i in range(max(len(o), len(n))):
            a = o[i] if i < len(o) else '<EOF>'
            b = n[i] if i < len(n) else '<EOF>'
            if a != b:
                diff += 1
                if diff <= 3:
                    print('  DIFF line %d:\n   old: %r\n   new: %r' % (i + 1, a[:90], b[:90]))

    # 3) 括号配平
    braces_ok = (ntext.count('{') == ntext.count('}'))
    parens_ok = (ntext.count('(') == ntext.count(')'))

    # 4) 渲染残留
    residue = ('@BAR@' in ntext) or ('%%' in ntext) or ('@@N_' in ntext)
    # 5) 头部 L1 标记
    has_l1 = '【L1 精修层】' in ntext
    has_sec = '// SECTION ' in ntext

    status = 'PASS' if (va_ok and same and braces_ok and parens_ok and not residue and has_l1 and has_sec) else 'FAIL'
    ok_all &= (status == 'PASS')
    print('%s %-16s VA %d==%d==%d %s | code-lines %d vs %d identical=%s (diff=%d) | braces %s | parens %s | residue %s | L1头 %s | 分区 %s'
          % (status, f, n_old, n_new, expect, 'OK' if va_ok else 'BAD',
             len(o), len(n), same, diff, braces_ok, parens_ok, residue, has_l1, has_sec))

print('ALL:', 'PASS' if ok_all else 'FAIL')
