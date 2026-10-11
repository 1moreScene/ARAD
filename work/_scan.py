import re, sys
src = open(r'E:\DOF LX\【40JP逆向源代码工程】\work\dump_parsers.txt', encoding='utf-8', errors='replace').read().splitlines()
starts = [(i, l) for i, l in enumerate(src) if re.match(r'^// [0-9A-F]{8}  ', l)]
str_re = re.compile(r'"([^"]{1,64})"')
out = []
for idx, (i, l) in enumerate(starts):
    end = starts[idx + 1][0] if idx + 1 < len(starts) else len(src)
    body = src[i:end]
    strs = []
    for line in body:
        for m in str_re.finditer(line):
            t = m.group(1)
            if t not in strs:
                strs.append(t)
    va = l.split()[1]
    name = l.split()[2] if len(l.split()) > 2 else ''
    if strs:
        out.append(va + ' ' + name + ' | ' + ' / '.join(strs[:12]))
open(r'E:\DOF LX\【40JP逆向源代码工程】\work\strmap.txt', 'w', encoding='utf-8').write('\n'.join(out))
print(len(out), 'funcs with strings')
