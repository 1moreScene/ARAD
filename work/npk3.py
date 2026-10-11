# -*- coding: utf-8 -*-
# 第三轮: PVF 解密调用点反汇编 + CRC 表验证 + 内容树读取器
import idaapi, idautils, idc, ida_bytes, ida_funcs, ida_hexrays
import struct

OUT = r"E:\DOF LX\【40JP逆向源代码工程】\work\recon_npk_pvf3.txt"
lines = []
def log(s): lines.append(s)

# 1. sub_527AC0 调用点反汇编（在 sub_528DB0 内）
log("### sub_528DB0 内对 sub_527AC0 的调用点反汇编")
f = ida_funcs.get_func(0x528DB0)
ea = f.start_ea
while ea < f.end_ea:
    if ida_bytes.get_byte(ea) == 0xE8:
        import struct as st
        rel = st.unpack_from("<i", ida_bytes.get_bytes(ea+1, 4), 0)[0]
        if ea + 5 + rel == 0x527AC0:
            start = max(f.start_ea, ea - 48)
            log("-- call site %08X, 前 48 字节 --" % ea)
            cur = start
            while cur <= ea:
                log("  %08X  %s" % (cur, idc.GetDisasm(cur)))
                cur = ida_bytes.next_head(cur, ea + 1)
    ea = ida_bytes.next_head(ea, f.end_ea)

# 2. sub_527AC0 本体的入口反汇编（确认 userpurge 寄存器参数）
log("")
log("### sub_527AC0 入口反汇编 (前 60 字节)")
ea = 0x527AC0
for _ in range(18):
    log("  %08X  %s" % (ea, idc.GetDisasm(ea)))
    ea = ida_bytes.next_head(ea, 0x528000)

# 3. CRC 表验证: dword_9B8E80 前 8 项
log("")
log("### dword_9B8E80 CRC 表前 8 项")
for i in range(8):
    log("  [%d] %08X" % (i, ida_bytes.get_dword(0x9B8E80 + i*4)))
log("  (标准 CRC-32/IEEE 表: 00000000 77073096 EE0E612C 990951BA 076DC419 706AF48F E963A535 9E6495A3)")

# 4. sub_57A1C0 (CRC 表初始化? dword_9B927C 门)
log("")
log("### sub_57A1C0 (表初始化函数)")
try:
    cf = ida_hexrays.decompile(0x57A1C0)
    lines.extend(str(cf).split("\n")[:80])
except Exception as e:
    log("// exc %s" % e)

# 5. PVF 内容树读取器: 引用 unk_97B210 的函数
log("")
log("### 引用 unk_97B210 (PVF 对象) 的函数")
funcs = set()
for x in idautils.XrefsTo(0x97B210):
    f = ida_funcs.get_func(x.frm)
    if f: funcs.add(f.start_ea)
log(", ".join("%08X" % x for x in sorted(funcs)))
# 反编译前 4 个
for fea in sorted(funcs)[:4]:
    log("")
    log("// ===== PVF content %08X =====" % fea)
    try:
        cf = ida_hexrays.decompile(fea)
        lines.extend((str(cf).split("\n") if cf else ["// failed"])[:180])
    except Exception as e:
        log("// exc %s" % e)

# 6. NPK 数据读取: IMG 打开 0x418CD0 调用 0x40E3E0 后怎么用（读 NPK 内数据）
log("")
log("### IMG/NPK 数据消费: 0x418CD0 头部")
try:
    cf = ida_hexrays.decompile(0x418CD0)
    lines.extend(str(cf).split("\n")[:120])
except Exception as e:
    log("// exc %s" % e)

open(OUT, "w", encoding="utf-8").write("\n".join(lines))
import idc
idc.qexit(0)
