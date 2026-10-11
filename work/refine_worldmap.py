# -*- coding: utf-8 -*-
# L1 精修脚本：worldmap_reward.cpp —— 只加注释/分区横幅，函数体零改动
import io

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\game\worldmap_reward.cpp"
lines = io.open(PATH, encoding="utf-8").read().split("\n")

new_header = """// ============================================================
// 【40JP逆向源代码工程】game/worldmap_reward.cpp
// 模块: 世界地图/副本选择/房间列表/奖励（CNPVPModule/CNSelectDungeonModule/
//       CNRoomListModule/CNRDParty/CNRDReward/CNRDNpc）
// 来源: work/dump_combat.txt  区块: 0x450000-0x460000
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
// 函数数: 217
// ------------------------------------------------------------
// ★ 本文件已按以下 UI 事件锚点升级注释（sub_432EA0 事件名 = 字符串自证）：
//   地图交互：MAP_DISAPPEAR(0x453B10) / MAP_MOUSEOVER·MAP_SELECT
//     （CNSelectDungeonModule__vmt4=0x454740）/ MAP_COUNTDOWN(0x454F00 巨型)
//   按钮音效：CLICK_BUTTON1(0x451AD0) / CLICK_BUTTON2（0x453DD0 座位确认、
//     0x4577F0 奖励开箱等多处）
//   奖励系统：REWARD_OPEN1(0x4577F0/0x458120) / REWARD_OPEN2(0x458930) /
//     REWARD_BOX_SET·REWARD_ICON_OFF(0x459ED0) / REWARD_BOX_GLITTER(CNRDReward__vmt0)
//   世界地图创建："CRITICAL ERROR : WORLD MAP LOAD FAIL %d" 自证
//     0x456210（地图创建）/ CNRoomListModule__vmt8=0x45B4A0（房间列表侧）
//   PVP 房间地图："HendonMyre/Hendon_Pub_PVP.map" 自证（0x452C50）
// ★ 战斗分发器（sub_4471E0，见 bigfunc_4471E0_sub_4471E0.cpp）配套落点：
//   case 2/3 → 0x453DD0 座位同步；case 2 循环体 → 0x455EB0 队伍分组登记；
//   case 0x21 → 0x45ED80 对象移除；case 0x22 → 0x45EF30 全场怪物同步；
//   case 0x15/0x16 → 0x45FE80 对象注册；id→对象解析 → 0x45EBE0（全 case 共用）
// ★ 帧循环锚点（docs/13 §2）：0x451D50 = 主管理器 tick（dword_8F8528）；0x45E1A0 =
//   UDP 接收泵（dword_8F8524，SOCKET recvfrom 直证，按模块类型分发）
// 交叉引用：发包 API 0x423200/0x423380 已 L1 精修于 src/nengine/net_socket.cpp
// ============================================================"""

sections = {
  "0x450130": """// ============================================================
// 【分区 1】世界地图数据容器（0x450130-0x452580）
// 地图/房间词条表与容器操作（含 0x451D50 主管理器 tick，docs/13 §2）。
// ============================================================""",
  "0x452580": """// ============================================================
// 【分区 2】地图选择工具（0x452580-0x452FA0）
// 加载门 8 态推进 0x452700（分发器 case 0x25 落点）；
// PVP 房间地图装载 0x452C50（"HendonMyre/Hendon_Pub_PVP.map" 自证）。
// ============================================================""",
  "0x452FA0": """// ============================================================
// 【分区 3】CNPVPModule 与座位管理（0x452FA0-0x454610）
// CNPVPModule__vmt4（RTTI 自证）；0x453DD0 = 座位/选中槽同步
// （战斗分发器 case 2/3 落点，CLICK_BUTTON2 自证）。
// ============================================================""",
  "0x454610": """// ============================================================
// 【分区 4】副本选择界面（0x454610-0x455E20）
// CNSelectDungeonModule__vmt4 = 副本选择帧（MAP_MOUSEOVER/MAP_SELECT）；
// 副本倒计时巨函数 0x454F00（MAP_COUNTDOWN 自证，~780 行）。
// ============================================================""",
  "0x455E20": """// ============================================================
// 【分区 5】地图创建与队伍分组登记（0x455E20-0x456690）
// 队伍分组登记 0x455EB0（分发器 case 2 循环体落点）；
// 世界地图创建 0x456210（"CRITICAL ERROR : WORLD MAP LOAD FAIL %d" 自证）；
// 座位查询族 0x456390/0x456420/0x456480（docs/13 §3 进图 2 座位分配）。
// ============================================================""",
  "0x456690": """// ============================================================
// 【分区 6】队伍管理（0x456690-0x457790）
// CNRDParty__vmt0（RTTI 自证）与队员槽位维护。
// ============================================================""",
  "0x457790": """// ============================================================
// 【分区 7】奖励系统 I（0x457790-0x458930）
// 奖励箱开启：REWARD_OPEN1（0x4577F0/0x458120，CLICK_BUTTON2 自证）。
// ============================================================""",
  "0x458930": """// ============================================================
// 【分区 8】奖励系统 II（0x458930-0x459ED0）
// REWARD_OPEN2（0x458930 自证）——二级奖励开启。
// ============================================================""",
  "0x459ED0": """// ============================================================
// 【分区 9】奖励系统 III（0x459ED0-0x45AC20）
// 奖励箱点亮动画 0x459ED0（REWARD_BOX_SET / REWARD_ICON_OFF 自证，
// 4 箱 × sub_4049F0 500ms 插值）。
// ============================================================""",
  "0x45AC20": """// ============================================================
// 【分区 10】CNRDReward 奖励主类（0x45AC20-0x45B4A0）
// CNRDReward__vmt0（RTTI 自证；REWARD_BOX_GLITTER 自证）。
// ============================================================""",
  "0x45B4A0": """// ============================================================
// 【分区 11】房间列表模块（0x45B4A0-0x45D190）
// CNRoomListModule__vmt8/vmt4（RTTI 自证；vmt8 侧同样打
// "CRITICAL ERROR : WORLD MAP LOAD FAIL %d"）。
// ============================================================""",
  "0x45D190": """// ============================================================
// 【分区 12】对象管理器（0x45D190-0x45FB20）
// UDP 接收泵 0x45E1A0（docs/13 §2 主帧循环调用）；id 对→对象查找 0x45EBE0
// （分发器全 case 共用）；0x45ED80/0x45EF30 = 分发器 case 0x21/0x22 落点。
// ============================================================""",
  "0x45FB20": """// ============================================================
// 【分区 13】NPC 与对象注册（0x45FB20-0x460000）
// CNRDNpc__vmt0/vmt154（RTTI 自证）；0x45FE80 = 对象注册
// （分发器 case 0x15/0x16 与 ui_icon_perf 0x4600B0 的注册链入口）。
// ============================================================""",
}

funcs = {
  "0x451AD0": "// 世界地图交互：CLICK_BUTTON1（按钮音效自证）【推断：翻页/确认】",
  "0x451D50": """// 主管理器 tick（docs/13 §2：主帧循环 sub_433690 调 sub_451D50(0x8F8528)）——
// 全 UI/窗口管理器心跳；快门特性横幅去重亦经此路径（docs/13 §2）""",
  "0x452700": """// 分发器 case 0x25 落点 —— 加载门推进：a2(0..7) 置 +128 位表；
// 8 门（sub_48F690 查询存在性）全开或 a2==8 → sub_698FE0 触发事件""",
  "0x452B60": "// 【推断】地图词条/路径辅助",
  "0x452C50": """// PVP 房间地图装载（"HendonMyre/Hendon_Pub_PVP.map" 自证）：
// sub_452B60 取词条 → dword_8F84B8 vt+60 载入 .map → 场景层创建""",
  "0x453250": "// CNPVPModule::vmt4 —— PVP 模块帧处理（RTTI 自证）",
  "0x453DD0": """// 座位/选中槽同步 —— 战斗分发器 case 2/3 落点：
// +276 座位表（16 字节/项）/ +288 当前选中；a2==当前且 +4 槽位吻合则短路返回；
// 换座经 sub_476B60 取角色数据 → 23 字节/项的名字表刷新（sub_69A5F0）→
// +16/+20 按钮可见性 → a3!=4 时回写 +4 槽位并播 CLICK_BUTTON2""",
  "0x454740": """// CNSelectDungeonModule::vmt4 —— 副本选择界面帧（RTTI 自证）：
// 主帧循环 → 选中项命中测试 sub_454070（1100ms 启动门 +180）→
// MAP_MOUSEOVER（悬停变更）/ MAP_SELECT（确认）；悬停信息条 dword_8F8528 vt 55""",
  "0x454F00": """// ★ 巨型（~780 行）副本倒计时处理 —— MAP_COUNTDOWN 自证（4558 行附近）：
// 全员就绪轮询 → 倒计时推进 → 入场转移【推断】""",
  "0x455EB0": """// 队伍分组登记 —— 分发器 case 2 循环体落点：
// sub_476B60(角色) +92 队伍规模 → 与本地座位(sub_456480→+16)规模比较定分组级
// （0/1/2，-1=不可比）→ 本地座位补 sub_472F90 → Block{id,0,数据,等级} →
// sub_455E40 入 +272 表""",
  "0x456210": """// 世界地图创建 —— "CRITICAL ERROR : WORLD MAP LOAD FAIL %d" 自证：
// sub_4CE8A0 取地图数据 → 图像名 → dword_8F84B8 vt+60 载入 → sub_418DD0 建层
// → sub_455FC0/sub_4542A0 构建节点 → +6/+7 按钮激活 → 2×300ms 定时器（+140 位表）""",
  "0x456390": "// 座位号查询（本地玩家 → 座位 index；触发条件评估器 case 5 所用）",
  "0x456420": """// 座位查找（docs/13 §3：进 2 = MAP_APPEAR + 座位 sub_456420）：
// 本地角色在 +4 起的 4 槽中定位 → 0..3，未找到 -1""",
  "0x456480": "// 座位号（PVP/WARROOM 场景恒 -1；其余取 +8）",
  "0x456CD0": "// CNRDParty::vmt0 —— 队伍对象构造（RTTI 自证）",
  "0x4577F0": """// 奖励箱开启 —— REWARD_OPEN1 + CLICK_BUTTON2 自证：
// +127 槽空且 a3 有效 → 记箱号/时刻；与本地主角 vt+1216 比对 → +135 命中标记
// （CLICK_BUTTON2）否则 +146=-1 → 按钮熄灭 → REWARD_OPEN1""",
  "0x458120": "// 奖励箱开启循环（REWARD_OPEN1 自证）【推断：多箱批量推进】",
  "0x458930": "// 奖励箱二级开启 —— REWARD_OPEN2 自证（7308 行附近）",
  "0x459ED0": """// 奖励箱点亮动画 —— REWARD_BOX_SET / REWARD_ICON_OFF 自证：
// +252 表（40 字节/项）前 4 箱按 +8 时钟差插值（sub_4049F0 0→30 / 500ms）；
// +264<11 且 +432 未置位时记时刻并 REWARD_BOX_SET""",
  "0x45AC20": "// CNRDReward::vmt0 —— 奖励主类帧处理（RTTI 自证；REWARD_BOX_GLITTER 自证）",
  "0x45B4A0": "// CNRoomListModule::vmt8 —— 房间列表通知（RTTI 自证；世界地图装载失败同串）",
  "0x45CB30": "// CNRoomListModule::vmt4 —— 房间列表帧处理（RTTI 自证）",
  "0x45E1A0": """// UDP 接收泵（docs/13 §2：主帧循环 sub_433690 调 sub_45E1A0(0x8F8524)）：
// dword_8F7B08+8 已连接且模块 +16 槽使能 → 按 +68 模块类型分发：
// 4(PVP) → sub_45DDA0 + sub_445940 就绪查询 + sub_45DF90；recvfrom/in_addr 直证""",
  "0x45EBE0": """// id 对 → 对象查找（战斗分发器全 case 共用解析器）：
// 遍历 +2 对象表，vt+48/vt+52 返回值与 (a2,a3) 双字匹配 → 返回对象，未中 0""",
  "0x45ED80": "// 对象移除 —— 分发器 case 0x21 落点【推断：退场清理】",
  "0x45EF30": """// 全场怪物参数批量同步 —— 分发器 case 0x22 落点：
// dword_8F8520 vt+76 逐对象读 id 对 → IRDMonster → vt+1128(4 参数)；
// 值相等则以 -2 抹平（1 万次上限，aCriticalErrorC_1 自证溢出）""",
  "0x45FB90": "// CNRDNpc::vmt0 —— NPC 对象构造（RTTI 自证）",
  "0x45FE80": """// 对象注册（场景对象表插入）—— 分发器 case 0x15/0x16 与
// ui_icon_perf 0x4600B0 的注册链入口；参数 (容器, 源表, a2, a3, 对象, 0, 0)""",
}

def find(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]):
            return i
    return -1

SEP = "// ------------------------------------------------------------"

assert lines[0].startswith("// ====")
hdr_end = next(i for i in range(1, 15) if lines[i].startswith("// ===="))
lines[0:hdr_end+1] = new_header.split("\n")

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
print("OK worldmap_reward: %d sections, %d comment lines" % (n_sec, n_fun))
