# -*- coding: utf-8 -*-
import idaapi, idc, ida_hexrays, ida_funcs, ida_bytes
OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\retry_445650.txt"
lines = []
va = 0x445650
f = ida_funcs.get_func(va)
lines.append("func: %s size=%d" % ("OK" if f else "NONE", f.size() if f else 0))
lines.append("=== DISASM ===")
ea = va
end = f.end_ea if f else va + 112
guard = 0
while ea < end and guard < 300:
    lines.append("%08X  %s" % (ea, idc.GetDisasm(ea)))
    nxt = ida_bytes.next_head(ea, end)
    ea = nxt if nxt > ea else ea + 1
    guard += 1
lines.append("")
lines.append("=== DECOMPILE ===")
try:
    cf = ida_hexrays.decompile(va)
    lines.append(str(cf) if cf else "(failed: None)")
except Exception as e:
    lines.append("(exception %s)" % e)
open(OUT, "w", encoding="utf-8").write("\n".join(lines))
idc.qexit(0)
