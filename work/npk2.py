# -*- coding: utf-8 -*-
# 第二轮: PVF 魔法串/解密函数 + NPK 数据读取函数
import idaapi, idautils, idc, ida_bytes, ida_funcs, ida_hexrays

OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\recon_npk_pvf2.txt"
lines = []
def log(s): lines.append(s)

# 1. off_7930CC / off_7930D0 的内容
log("### off_7930CC / off_7930D0 (PVF 魔法串?)")
for name, addr in [("off_7930CC", 0x7930CC), ("off_7930D0", 0x7930D0)]:
    p = ida_bytes.get_dword(addr)
    log("%s -> ptr %08X" % (name, p))
    if 0x400000 <= p < 0x9D6000:
        s = ida_bytes.get_strlit_contents(p, -1, 0)
        log("  str: %r" % (s,))

def dump_func(ea, tag, maxlines=250):
    log("")
    log("// ==================== %s @ %08X ====================" % (tag, ea))
    try:
        cf = ida_hexrays.decompile(ea)
        lines.extend((str(cf).split("\n") if cf else ["// failed"])[:maxlines])
    except Exception as e:
        log("// (exc %s)" % str(e)[:80])

# 2. PVF 解密与文件表函数
for ea, tag in [(0x527AC0, "PVF decrypt sub_527AC0"),
                (0x528D50, "PVF sub_528D50"),
                (0x527DB0, "PVF sub_527DB0")]:
    dump_func(ea, tag)

# 3. NPK: 登记函数/缓存读取/按名取数据
for ea, tag in [(0x40DB60, "NPK sub_40DB60 (登记)"),
                (0x40E700, "NPK sub_40E700 (加入列表)"),
                (0x40DED0, "NPK sub_40DED0 (缓存命中路径)"),
                (0x40DB60, "NPK dup")]:
    dump_func(ea, tag)

# 4. NPK 按名取数据的函数: 找引用 sub_40E9E0 所在类的调用者
log("")
log("### NPK 按名取数据: 0x40E3E0 与 0x40E9E0 的调用者")
for target in (0x40E3E0, 0x40E9E0, 0x40DED0):
    log("-- callers of %08X --" % target)
    for x in idautils.XrefsTo(target):
        f = ida_funcs.get_func(x.frm)
        if f and f.start_ea != target:
            log("  %08X in %08X" % (x.frm, f.start_ea))

# 5. PVF 主加载器 0x4422C0 的调用者 + 0x4422C0 本体
log("")
log("### PVF 主加载器 0x4422C0")
dump_func(0x4422C0, "PVF loader main sub_4422C0")
log("-- callers of 0x4422C0 --")
for x in idautils.XrefsTo(0x4422C0):
    f = ida_funcs.get_func(x.frm)
    if f: log("  %08X in %08X" % (x.frm, f.start_ea))

open(OUT, "w", encoding="utf-8").write("\n".join(lines))
import idc
idc.qexit(0)
