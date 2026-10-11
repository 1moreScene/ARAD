import idaapi, ida_bytes, idc
OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\recon_527AC0_disasm.txt"
lines = []
ea = 0x527AC0
end = 0x527BE0
while ea < end:
    lines.append("%08X  %s" % (ea, idc.GetDisasm(ea)))
    nxt = ida_bytes.next_head(ea, end)
    ea = nxt if nxt > ea else ea + 1
open(OUT, "w", encoding="utf-8").write("\n".join(lines))
import idc as _
idc.qexit(0)
