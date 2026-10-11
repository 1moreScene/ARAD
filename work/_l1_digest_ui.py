# -*- coding: utf-8 -*-
"""L1 精修辅助：为 src/game/ui/*.cpp 生成逐函数摘要（VA/行数/字符串/被调/事件TAG）。
输出 work/_ui_digest/<file>.digest.txt，供人工审读后构建注释升级映射。不改源文件。"""
import re, os, sys, io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(BASE, 'src', 'game', 'ui')
OUT = os.path.join(BASE, 'work', '_ui_digest')
os.makedirs(OUT, exist_ok=True)

VA_RE = re.compile(r'^// (0x[0-9A-Fa-f]{6}) ([A-Za-z_][A-Za-z0-9_:]*)\s*$')
STR_RE = re.compile(r'"((?:[^"\\]|\\.)*)"')
CALL_RE = re.compile(r'\b(sub_[0-9A-Fa-f]{6})\s*\(')
VT_RE = re.compile(r'VT_CALL|\(\* \(int \(/\*thiscall\*/ \*\*\)\(void \*\*\)\(\*\(int\*\)this \+ (0x[0-9A-Fa-f]+)\)\)')
EVT_RE = re.compile(r'"((?:CLICK_BUTTON|MOUSE_OVER|MOUSE_OUT|MOUSE_LBUTTONDOWN|MOUSE_LBUTTONUP|MOUSE_MOVE|MOUSE_WHEEL|KEY_[A-Z_]+|CHARSET_[A-Z_]+|UPDATE|CREATE|DESTROY|DRAW[A-Z_]*|CLOSE[A-Z_]*|[A-Z][A-Z0-9_]{2,}))"')

def analyze(path):
    with open(path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    funcs = []  # (va, name, start_idx, end_idx)
    for i, ln in enumerate(lines):
        m = VA_RE.match(ln)
        if m:
            if funcs:
                funcs[-1] = funcs[-1][:3] + (i,)
            funcs.append((m.group(1), m.group(2), i, len(lines)))
    out = []
    for va, name, s, e in funcs:
        body = lines[s:e]
        nline = e - s
        strs = []
        for ln in body:
            for m in STR_RE.finditer(ln):
                t = m.group(1)
                if len(t) >= 3 and t not in strs:
                    strs.append(t)
        callees = []
        for ln in body:
            for m in CALL_RE.finditer(ln):
                c = m.group(1)
                if c != name and c not in callees:
                    callees.append(c)
        # vtable slot offsets
        vts = set()
        for ln in body:
            for m in re.finditer(r'\*\(int \*\)this \+ (0x[0-9A-Fa-f]+)\)', ln):
                vts.add(m.group(1))
            for m in re.finditer(r'\*\(int \*\)\(?\(?[A-Za-z_0-9]+ \+ (0x[0-9A-Fa-f]+)\)', ln):
                pass
        # event-ish tags (UPPER CASE strings)
        evts = sorted(set(t for t in strs if re.match(r'^[A-Z][A-Z0-9_]{2,}$', t)))
        interesting = [t for t in strs if re.search(r'\.(img|etc|ani|ptl|nif|tbl|url|com|php|asp)', t) or 'http' in t.lower()]
        out.append((va, name, nline, interesting, evts[:12], callees[:24], vts))
    return out

def main():
    files = sys.argv[1:] or ['cashshop.cpp','window_party.cpp','trade_shop.cpp','aura_create.cpp','http_item_info.cpp','char_server_info.cpp']
    for fn in files:
        p = os.path.join(SRC, fn)
        res = analyze(p)
        with open(os.path.join(OUT, fn + '.digest.txt'), 'w', encoding='utf-8') as w:
            w.write('# digest of %s  funcs=%d\n' % (fn, len(res)))
            w.write('# 格式: VA|行数|资源串|TAG串|被调|vt槽\n')
            for va, name, nline, interesting, evts, callees, vts in res:
                w.write('%s %s | %d\n' % (va, name, nline))
                if interesting: w.write('   RES: %s\n' % ' ; '.join(interesting[:14]))
                if evts:        w.write('   TAG: %s\n' % ' '.join(evts))
                if callees:     w.write('   CALL: %s\n' % ' '.join(callees))
                if vts:         w.write('   VT: %s\n' % ' '.join(sorted(vts, key=lambda x:int(x,16))))
        print(fn, len(res), 'funcs ->', os.path.join(OUT, fn + '.digest.txt'))

if __name__ == '__main__':
    main()
