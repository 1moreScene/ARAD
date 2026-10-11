# -*- coding: utf-8 -*-
# L1 精修脚本：script_string.cpp —— 只加注释/分区横幅，函数体零改动
import io

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\game\script_string.cpp"
lines = io.open(PATH, encoding="utf-8").read().split("\n")

new_header = """// ============================================================
// 【40JP逆向源代码工程】game/script_string.cpp
// 模块: 脚本字符串管理/元素/职业标签/战斗分发器宿主类(CNRDUdpModule)/窗口族存根
// 来源: work/dump_combat.txt  区块: 0x440000-0x450000
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
// 函数数: 268
// ------------------------------------------------------------
// ★ 本文件已按以下定案升级注释（锚点速查）：
//   0x4422C0  脚本包根路径 + Script.pvf 挂载判定（_ScriptPacks/!ScriptPacks 自证）
//   0x4426A0  职业标签→索引（[swordman]/[fighter]/[gunner]/[mage]）
//   0x442710  元素标签→索引（[fire/water/dark/light element]）
//   0x442780  当前元素标签 setter（全局 dword_97B384）
//   0x442E10  RDARScriptStringManager 入口（"::ERROR::input NULL string" 自证）【推断】
//   0x444C20/0x444C60  CNMainGameModule vmt8/vmt4 —— L1 逐行精修版见
//             src/game/module/CNMainGameModule.cpp（docs/13 §1），此处保留 L2 转写体
//   0x445650  CNRDUdpModule::vmt90 按场景取操作角色（本会话早前已由主代理补齐，未改动）
//   0x446130/0x446190  PVP/WARROOM 被动物体特殊绑定（分发器 case 8/0x15/0x16 配套）
//   0x446710/0x446F00/0x449EB0  peer 加入/清理广播（@@ADD/REMOVE PEER 自证）
//   0x44EA20  模块切换收尾（docs/13 §3 末行定案调用）
// ★ 战斗分发器本体 sub_4471E0（CNRDUdpModule::vmt0，11KB 巨函数）单独成文件：
//   src/game/bigfunc_4471E0_sub_4471E0.cpp（含全 case 语义表）
// 交叉引用：发包 API 0x423200/0x423380 已 L1 精修于 src/nengine/net_socket.cpp（docs/13 §5）
// ============================================================"""

sections = {
  "0x440F90": """// ============================================================
// 【分区 1】脚本字符串管理核心（0x440F90-0x442280）
// RDARScriptStringManager 巨函数对（0x440F90 ~360 行 / 0x441750 ~470 行，
// std::string 容器操作 sub_40BF00/43BF40/43DA50）+ 容器/vector 辅助。
// 词条与标签的具体取值见分区 2 的标签族。
// ============================================================""",
  "0x442280": """// ============================================================
// 【分区 2】脚本包体与标签解析（0x442280-0x4428C0）
// 日志转发桩 → Script.pvf 包体挂载（0x4422C0，_ScriptPacks/!ScriptPacks 目录探针）
// → token 数值解析族（0x4423D0/0x442480/0x442520/0x4425A0，%d/%x/%f 三型）
// → 职业标签族（0x4426A0）与元素标签族（0x442710/0x442780）。
// ============================================================""",
  "0x4428C0": """// ============================================================
// 【分区 3】字符串容器/管理器辅助（0x4428C0-0x444360）
// 28 字节/项的 std::string 数组搬运与查重；0x442E10 为
// RDARScriptStringManager 入口（"::ERROR::input NULL string" 自证）【推断】；
// 错误串二次使用于 0x443370；0x442EF0 = vector length_error 抛出（CRT 模板实体）。
// ============================================================""",
  "0x444360": """// ============================================================
// 【分区 4】CNMainGameModule 虚表槽（0x444360-0x444E70）
// vmt8=0x444C20 模块退出清理 / vmt4=0x444C60 每帧 tick（模块心跳）。
// 两函数已由 docs/13 §1 定案并 L1 逐行精修于 src/game/module/CNMainGameModule.cpp
// （vmt4 流程：帧钩→场景推进→震动档→fade 状态机→发包 type=40），此处保留 L2 转写体。
// ============================================================""",
  "0x444E70": """// ============================================================
// 【分区 5】战斗分发器共享工具（0x444E70-0x4451F0）
// 被 bigfunc_4471E0（sub_4471E0 = CNRDUdpModule::vmt0）直接调用：
//   0x4450E0 ← case 0x18 / 0x445120 ← case 0x14 / 0x445150 ← case 0xA 收尾 /
//   0x4451C0 ← case 0xF（进图音效旗标）。
// ============================================================""",
  "0x4451F0": """// ============================================================
// 【分区 6】CNRDUdpModule 虚表族 + CNRDParticle 族（0x4451F0-0x449EB0）
// 战斗分发器宿主类（VT 006C46B4，docs/11 排期表）。vmt 槽实测：
// vmt8/3C/5C/60/64(队友入场 @@ADD PEER 2)/68/7C/88/90(按场景取操作角色)/98/A8/AC/B0；
// UDP 回报落地 0x4456C0；PVP/WARROOM 被动物体绑定 0x446130/0x446190；
// 分发器 case 2/5 读包配套 0x446960-0x446CF0；粒子族 0x445C10-0x445EF0。
// 分发器本体见 src/game/bigfunc_4471E0_sub_4471E0.cpp。
// ============================================================""",
  "0x449EB0": """// ============================================================
// 【分区 7】CNRDUdpModule__vmt74（0x449EB0-0x44A950）
// peer 表重建/清理：@@REMOVE PEER + @@ADD PEER 4（日志自证）。
// ============================================================""",
  "0x44A950": """// ============================================================
// 【分区 8】窗口族 vmt4 存根 + 界面管理器（0x44A950-0x44FF60）
// IRDWindow/CNRD*Window 各窗口类 vmt4 槽（多为调用 0x44A950 共享实现或空存根）；
// CNRDInterfaceManager 构造 0x44BA00；模块切换收尾 0x44EA20（docs/13 §3 定案）。
// ============================================================""",
}

funcs = {
  "0x440F90": "// 【推断】RDARScriptStringManager 词条表操作（std::string 容器 + sub_43BF40/43DA50）",
  "0x441750": "// 【推断】RDARScriptStringManager 词条表操作 II（usercall，ecx/edi/esi 三隐参）",
  "0x4422C0": """// 脚本包根路径设置：拷贝路径到 byte_8F8898（补 \\\\ 或 / 结尾）；
// "_ScriptPacks" 存在 → byte_78FF3C=1；"!ScriptPacks" 不存在且标志置位 →
// sub_528DB0("Script.pvf") 打开脚本包体（0x528DB0 = pvf 挂载器）。""",
  "0x442390": "// 路径拼接：sprintf(出, \"%s%s\", 包根 byte_8F8898, vsprintf 后的格式串)",
  "0x4423D0": "// token 数值解析：按 sub_65BE30 词法型别 2=%d / 3=%x / 4=%f 取值；*a1=是否可解析",
  "0x4426A0": "// 职业标签→索引：[swordman]=0 / [fighter]=1 / [gunner]=2 / [mage]=3（strcmp 链自证）",
  "0x442710": "// 元素标签→索引：[fire]=0 / [water]=1 / [dark]=2 / [light]=3（strcmp 链自证）",
  "0x442780": """// 当前元素标签 setter：按入参写全局 dword_97B384
// （"[water element]"/"[dark element]"/"[light element]"/缺省 "[fire element]"，
// 与 0x442710 索引一一对应；该全局即 UI/技能元素着色取词源）""",
  "0x442E10": "// RDARScriptStringManager 入口：空串防御 \"RDARScriptStringManager::ERROR::input NULL string\"【推断】",
  "0x444C20": "// CNMainGameModule::vmt8 模块退出清理 —— L1 精修版见 src/game/module/CNMainGameModule.cpp:238",
  "0x444C60": """// CNMainGameModule::vmt4 每帧 tick（模块心跳）—— docs/13 §1 定案全流程：
// 帧钩 sub_4AEB30 → 主帧循环 sub_433690 → 场景推进 → 用户震动档 → fade 状态机。
// L1 逐行精修版见 src/game/module/CNMainGameModule.cpp:152；此处保留 L2 转写体。""",
  "0x4450E0": "// 分发器 case 0x18 落点：场景级事件处理【推断】",
  "0x445120": "// 分发器 case 0x14 落点：命中侧查询（返回真走 sub_495470(+v174[1668]) 分支）",
  "0x445150": "// 分发器 case 0xA 收尾：装备同步资源清理（v306 结构池回收）",
  "0x4451C0": "// 分发器 case 0xF 落点：进图音效旗标（48001 已播标记）",
  "0x4456C0": """// 分发器 case 0 配套：UDP 回报落地 —— 按模块类型 4(PVP)→sub_48F690 通道、
// 6(WARROOM)→对应通道分发【推断】；与 0x446CF0 成对使用""",
  "0x445940": "// 对端就绪查询：宿主(+0x100200 处指针)+2224 标志为 0 即就绪（分发器配套）",
  "0x446130": "// 分发器 case 8/0x15/0x16 配套：PVP（模块类型 4）被动物体特殊绑定",
  "0x446190": "// 分发器 case 8/0x15/0x16 配套：WARROOM（模块类型 6）被动物体特殊绑定",
  "0x446710": """// 【推断】队伍成员信息广播：@@ADD PEER 2 %d / @@ADD PEER 1 %d [%s] %d（名字+5063）
// —— 与 0x446F00(vmt64) 同族，操作 this+0x100000 级 peer 表""",
  "0x446960": "// 分发器 case 2 配套：对象可见表重置【推断】",
  "0x4469A0": "// 分发器 case 5 配套：按计数建表（逐项 0x446BE0 读取）【推断】",
  "0x446AB0": "// 分发器 case 5 配套：填充 3 float 向量结构（sub_4E7DA0 取数）【推断】",
  "0x446BE0": "// 分发器 case 5 配套：逐项读入状态字节【推断】",
  "0x446CF0": "// 分发器 case 0 配套：UDP RETURN/FINISH 分支处理（子码 1/2）【推断】",
  "0x446F00": """// CNRDUdpModule::vmt64 —— 队友入场同步（"@@ADD PEER 2 %d" 日志自证）：
// 加入者座位/名字经 sub_42A380 取名、声音通道表(sub_656710/656880)广播；
// a4/a5 控制 peer 表(+28*槽) 状态写入""",
  "0x447140": "// CNRDUdpModule::vmt98【推断】peer 表操作（与本分区 vmt 族同宿主）",
  "0x449EB0": """// CNRDUdpModule::vmt74 —— peer 表重建/清理（日志自证）：
// "@@REMOVE PEER" 逐席移除后按需 "@@ADD PEER 4 %d" 重建""",
  "0x44BA00": "// CNRDInterfaceManager 构造：窗口族 vmt4 存根的宿主",
  "0x44EA20": """// 模块切换收尾 sub_44EA20(from, to) —— docs/13 §3 定案调用：
// sub_41CC80 迁移矩阵执行后的资源收尾（窗口/声音/上下文按 from/to 清理）""",
}

def find(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]):
            return i
    return -1

SEP = "// ------------------------------------------------------------"

# 1) 头部替换（原头 8 行，动态找结束横幅）
assert lines[0].startswith("// ====")
hdr_end = next(i for i in range(1, 15) if lines[i].startswith("// ===="))
lines[0:hdr_end+1] = new_header.split("\n")

# 2) 自上而下：先插横幅（在 VA 块上方的 SEP 行之前），再插函数注释（在 VA 块下方 SEP 行之后）
marks = sorted(set(list(sections.keys()) + list(funcs.keys())), key=lambda v: int(v, 16))
offset = 0
n_sec = n_fun = 0
for va in marks:
    pat = "// " + va + " "
    mi = find(lambda l: l.startswith(pat), offset)
    assert mi > 0, va
    assert lines[mi-1].strip() == SEP, (va, lines[mi-1])
    assert lines[mi+1].strip() == SEP, va
    if va in sections:
        lines[mi-1:mi-1] = sections[va].split("\n")
        mi += len(sections[va].split("\n"))
        n_sec += 1
    if va in funcs:
        lines[mi+2:mi+2] = funcs[va].split("\n")
        n_fun += len(funcs[va].split("\n"))
    offset = mi + 2

io.open(PATH, "w", encoding="utf-8", newline="\n").write("\n".join(lines))
print("OK script_string: %d sections, %d comment lines" % (n_sec, n_fun))
