import idaapi, idautils, idc, ida_bytes, ida_hexrays
OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\recon_pvf_content.txt"
lines = []
def log(s): lines.append(s)
def dump(ea, tag, maxl=300):
    log("// ===== %s @ %08X =====" % (tag, ea))
    try:
        cf = ida_hexrays.decompile(ea)
        lines.extend((str(cf).split("\n") if cf else ["// failed"])[:maxl])
    except Exception as e:
        log("// exc %s" % e)
    log("")
dump(0x528650, "PVF 内容读取+解密 sub_528650")
dump(0x528300, "PVF sub_528300 (相邻，可能是内容定位)")
# 内容解密如果是 XOR 类，找 0x528650 调用的关键函数
f = ida_funcs.get_func(0x528650)
ea = f.start_ea
callees = set()
while ea < f.end_ea:
    for x in idautils.XrefsFrom(ea):
        if x.type in (idaapi.fl_CN, idaapi.fl_CF) and 0x400000 <= x.to < 0x6C1000:
            callees.add(x.to)
    ea = ida_bytes.next_head(ea, f.end_ea)
log("sub_528650 被调: %s" % ", ".join("%08X" % c for c in sorted(callees)))
open(OUT, "w", encoding="utf-8").write("\n".join(lines))
idc.qexit(0)
