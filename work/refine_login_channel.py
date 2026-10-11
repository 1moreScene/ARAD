# -*- coding: utf-8 -*-
# L1 精修脚本：login_channel.cpp —— 只加注释/分区横幅，函数体零改动
import io

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\game\login_channel.cpp"
lines = io.open(PATH, encoding="utf-8").read().split("\n")

new_header = """// ============================================================
// 【40JP逆向源代码工程】game/login_channel.cpp
// 模块: 登录/频道连接 CNEntranceChannel + 选区→选角→进副本→结算链路
// 来源: work/dump_combat.txt  区块: 0x470000-0x480000
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
// 函数数: 291
// ------------------------------------------------------------
// ★ 登录协议阶段图（自 connectServer 至 M_CHARACTER_SELECT 的完整链）：
//   阶段① 频道列表获取（HTTP）
//     CNEntranceChannel 的 HTTP 线程 0x46EB70"StartAddress"（见 ui_icon_perf.cpp，
//     "MERONG(0.9/;p)" 解密）→ 结果存 dword_98BA5C → 0x471CC0
//     checkHTTPChannelDataWork 轮询（超时 5000ms/失败自证）→ 成功调 0x471850。
//   0x471850 retrieveServerInfo（IDA 误名 money_put::_Putmfld，日志串自证真名）：
//     解析 "user_age#server_count|…"（分隔符 ;|#,,）→ 填 6 服务器型 × 5 频道型 ×
//     N 频道条目（104 字节/条：名字/负载 cur/max/满员旗标>95%/地址/端口），
//     打印 "** server_count = %d  user_age = %d"。
//   阶段② 选区界面（CNEntranceChannel__vmt4 = 0x471280 帧 tick）
//     进图一次性 M_CHARACTER_SELECT → 服务器页签×6(BUTTON_04)/频道型×5(BUTTON_03)
//     /翻页(CLICK_BUTTON1) → 频道单击(BUTTON_02)或双击(timeGetTime 差 < +1744 阈值)
//     → 调 connectServer。
//   阶段③ 连接频道（connectServer = 0x470640，日志串自证）
//     校验 server_type<6 / channel_type<5 / index 有效 → 连接历史存 +74/+76/+78
//     （channel_type 4 除外）→ 组登录串 "%d?%s?%d?%s?%s" → sub_488690 切分 →
//     sub_41E4F0 发出（发包 API 0x423200/423380 见 src/nengine/net_socket.cpp）；
//     测试账号白名单 dnftest299/300、testman412/425 → dword_8F7A24+244=2（"TN"）。
//   阶段④ 预约/重连：0x46EAA0 reserveConnection 预约 → 0x46E360
//     startTimerForReservedConnectChannel（500ms tick）→ 0x470A80
//     processReservedConnection 自动重连。
//   阶段⑤ 选角：CNEntranceModule__vmt4（0x46C300，见 ui_icon_perf.cpp）+
//     键盘导航 0x46BD70（SELECT_CHARACTER/CLICK_BUTTON4）。
// ★ 其余链路：副本创建（DUNGEON create FAIL 自证）→ 进副本一次性演出（0x475810
//   巨型，docs/13 遗留#3）→ 结算（RESULT_* 事件 + CNRDResult__vmt0）→
//   社交管理器（Friend/Member/Guild/Buddy）→ 触发条件评估器（0x47BBE0）。
// 交叉引用：频道表条目操作 0x46ECC0/0x46EF10/0x46E6B0 等在 ui_icon_perf.cpp 尾部；
//   发包 API 0x423200/0x423380 已 L1 精修于 src/nengine/net_socket.cpp（docs/13 §5）
// ============================================================"""

sections = {
  "0x470330": """// ============================================================
// 【分区 1】CNEntranceChannel 频道连接核心（0x470330-0x471280）
// 频道表容器（104 字节/条目的定长搬运 vector，容量上限 41297762 条）→
// connectServer（协议阶段③）→ processReservedConnection（阶段④ 重连）。
// ============================================================""",
  "0x471280": """// ============================================================
// 【分区 2】选区界面帧处理（0x471280-0x471850）
// CNEntranceChannel__vmt4 = 选区界面帧 tick（协议阶段②）。
// ============================================================""",
  "0x471850": """// ============================================================
// 【分区 3】频道列表获取（0x471850-0x471DB0）
// retrieveServerInfo（协议阶段① 数据落表）+ checkHTTPChannelDataWork（线程轮询）。
// ============================================================""",
  "0x471DB0": """// ============================================================
// 【分区 4】登录过场/数据装载（0x471DB0-0x475510）
// MonoLayer 渐隐过场（0x472AF0-0x472B80）、物品数据装载（"ITEM CREATE FAIL!"
// 自证 0x4730A0/0x4730F0）、红黑树/容器辅助（0x473440/0x473920 族）【推断】。
// ============================================================""",
  "0x475510": """// ============================================================
// 【分区 5】进副本一次性演出（0x475510-0x476300）
// 巨型 0x475810（docs/13 遗留#3：进副本震屏 85/100 + 物件生成，半还原范围）。
// ============================================================""",
  "0x476300": """// ============================================================
// 【分区 6】副本创建（0x476300-0x477060）
// "DUNGEON create FAIL! FILENAME[%s]" / "INDEX %d" 自证（0x476600/0x476AF0）；
// 选区角色数据查找 0x476B60（座位表 sub_453DD0 所用）。
// ============================================================""",
  "0x477060": """// ============================================================
// 【分区 7】结算界面（0x477060-0x478C70）
// 事件自证：RESULT_FLASH / M_RESULT / RESULT_CHARACTER / RESULT_SCORE_UP_LOOP；
// 结算主类 CNRDResult__vmt0 = 0x477B60。
// ============================================================""",
  "0x478C70": """// ============================================================
// 【分区 8】社交管理器 I（0x478C70-0x47B000）
// CNRDFriendManager__vmt4/vmt8、CNRDMemberManager__vmt4/vmt8（RTTI 自证）。
// ============================================================""",
  "0x47B000": """// ============================================================
// 【分区 9】社交管理器 II（0x47B000-0x47B9E0）
// CNRDGuildManager/CNRDMemberManager/CNRDBuddyManager 各 vmt 槽（RTTI 自证）。
// ============================================================""",
  "0x47B9E0": """// ============================================================
// 【分区 10】触发条件评估器（0x47B9E0-0x47C2F0）
// 评估器 0x47BBE0：八类条件 case 0-7（0=计时器 / 2=比例% / 3=加载进度 sub_477260×100/
// sub_43B5C0 / 4=旗标 / 5=座位比对 / 6=恒真 / 7=进度值）——不满足即 sub_47B8D0 关断。
// ============================================================""",
  "0x47C2F0": """// ============================================================
// 【分区 11】物品/封印数据处理（0x47C2F0-0x47E720）
// 44 字节/条目的批量数据处理；SEAL_ON/SEAL_OFF 事件（0x47E000 附近）【推断】。
// ============================================================""",
  "0x47E720": """// ============================================================
// 【分区 12】技能数据（0x47E720-0x480000）
// CNRDSkill__vmt4（RTTI 自证）与技能条目读写。
// ============================================================""",
}

funcs = {
  "0x470330": """// 频道表 vector 定点插入：104 字节/条目（qmemcpy 0x68）；尾部插入 / 中部
// 腾挪（sub_46ECC0/46E6B0）/ 扩容重建（operator new + free）三路；
// 41297762 与 1321528399 = 扩容运算魔数（×4 溢出等价）""",
  "0x4705D0": "// 频道表追加一条并返回新条目指针（配合 0x470330 定点插入）",
  "0x470640": """// CNEntranceChannel::connectServer —— 协议阶段③ 连接频道（日志串自证）：
// 1) 校验 server_type<6 / channel_type<5 / 频道索引在表内（无效即日志返回 -1）
// 2) channel_type!=4 时保存连接历史 +74/+76/+78（"connection history saved"）
// 3) 条目 +88 满员旗标为 1 → 拒绝
// 4) byte_99FCD1 ← 条目首字节；sub_46E230 取年龄档 → sprintf("%d?%s?%d?%s?%s")
//    登录串 → sub_488690 按 '?' 切分 → sub_41E4F0 发出登录请求
// 5) sub_48B7C0(13/14/15/16) 置请求参数；135 号串提示（+79==-1 时）
// 6) 返回值解析 atol → dword_8F7A24+244（==2 → sub_419CF0("TN")）
// 7) 测试账号白名单 dnftest299/300、testman412/425（tolower 比对）→ 同置 "TN"
//    （docs/13 §6：开发期测试账号残留）""",
  "0x470A80": """// CNEntranceChannel::processReservedConnection（日志串自证）—— 协议阶段④：
// 预约三元组(+300/+308/+316)有效 → 打印目标频道名/端口 → 强制选中界面按钮态 →
// connectServer 重连 → 清预约。由 vmt4 的 500ms 定时器（0x46E360 启动）触发""",
  "0x470B20": "// 频道条目入表（0x4705D0 包装：容量够则原地 +104）",
  "0x471280": """// CNEntranceChannel::vmt4 —— 选区界面帧 tick（协议阶段②）：
// 主帧循环 sub_433690 → 预约定时器到点则 processReservedConnection →
// 一次性 M_CHARACTER_SELECT（+1460）→ 各按钮命中(dword_79BE34 焦点窗口)：
// 服务器页签×6（BUTTON_04，+64 起）/频道型×5（BUTTON_03，+92 起）/
// 频道翻页（CLICK_BUTTON1，+172/-1 页与 +176/+1 页，页容量 10）/频道单击
// （BUTTON_02 → channel_type 2/3 直连或按高亮列）→ 双击检测（+1728/+1732 记槽位、
// +1736/+1740 timeGetTime，差 < +1744 阈值即 connect）→ connectServer(server, type, idx)""",
  "0x471850": """// CNEntranceChannel::retrieveServerInfo —— 协议阶段① 频道列表落表。
// ★ IDA 误名 money_put::_Putmfld（相邻 CRT 符号）；真名由日志串自证
//   "CNEntranceChannel::retrieveServerInfo - refresh channel information start"。
// 流程：清空 6×5 频道表 → 分词器(sub_577850/577310，分隔符 ";" "|" "#" ",") →
//   首字段 user_age（+72）/ server_count → 按服务器名匹配 off_790590[0..5] →
//   每频道 5 字段：名字(→sub_46E260 抽数字 id)/类型槽 sub_568A70/负载 cur/max
//   （max×100/cur>95 → 满员旗标 +22）/地址串/端口 → sub_470B20 入表并打日志
//   "'%s' '%s' '%d' %s:%d %d/%d"；异常名 "abnormal server name"；
//   收尾 sub_4F8FE0("Etc/pvpChannelList.etc") 加载 PVP 频道表""",
  "0x471CC0": """// CNEntranceChannel::checkHTTPChannelDataWork（日志串自证）—— 阶段① 线程轮询：
// byte_98BB64 线程在跑 → GetExitCodeThread：259=STILL_ACTIVE（超 5000ms 强收）、
// 1=成功 → retrieveServerInfo(dword_98BA5C 数据) 并 sub_46E810 刷新界面、
// -2=失败（"the http thread failed"）→ 重启线程 sub_46EE10""",
  "0x471DB0": "// 【推断】登录数据装载巨函数（~310 行，与物品/条目装载同族）",
  "0x472AF0": "// 登录过场层装载：dword_8F84B8 vt+60 载入 \"Common/Etc/MonoLayer.img\" → sub_418DD0 建层",
  "0x472B30": "// 过场渐隐控制：byte_99FB90 使能 + dword_99FB94 起始时刻",
  "0x472B80": "// 过场渐隐绘制：3000ms 内 sub_4049C0 透明度插值，超时自关",
  "0x4730A0": "// 物品数据装载（\"ITEM CREATE FAIL! ItemIndex %d\" 自证）【推断】",
  "0x4730F0": "// 物品数据装载 II（同上自证串）【推断】",
  "0x473440": "// 红黑树/有序容器节点插入辅助（CRT 容器模板实体）",
  "0x473920": "// 红黑树/有序容器查找-插入辅助（CRT 容器模板实体）",
  "0x475810": """// ★ 巨型（~310 行，userpurge）—— 进副本一次性演出，docs/13 遗留#3 课题：
// 震屏 85/100 触发 + 副本物件批量生成（"ITEM CREATE FAIL!" 族日志可见）；
// 半还原范围（前半主干），完整逻辑待 P5 专项""",
  "0x476600": """// 副本创建（按文件名）—— "DUNGEON create FAIL! FILENAME[%s]" 自证：
// dword_9B0BB8 表查词条 → sub_476600 主干：障碍/事件列表展开（9 字节/项）、
// 464 偏移状态、dword_99FB70 清零、+123=-1""",
  "0x476AF0": "// 副本创建（按索引）—— \"DUNGEON create FAIL! INDEX %d\" 自证；词条取后转 0x476600",
  "0x476B60": "// 选区角色数据查找（座位表 sub_453DD0 所用；+80/+84 名字串）",
  "0x477060": "// 结算事件：RESULT_FLASH（6195 行附近）【推断：闪光演出】",
  "0x4772A0": "// 结算事件：M_RESULT 打开结算界面（6339 行附近）",
  "0x477260": "// 加载进度查询（触发条件 case 3 所用：sub_477260×100/sub_43B5C0）",
  "0x477B60": """// CNRDResult::vmt0 —— 结算主类帧处理（RTTI 自证）：
// RESULT_CHARACTER + RESULT_SCORE_UP_LOOP(33311) 事件序列（6899 行附近）""",
  "0x479340": "// CNRDFriendManager::vmt4 好友管理帧处理（RTTI 自证）",
  "0x4793D0": "// CNRDFriendManager::vmt8（RTTI 自证）",
  "0x479430": "// CNRDMemberManager::vmt4 会员管理帧处理（RTTI 自证）",
  "0x479490": "// CNRDMemberManager::vmt8（RTTI 自证）",
  "0x47AAF0": "// CNRDGuildManager::vmtC（RTTI 自证）",
  "0x47ACB0": "// CNRDMemberManager::vmtC（RTTI 自证）",
  "0x47AF40": "// CNRDBuddyManager::vmt0（RTTI 自证）",
  "0x47AF70": "// CNRDGuildManager::vmt0（RTTI 自证）",
  "0x47AFA0": "// CNRDMemberManager::vmt0（RTTI 自证）",
  "0x47BBE0": """// 触发条件评估器：遍历条件对象（+68 表，20 字节/项），按 v9[69] 型别：
// 0=计时器（dword_8F8520 vt+120 时钟 +164）/ 1=|Y| 阈值（dword_99FD58+948）/
// 2=百分比（+906/+920 ×100）/ 3=加载进度（sub_477260×100/sub_43B5C0）/
// 4=旗标（+120 非零）/ 5=座位号比对 sub_456390 / 6=恒真 / 7=进度值 sub_495C80；
// 不满足 → sub_47B8D0(对象, 0) 关断""",
  "0x47E720": "// 技能条目查找/换算（配合 0x47EFD0 的 14 字节/项表）【推断】",
  "0x47EFD0": "// 技能表槽位设置（越界打 aCriticalError_8；+87 处 14 字节/项表）【推断】",
  "0x47F190": "// CNRDSkill::vmt4 技能帧处理（RTTI 自证）",
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
print("OK login_channel: %d sections, %d comment lines" % (n_sec, n_fun))
