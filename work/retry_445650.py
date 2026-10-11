# -*- coding: utf-8 -*-
import idaapi, idc, ida_hexrays, ida_funcs, ida_bytes, ida_ua
OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\retry_445650.txt"
lines = []
va = 0x445650
f = ida_funcs.get_func(va)
lines.append("func: %s size=%d" % ("OK" if f else "NONE", f.size() if f else 0))
# 反汇编
lines.append("=== DISASM ===")
ea = va
end = f.end_ea if f else va + 112
while ea < end:
    lines.append("%08X  %s" % (ea, ida_ua.generate_disasm_line(ea, 0) or "?"))
    ea = ida_bytes.next_head(ea, end)
# 反编译
lines.append("")
lines.append("=== DECOMPILE ===")
try:
    cf = ida_hexrays.decompile(va)
    if cf:
        lines.append(str(cf))
    else:
        lines.append("(failed) hexrays returned None")
except Exception as e:
    lines.append("(exception %s)" % e)
open(OUT, "w", encoding="utf-8").write("\n".join(lines))
idc.qexit(0)
