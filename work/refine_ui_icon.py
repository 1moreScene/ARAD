# -*- coding: utf-8 -*-
# L1 精修脚本：ui_icon_perf.cpp —— 只加注释/分区横幅，函数体零改动
import io

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\game\ui_icon_perf.cpp"
lines = io.open(PATH, encoding="utf-8").read().split("\n")

new_header = """// ============================================================
// 【40JP逆向源代码工程】game/ui_icon_perf.cpp
// 模块: UI 图标/快捷栏/性能统计/选角界面/CNEntranceChannel 助手
// 来源: work/dump_combat.txt  区块: 0x460000-0x470000
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
// 函数数: 156
// ------------------------------------------------------------
// ★ 性能统计系统归类（统计点全表）：
//   初始化  0x460590 —— 动态解析 psapi.dll!GetProcessMemoryInfo 与
//             Kernel32!GetThreadTimes（内存/线程时间采样入口）
//   重置    0x4605F0 —— 统计对象清零（12 槽 + 计数族，+568=-1 哨兵）
//   采样    sub_405030(n)/sub_4050B0(n)（引擎统计查询，n=槽号）
//   HUD     0x461840 —— 每 1000ms 重采样并逐行绘制到 dword_79C0B0 vt+80：
//             "fps : %03d"（=1000000/微秒帧时长 sub_405030(2)，y=163）
//             "texture size : %0.3f MB"（sub_4050B0(1)×0.00000095367432=1/1048576，y=176）
//             "cached chars : %d"（sub_4050B0(2)，y=189）
//             "traversal : %d usec"（sub_405030(9)，y=202）
// ★ UI 事件锚点（sub_432EA0 事件名 = 字符串自证）：
//   ICON_DROP_TO_QUICKSLOT（0x4633D0 快捷栏拖放）/ SELECT_CHARACTER（0x46BD70
//   选角键盘导航）/ SCROLL_ARROW（CNEntranceModule__vmt4 内滚动）/ CLICK_BUTTON2/4
// ★ 登录链路衔接（docs/13 §6；协议阶段图见 login_channel.cpp 文件头）：
//   CNEntranceModule__vmt4 = 0x46C300 选角模块帧（阶段⑤）；
//   CNEntranceChannel 复位 0x46DE70（+12=6/+16=5 与 connectServer 校验上限一致）；
//   vmt8 = 0x46E1F0 离开清理；reserveConnection = 0x46EAA0（阶段④ 预约）；
//   startTimerForReservedConnectChannel = 0x46E360（500ms tick）；
//   HTTP 频道列表线程 = 0x46EB70 "StartAddress"（"MERONG(0.9/;p)" 校验，
//   结果 dword_98BA5C → retrieveServerInfo 的输入；阶段① 数据源）
// 交叉引用：频道表条目搬运族（0x46ECC0/0x46EF10/0x46E6B0 等）被
//   login_channel.cpp 0x470330 调用；对象注册链 0x4600B0 → worldmap_reward 0x45FE80
// ============================================================"""

sections = {
  "0x460070": """// ============================================================
// 【分区 1】图标容器与对象注册辅助（0x460070-0x460590）
// 批量对象注册 0x4600B0（→ worldmap_reward 0x45FE80 注册链）。
// ============================================================""",
  "0x460590": """// ============================================================
// 【分区 2】性能统计系统 + 早期图标绘制（0x460590-0x4622A0）
// 统计点归类见文件头；0x460830-0x460DC0 为穿插其中的图标绘制
// （quickslot 布局），0x461C20/0x461F50 为采样/计时辅助【推断】。
// ============================================================""",
  "0x4622A0": """// ============================================================
// 【分区 3】快捷栏与图标拖放（0x4622A0-0x463C90）
// ICON_DROP_TO_QUICKSLOT 自证（0x4633D0）。
// ============================================================""",
  "0x463C90": """// ============================================================
// 【分区 4】界面窗口族（0x463C90-0x469170）
// 图标格/窗口绘制与命中测试（含 ~890 行巨函数 0x464C60）【推断】。
// ============================================================""",
  "0x469170": """// ============================================================
// 【分区 5】CNRDBaseInterface（0x469170-0x469DD0）
// RTTI 自证基接口构造与查询。
// ============================================================""",
  "0x469DD0": """// ============================================================
// 【分区 6】界面管理器辅助（0x469DD0-0x46ABB0）
// 深拷贝/容器克隆类操作（~630 行）【推断】。
// ============================================================""",
  "0x46ABB0": """// ============================================================
// 【分区 7】选角界面辅助（0x46ABB0-0x46BCB0）
// 光标/条目状态 0x46B0E0、条目计数 0x46B060 等。
// ============================================================""",
  "0x46BCB0": """// ============================================================
// 【分区 8】选角输入（0x46BCB0-0x46C300）
// 键盘导航 0x46BD70（SELECT_CHARACTER/CLICK_BUTTON4 自证）。
// ============================================================""",
  "0x46C300": """// ============================================================
// 【分区 9】CNEntranceModule 选角模块（0x46C300-0x46CB70）
// vmt4 = 选角界面帧（登录协议阶段⑤，见文件头）。
// ============================================================""",
  "0x46CB70": """// ============================================================
// 【分区 10】界面杂项（0x46CB70-0x46DE70）
// ============================================================""",
  "0x46DE70": """// ============================================================
// 【分区 11】CNEntranceChannel 复位与助手（0x46DE70-0x46EB70）
// 复位（+12=6/+16=5）/ vmt8 离开清理 / 预约连接计时器 / reserveConnection。
// ============================================================""",
  "0x46EB70": """// ============================================================
// 【分区 12】HTTP 频道列表线程与频道表 vector（0x46EB70-0x46F800）
// StartAddress（"MERONG(0.9/;p)" 校验）→ 0x46EE10 线程启动包装 →
// 104 字节/条目的搬运族（0x46ECC0/0x46EF10/0x46E6B0/0x46ED50/0x46F6B0，
// 被 login_channel.cpp 0x470330 调用）。
// ============================================================""",
  "0x46F800": """// ============================================================
// 【分区 13】nengine 单选/复选按钮（0x46F800-0x470000）
// nengine__CNRadioButton__vmt0 / nengine__CNCheckBox__vmt0（RTTI 自证）。
// ============================================================""",
}

funcs = {
  "0x460070": "// 图标 id 查容器槽（命中返回槽指针，未中尾插 sub_431230）",
  "0x4600B0": "// 批量对象注册：源表逐 id 调 worldmap_reward 0x45FE80 入场景表",
  "0x460590": """// 性能统计初始化：GetProcessMemoryInfo==0 时 LoadLibraryA("psapi.dll") 动态解析；
// dword_98B754 空时解析 Kernel32!GetThreadTimes —— 内存/线程时间采样入口""",
  "0x4605F0": "// 统计对象清零：两组 6 槽 + 计数族（+100..+116、+204..+228），+568=-1 哨兵",
  "0x460920": "// 图标绘制（quickslot 主体，477-a4 底边布局，双态 sub_418DD0 取图）【推断】",
  "0x461840": """// ★ 性能统计 HUD 绘制（每 1000ms 经 +358 时钟重采样）：
// fps = 1000000/帧时长 usec（sub_405030(2)）；纹理 MB = sub_4050B0(1)×0.00000095367432
// （1/1048576）；cached chars = sub_4050B0(2)；traversal = sub_405030(9) usec；
// 四行 sprintf → dword_79C0B0 vt+80 绘制（y=163/176/189/202）；"Main" 标签""",
  "0x4633D0": """// ★ 快捷栏拖放 —— ICON_DROP_TO_QUICKSLOT 自证（2939/2977 行附近）：
// 拖拽源/目标格命中 → 物品换位/落格【推断】""",
  "0x469170": "// CNRDBaseInterface::vmt0 —— 基接口构造（RTTI 自证）",
  "0x46BD70": """// 选角键盘导航：VK_LEFT(37)/VK_UP(38)/VK_RIGHT(39)/VK_DOWN(40)，
// 左右 ±1、上下 ±4（4 列网格）→ SELECT_CHARACTER 音效 + 0x46B0E0 置光标；
// Enter(13) → sub_46BCB0 确认 + CLICK_BUTTON4；ESC 态经 0x44BCC0 关闭""",
  "0x46C300": """// CNEntranceModule::vmt4 —— 选角界面帧（登录协议阶段⑤）：
// 主帧循环 → 退出确认(vt+20(10)/vt+16(84) 同 docs/13 §1) → 0x46BD70 键盘 →
// 一次性 M_CHARACTER_SELECT(+50) → sub_45DDA0(dword_8F8524,1) UDP 上报(+49) →
// 角色按钮（CLICK_BUTTON2，选中板 30 号/名字板 0x1E 号 dword_8F8528 vt）""",
  "0x46DE70": """// CNEntranceChannel 复位：+12=6（服务器型数上限）/+16=5（频道型数上限）——
// 与 connectServer 校验阈值一致；其余状态/页签槽清零""",
  "0x46E1F0": "// CNEntranceChannel::vmt8 —— 离开选区清理：声音通道停（dword_79C0B0 vt+84/88）+ sub_69AB00",
  "0x46E360": """// CNEntranceChannel::startTimerForReservedConnectChannel（日志串自证）——
// 阶段④：+332 使能 → sub_698F90(+320, 500ms, 0) 定时；vmt4 到点调
// processReservedConnection(0x470A80)""",
  "0x46EAA0": """// CNEntranceChannel::reserveConnection（日志串自证）—— 阶段④ 预约：
// 校验同 connectServer（server<6/channel<5/index 表内）→ 预约三元组 +75/+77/+79""",
  "0x46EB70": """// HTTP 频道列表下载线程入口（IDA 名 StartAddress）—— 阶段① 数据源：
// 临界区 stru_98BA28 → sub_5770B0 缓冲 → "MERONG(0.9/;p)" 口令校验
// （sub_576F10）→ 解密结果指针存 dword_98BA5C（retrieveServerInfo 的输入）；
// 退出码 1=成功 / -2=失败（checkHTTPChannelDataWork 0x471CC0 轮询）""",
  "0x46EE10": "// HTTP 线程启动包装（CreateThread → StartAddress；句柄存 hObject）",
  "0x46ECC0": "// 频道表 104 字节/条目逆向搬运（login_channel 0x470330 中插腾挪用）",
  "0x46F800": "// nengine::CNRadioButton::vmt0（RTTI 自证）",
  "0x46F910": "// nengine::CNCheckBox::vmt0（RTTI 自证）",
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
print("OK ui_icon_perf: %d sections, %d comment lines" % (n_sec, n_fun))
