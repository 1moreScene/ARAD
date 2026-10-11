# -*- coding: utf-8 -*-
# 全量补 dump: 不截断, 输出 0x4F0000-0x510000 全部函数
import idaapi, idautils, idc, ida_bytes, ida_funcs, ida_hexrays, ida_name
import os

OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\dump_parsers_full.txt"
PROG = r"E:\DOF LX\【40JP逆向源代码工程】\work\full_progress.log"

def plog(s):
    with open(PROG, "a", encoding="utf-8") as f:
        f.write(s + "\n")

LO, HI = 0x4F0000, 0x510000
seen = set()
funcs = []
ea = LO
while ea < HI:
    f = ida_funcs.get_func(ea)
    if f:
        if f.start_ea not in seen:
            seen.add(f.start_ea)
            funcs.append(f.start_ea)
        ea = f.end_ea
    else:
        ea += 16

plog("funcs=%d" % len(funcs))
lines = ["### BATCH DECOMPILE FULL 0x4F0000-0x510000", "### funcs=%d" % len(funcs)]
for idx, fea in enumerate(sorted(funcs)):
    if idx % 25 == 0:
        plog("part %d/%d" % (idx, len(funcs)))
    nm = ida_name.get_ea_name(fea) or ""
    lines.append("")
    lines.append("// ============================================================")
    lines.append("// %08X  %s" % (fea, nm))
    try:
        cf = ida_hexrays.decompile(fea)
        if cf:
            lines.extend(str(cf).split("\n"))
        else:
            lines.append("// (decomp failed)")
    except Exception as e:
        lines.append("// (exception %s)" % str(e)[:80])
    if idx % 50 == 49:
        open(OUT, "w", encoding="utf-8").write("\n".join(lines))

open(OUT, "w", encoding="utf-8").write("\n".join(lines))
plog("DONE %d" % len(funcs))
idc.qexit(0)
