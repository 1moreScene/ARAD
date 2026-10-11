# -*- coding: utf-8 -*-
# IMG 内容解析器逆向: 0x418CD0 后的详细解析 + 像素读取
import idaapi, idautils, idc, ida_bytes, ida_hexrays, ida_funcs

OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\recon_img_parser.txt"
lines = []
def log(s): lines.append(s)
def dump(ea, tag, maxl=350):
    log("// ===== %s @ %08X =====" % (tag, ea))
    try:
        cf = ida_hexrays.decompile(ea)
        lines.extend((str(cf).split("\n") if cf else ["// failed"])[:maxl])
    except Exception as e:
        log("// exc %s" % e)
    log("")

# 1. 0x418CD0 的调用者（谁打开 IMG 后继续解析内容）
log("### 0x418CD0 的调用者")
for x in idautils.XrefsTo(0x418CD0):
    f = ida_funcs.get_func(x.frm)
    if f: log("  %08X in %08X" % (x.frm, f.start_ea))

# 2. 图像数据读取函数: 找 0x419550（之前 recon 发现引用 8F7B08 的函数）
log("")
dump(0x419550, "IMG 图像数据读取 sub_419550")
dump(0x418E20, "IMG sub_418E20 (延迟加载)")

# 3. 查找 0x418CD0 附近的其他 IMG 函数（0x419xxx 区间）
log("### 0x419xxx-0x41Axxx 函数列表")
ea = 0x419000
while ea < 0x41A800:
    f = ida_funcs.get_func(ea)
    if f:
        log("  %08X size=%d" % (f.start_ea, f.size()))
        ea = f.end_ea
    else:
        ea += 4

# 4. 反编译几个关键的
for fea, tag in [(0x418E20, "IMG lazy load"), (0x419550, "IMG data read")]:
    dump(fea, tag)

open(OUT, "w", encoding="utf-8").write("\n".join(lines))
import idc
idc.qexit(0)
