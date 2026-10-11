# -*- coding: utf-8 -*-
# L1 精修脚本：bigfunc_4471E0_sub_4471E0.cpp —— 只加注释，函数体零改动
import io, re, sys

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\game\bigfunc_4471E0_sub_4471E0.cpp"
src = io.open(PATH, "r", encoding="utf-8").read()
lines = src.split("\n")

# ---------- 1) 文件头替换（原 1-5 行） ----------
new_header = """// ============================================================
// 【40JP逆向源代码工程】src/game/bigfunc_4471E0_sub_4471E0.cpp
// 模块: ★战斗分发器 sub_4471E0 = CNRDUdpModule::vmt0（docs/90 P2 定案）
// 来源: work/dump_combat.txt  区块: 0x44（单独反编译，size=11291 字节）
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
// 函数数: 1（单文件单函数，放宽行数上限）
// ------------------------------------------------------------
// ★ 包接收结构（函数前导直证）：
//   +0  BYTE   附加标识（仅入 "RECV %d %d" 日志第二参数）
//   +1  WORD   包类型 = switch 分发键
//   +3  DWORD  校验和（sub_48BE40(体,len) 比对，不符仅打日志返回）
//   +7  ...    包体（先经 sub_48BDC0 解密/预处理）
//   读包四档读取器：vt+72=1 字节 / vt+76=2 字节 / vt+80=4 字节 / vt+88=定长块
// ★ 宿主字段：this+262149(int*下标)=+0x100034 对象容器宿主(+104=对象列表)；
//   dword_8F7A24+12=当前模块类型(1主城/2选图/4PVP/6WARROOM，docs/13 §3)；
//   dword_8F7C04=本地主角；dword_8F8548=全局通知管理器【推断】；dword_99FD58=震屏管理器
// ★ 全 case 语义表见各 case 行上注释；缺席分支：0x01、0x24（服务器侧专用或废弃【待验证】）
// ============================================================"""

# ---------- 2) case 行上注释 ----------
case_comments = {
 0:  "case 0 ——【字符串自证】UDP 通道握手回报：子码 0=REPLY TO / 1=RETURN / 2=FINISH（sub_446CF0/sub_4456C0/sub_52D970）",
 2:  "case 2 —— 对象可见列表刷新：按计数循环 sub_455EB0 注册，尾部 (字节,字节) → sub_453DD0（选中槽同步）",
 3:  "case 3 —— 选中槽同步：两个字节直传 sub_453DD0（座位/选中项 + 状态）",
 4:  "case 4 ——【推断】远程物件移除：id 对查对象 → 校验归属(vt+304/48/52) → vt+60 析构 + sub_4A0180 释放",
 5:  "case 5 ——【推断】对象状态/击退同步：id 对 + 3D 向量(sub_4469A0/446AB0/4E7DA0) → 判定体 vt+472 施向量、vt+228 状态机、vt+568/572 → sub_49FFC0",
 6:  "case 6 ——【推断】怪物归属绑定：IRDMonster ← 玩家槽(sub_429830)[49] 经 vt+1044 设主；失败打 aCriticalError 串",
 7:  "case 7 ——【推断】怪物行为触发：IRDMonster vt+1088（AI 事件）",
 8:  "case 8 —— 被动物体目标绑定：CNRDPassiveObject vt+680(目标碰撞体)；模块 4(PVP)→sub_446130 / 6(WARROOM)→sub_446190",
 9:  "case 9 ——【推断】对象管理器接管：为对象创建 156B 管理对象(sub_445D00/445B90/4A0B60)，同步 3 float、vt+148/vt+4",
 0xA:"case 0xA ——【结构自证】角色装备槽同步：0xFFFF=卸槽(vt+792/vt+1080)；否则 sub_522300 生成 CNRDEquipment → vt+784 穿戴 + 槽表 sub_429830(...)74+4*槽",
 0xB:"case 0xB —— 全装备栏校验刷新：本地主角 18 槽逐一比对 item id，不符即 vt+812(卸下)/vt+816(清槽)",
 0xC:"case 0xC ——【推断】对象数据注入：IRDActiveObject vt+732(data, 0, 1)",
 0xD:"case 0xD ——【推断】对象数据注入：IRDActiveObject vt+748(data, 1)",
 0xE:"case 0xE ——【推断】技能/状态事件：IRDActiveObject vt+768(子码, WORD, 目标碰撞体)",
 0xF:"case 0xF ——【推断】队伍编号同步：vt+672(0)=队号、vt+676(0)；本地主角且队号变化 → 48001 音效 + 502 号串横幅 + sub_41C150",
 0x10:"case 0x10 ——【推断】对象渲染数据块 +0 字段增量（vt+348 取块首 float *f += 值）",
 0x11:"case 0x11 ——【推断】对象渲染数据块 +8 字段增量（int16 累加）",
 0x12:"case 0x12 ——【推断】对象叠加状态表：vt+348 块 +24 起 3 槽（id / 权重×0.01 / 参数，17 结尾）",
 0x13:"case 0x13 ——【研究定案】碰撞体交互同步：两判定体(vt+380)间 6 参数链式设定(vt+496/508/500/512/504/516)，17 状态判定体 → vt+700 + sub_53FB90",
 0x14:"case 0x14 ——【研究定案】命中/伤害包：受击方 vt+576 判定体、攻击方 vt+732 受击状态、飘字咽喉 sub_4D7960（研究 20 §1.1）、受击位移 sub_424270；PVP 本体受击 → sub_496010 震屏(8)",
 0x15:"case 0x15 ——【推断】实体批量生成 I：按模块类型取场景管理器(4→+32/6→+40/余+28)，sub_53A220 创建 + sub_45FE80 注册；分支A 建 156B 管理对象(sub_52A2A0)，分支B vt+148/vt+112",
 0x16:"case 0x16 ——【推断】实体批量生成 II（带数据 blob）：sub_4ABE10 创建(+7801 头目标记)、vt+1164/1168/372/148/136/140/112/1112；aCriticalError_3 自证",
 0x17:"case 0x17 ——【推断】对象清理：vt+84(0)（退出可视/停止）",
 0x18:"case 0x18 —— 场景级事件 → sub_4450E0()（分发器配套，见 script_string.cpp）",
 0x19:"case 0x19 ——【推断】掉落物生成挂接：IRDActiveObject vt+868(物品id) + sub_522300 物品 + vt+136",
 0x1A:"case 0x1A ——【推断】对象状态对同步：vt+904(b1,b2)；无判定体时补 vt+748",
 0x1B:"case 0x1B ——【推断】批量状态竞争设置：vt+236==0 或 (state 相同且 vt+248 参数更小) → vt+228(state,param,0)",
 0x1C:"case 0x1C ——【字符串自证】装备破碎事件：角色槽物品 vt+196!=0 且为本地主角 → \"ITEM_BREAK\" 音效",
 0x1D:"case 0x1D ——【推断】怪物事件：IRDMonster vt+1060",
 0x1E:"case 0x1E ——【推断】归属播报：两名玩家槽(sub_429830)；同名→504 号串、异名→505 号串，横幅 sub_43BEE0(-6567706)；主人 vt+1248(1)",
 0x1F:"case 0x1F ——【推断】角色槽物品操作：IRDCharacter 槽(vt+792) → 物品 vt+284（耐久/消耗类）",
 0x20:"case 0x20 ——【推断】怪物目标绑定：IRDMonster vt+1152(目标对象)",
 0x21:"case 0x21 —— 对象移除 → sub_45ED80（分发器配套，见 worldmap_reward.cpp）",
 0x22:"case 0x22 —— 全场怪物参数批量同步 → sub_45EF30（分发器配套，见 worldmap_reward.cpp）",
 0x23:"case 0x23 —— 震屏复位 → sub_496320(dword_99FD58)（震屏管理器，docs/12）",
 0x25:"case 0x25 —— 加载门推进 → sub_452700(byte)：8 门全开触发事件（见 worldmap_reward.cpp）",
 0x26:"case 0x26 ——【推断】WARROOM 专属事件：仅模块类型 6 处理；IRDCharacter → sub_4B09B0(byte)",
 0x27:"case 0x27 ——【推断】全局通知管理器 dword_8F8548 vt+0(a2, dword)",
 0x28:"case 0x28 ——【推断】dword_8F8548 vt+4(a2)",
 0x29:"case 0x29 ——【推断】dword_8F8548 vt+8(a2, dword)",
 0x2A:"case 0x2A ——【推断】声音事件 → sub_656080(a2)（0x656xxx 声音通道区，docs/13 §2）",
 0x2B:"case 0x2B ——【推断】dword_8F8548 vt+12(byte, dword)",
 0x2C:"case 0x2C ——【推断】dword_8F8548 vt+16(dword, byte)",
}

# ---------- 3) 函数头注释（插在 VA 标记块之后） ----------
func_head = """// ★ CNRDUdpModule::vmt0 —— 网络 UDP 包分发虚入口（docs/90 P2：被判定者算法命名定案）。
// 流程：vt+64 登记读指针(+7 跳过包头) → vt+140 "RECV %d %d"(类型, 附加标识) 日志 →
//       sub_48BDC0(体,len,0) 解密/预处理 → sub_48BE40 校验和与 +3 DWORD 比对 →
//       switch(包类型) 分发 43 个 case（0x00..0x2C，缺 0x01/0x24）→ default 忽略。
// 下方每个 case 行上注释 = 本次扫描产出：字符串自证标【字符串自证】，其余为
// 结构推断【推断】，0x13/0x14 为打击感研究已定案锚点（研究 20）。"""

# ---- 定位执行 ----
def find_line(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]):
            return i
    return -1

# 头部：原 1-5 行整体替换
assert lines[0].startswith("// ===="), lines[0]
assert lines[4].startswith("// ===="), lines[4]
lines[0:5] = new_header.split("\n")

# VA 标记块之后插函数头注释
mi = find_line(lambda l: l.strip() == "// 0x4471E0 sub_4471E0  size=11291")
assert mi > 0, "VA marker not found"
assert lines[mi+1].startswith("// ---")
ins = mi + 2
lines[ins:ins] = func_head.split("\n") + [""]

# case 注释：自上而下，行号因插入而偏移，用相对查找
offset = 0
for cv in sorted(case_comments.keys()):
    pat = "case 0x%X:" % cv if cv >= 10 else "case %d:" % cv
    idx = find_line(lambda l: l.strip() == pat, offset)
    assert idx > 0, "case not found: " + pat
    indent = lines[idx][: len(lines[idx]) - len(lines[idx].lstrip())]
    lines[idx:idx] = [indent + "// " + case_comments[cv]]
    offset = idx + 1

# default 注释
di = find_line(lambda l: l.strip() == "default:", offset)
assert di > 0, "default not found"
lines[di:di] = ["    // default —— 未知包类型：静默忽略"]

io.open(PATH, "w", encoding="utf-8", newline="\n").write("\n".join(lines))
print("OK bigfunc_4471E0: header + %d case comments + func head" % (len(case_comments) + 1))
