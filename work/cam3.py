# -*- coding: utf-8 -*-
import ida_hexrays, idc
lines = []
for ea, tag in [(0x425570, "zoom tick candidate A"), (0x42F910, "zoom tick candidate B")]:
    lines.append("// ===== %s @ %08X =====" % (tag, ea))
    try:
        cf = ida_hexrays.decompile(ea)
        lines.append(str(cf) if cf else "// failed")
    except Exception as e:
        lines.append("// exc %s" % e)
    lines.append("")
open(r"E:\DOF LX\【40JP逆向源代码工程】\work\decomp_cam_verify3.txt", "w", encoding="utf-8").write("\n".join(lines))
idc.qexit(0)
