# -*- coding: utf-8 -*-
"""
L1 精修注入器 —— src/game/rtti_core.cpp (0x4A0000-0x4B0000, 531 函数)
只加注释，不改函数体逻辑。
产出: 头注释【已精修】交叉引用 + RTTI 槽位名 + 分区块 + 重点函数解读。
"""
import re, io, sys

SRC = r'E:\DOF LX\【40JP逆向源代码工程】\src\game\rtti_core.cpp'
COREVT = r'E:\DOF LX\【40JP逆向源代码工程】\work\decomp_core_vt.txt'
if len(sys.argv) > 1:
    SRC = sys.argv[1]

# ---------------------------------------------------------------
# 1. RTTI 槽位表（来自 work/decomp_core_vt.txt 的 name:/slots: 栏）
# ---------------------------------------------------------------
rtti = {}   # va -> (name, slots)
cur = None
for l in open(COREVT, 'r', encoding='utf-8', errors='replace').read().split('\n'):
    m = re.match(r'^// (00[0-9A-F]{6})  slots: (.+)$', l)
    if m:
        cur = (m.group(1), m.group(2)); continue
    m2 = re.match(r'^// name: (\S+)', l)
    if m2 and cur:
        rtti['0x' + cur[0][2:]] = (m2.group(1), cur[1]); cur = None

# ---------------------------------------------------------------
# 2. 已精修交叉引用表（VA -> (文件, 槽位/函数名, 一句话)）
#    来源: src/cnrd/*.cpp / *.h 与 docs/14、docs/15
# ---------------------------------------------------------------
REFINED = {
 '0x4A0C80': ('src/cnrd/IRDActiveObject.cpp', 'vmt2A0', '子对象基址 getter（1364 字节步长，this+1364*a2+676）'),
 '0x4A0DC0': ('src/cnrd/IRDCollisionObject.h', 'vmt1B8', '包围盒左界 = 中心x − 宽/2（AABB 边界族 1B8/1BC/1C0/1C4 之首）'),
 '0x4A0DE0': ('src/cnrd/IRDCollisionObject.h', 'vmt1BC [族级]', '包围盒右界 = 宽 + vt+440（cnrd 头以 1B8 "1B8/1BC/1C0/1C4 包围盒边界计算族" 一行覆盖）'),
 '0x4A0E00': ('src/cnrd/IRDCollisionObject.h', 'vmt1C0 [族级]', '包围盒上界 = 中心y − 高/2（cnrd 头同上族级覆盖）'),
 '0x4A0E20': ('src/cnrd/IRDCollisionObject.h', 'vmt1C4 [族级]', '包围盒下界 = 高 + vt+448（cnrd 头同上族级覆盖）'),
 '0x4A0F00': ('src/cnrd/IRDActiveObject.cpp', 'vmt2AC', '属性块基址（this+6132，HP 读处）'),
 '0x4A0F10': ('src/cnrd/IRDActiveObject.cpp', 'vmt2B0', '附加块基址 this+6160'),
 '0x4A0F20': ('src/cnrd/IRDActiveObject.cpp', 'vmt2BC', '读 +6296 槽'),
 '0x4A0F30': ('src/cnrd/IRDActiveObject.cpp', 'vmt184', '属性块（vt+672(3)）首 DWORD'),
 '0x4A0F40': ('src/cnrd/IRDActiveObject.cpp', 'vmt2D4', '读 +6552 霸体字节'),
 '0x4A0F50': ('src/cnrd/IRDActiveObject.h', 'vmt304', '17 状态槽数组访问（+6228+4*a2）'),
 '0x4A0F70': ('src/cnrd/IRDActiveObject.h', 'vmt318', '18 观察者槽数组访问（+6468+4*a2）'),
 '0x4A0F90': ('src/cnrd/IRDActiveObject.cpp', 'vmt330', '转发 vt+808(a2, 0xFFFF, 0, a3)'),
 '0x4A0FB0': ('src/cnrd/IRDActiveObject.cpp', 'vmt334', '写 +6740 字节'),
 '0x4A0FC0': ('src/cnrd/IRDActiveObject.h', 'vmt1C8', '命中列表追加（vt+676(0) 联动 + 基类）'),
 '0x4A0FF0': ('src/cnrd/IRDActiveObject.h', 'vmt1CC', '命中列表移除（IRDCollisionObject_vmt1CC 基类实现 + vt+676(0) 联动；0x4A0FC0=1C8 追加）'),
 '0x4A1010': ('src/cnrd/IRDActiveObject.h', 'vmt348', '大状态 getter（this+6556）'),
 '0x4A1020': ('src/cnrd/IRDActiveObject.h', 'vmt34C', '内嵌动作子对象 this+6560'),
 '0x4A1030': ('src/cnrd/IRDActiveObject.cpp', 'vmt350', '冻结标志写入（+6744，0→-1 / 非0→0）'),
 '0x4A1050': ('src/cnrd/IRDActiveObject.cpp', 'vmt354', '冻结标志 != -1 查询'),
 '0x4A10D0': ('src/cnrd/IRDActiveObject.cpp', 'vmt3DC', '非霸体自由条件（vt+724==0 且状态3/8==0）'),
 '0x4A1110': ('src/cnrd/IRDActiveObject.cpp', 'vmt3E4', '复活计时器清零（写 -1）'),
 '0x4A1120': ('src/cnrd/IRDActiveObject.cpp', 'vmt3E8', '复活计时中判定（!= -1）'),
 '0x4A1130': ('src/cnrd/IRDActiveObject.cpp', 'vmt3F0', '复活已计时（clockAux 域差值）'),
 '0x4A1CA0': ('src/cnrd/IRDActiveObject.cpp', 'vmt188', 'HP 类属性读取（属性块 +60 防篡读 × 浮点壳 +74）'),
 '0x4A1CE0': ('src/cnrd/IRDActiveObject.cpp', 'vmt18C', 'MP 类属性读取（+1012 × 浮点壳 +1026）'),
 '0x4A1D20': ('src/cnrd/IRDActiveObject.cpp', 'vmt190', '整数属性读取（+1306）'),
 '0x4A1D40': ('src/cnrd/IRDActiveObject.cpp', 'vmt194', '整数属性读取（+1124）'),
 '0x4A1E20': ('src/cnrd/IRDActiveObject.h', 'vmt2E0', '属性块首值（GetHP 族，此处索引）'),
 '0x4A1E30': ('src/cnrd/IRDActiveObject.h', 'vmt2E4', '属性块 +4 浮点壳（GetHP 族）'),
 '0x4A2460': ('src/cnrd/IRDCollisionObject.h', 'vmt29C', '研究族槽位（0x4A2480 转发壳）'),
 '0x4A2B80': ('src/cnrd/IRDActiveObject.cpp', 'vmt270', '基类检查 + 附加条件（+7093 == 0）'),
 '0x4A2BB0': ('src/cnrd/IRDCollisionObject.cpp', 'vmt270', '攻击使能检查（vt+608 + 动画帧率态 != 2）'),
 '0x4A2D30': ('src/cnrd/IRDCollisionObject.h', 'vmt180', '槽数组 getter（+348 起，越界返回 0）'),
 '0x4A2D70': ('src/cnrd/IRDActiveObject.cpp', 'vmt25C', '受击盒使能 setter（+641）+ 附属物 vt+604 广播'),
 '0x4A2DE0': ('src/cnrd/IRDCollisionObject.cpp', 'vmt160', '命中列表双链表清空'),
 '0x4A3900': ('src/cnrd/IRDCollisionObject.h', 'vmt164', '记录当前命中目标（研究已定案）'),
 '0x4A4570': ('src/cnrd/IRDCollisionObject.cpp', 'vmt288', '写入 100（格挡/减伤基础值槽）'),
 '0x4A4590': ('src/cnrd/IRDCollisionObject.h', 'vmt0 实体', 'IRDCollisionObject 析构本体（0x4A4710 vmt0 转发至此）'),
 '0x4A4710': ('src/cnrd/IRDCollisionObject.h', 'vmt0', 'IRDCollisionObject 析构 → 0x4A4590'),
}

# ---------------------------------------------------------------
# 3. 未覆盖重要函数的手写解读（VA -> 注释块行列表）
# ---------------------------------------------------------------
NOTES = {}

def note(va, lines):
    NOTES[va] = lines

# ---- 0x4A00xx-0x4A0Bxx 属性/渲染值同步族 ----
note('0x4A0180', [
 '【推断】属性同步主写器（IDA 原名 sub_4A0180）—— 单入口批量"脏位同步"。',
 'a2 是 32 位脏位掩码，每 2 位控制一个属性字段的三种写模式:',
 '  00 → 向会话管理器(dword_8F8520.vt+88)请求服务端值回填（拉取）',
 '  01 → 写 0（清零）  10/11 → 从浅拷贝 this+4.. 复制（本地域）',
 '字段布局: this+53 起 49 字节内串行排布；首段 a2&0x10 → 整块 49 字节拉取。',
 '【待验证】这是防篡改属性管线的"批量提交侧"，具体字段名待属性块专项。',
])
note('0x4A0550', [
 '【推断】重力/抛物线速度写入器。this+36 保存重力系数（int），',
 'a2 = 施加系数；v5 = a2*grav 为负时取 v6 = sqrt(-2*v5)（v = sqrt(2gh)），',
 '写动画对象 this+24（浮点）+ 脏标记 +196。a2<0 时取负根 —— 上行/下坠两向。',
 '【待验证】用途可能是击退抛物线的初速计算。',
])
note('0x4A05C0', [
 '【推断】零点过冲补偿。this+36 为负且 a2 转正时，计算 -a2*v2*0.001',
 '并交 sub_494F20(this, 2, v3) —— 与 vmt37C 的方向速率写入族同构。',
])
note('0x4A0600', [
 '【推断】速率零交叉修正（vmt37C 的伴侣）。this[a2+7] 与浮点 this[a2+4] 异号时，',
 '把 this[a2+10] 写为 浮点值×-1000/系数 —— 千分位换算 + 方向翻转。',
])
note('0x4A0650', [
 '【推断】坐标轴分量读取（a2 = 0/1/2）。this+104 为"控制对象"指针，',
 '存在且 +108 标志非 0 时走其 vt+40 接口，否则回退 sub_432F40(本地浮点)。',
 '与 0x4A0790 的 v10/v17/v20 读法完全一致 —— 同一族。',
])
note('0x4A0680', [
 '【推断】暂停/继续总开关。this+172 为现状字节；变化时冻结/解冻 this+144 时钟',
 '（sub_699030 / sub_698FE0）并向下游 this+104 对象转发 vt+56(a2) 广播。',
 '被 CNRDObject_vmtE4（0x49A320）用作"子对象全冻"的施加点。',
])
note('0x4A06D0', [
 '【推断】绑定目标对象并记录相对偏移。this+128 = 目标指针；',
 'this+132/136/140 = 本坐标 − 目标坐标（x/y/z 三条）。',
 '与 0x4A0790（每帧用偏移重算世界坐标）配对 —— 是"挂点/跟随"机制。',
])
note('0x4A0740', [
 '【推断】时间域变更深层递归（CNRDObject_vmt48 尾部调用的正是本函数）。',
 'sub_699070(a2) 换域后，对 this+104 子对象再调其 vt+68 —— 递归到叶子。',
])
note('0x4A0770', [
 '【推断】设置"宿主/所有者"指针并联动 sub_49FD20（动画归属重绑）。',
])
note('0x4A0790', [
 '【推断】★ 挂点对象每帧世界坐标刷新 —— 三段式:',
 ' ① 若 this+172（暂停标志）为 0: 用 this+32 目标坐标 + 偏移(33/34/35) 重算',
 '    this+4/+8/+12 浮点世界坐标，并置脏标记 this+124；',
 ' ② 对 3 个轴分量调 sub_49FD80（逐轴深化）；',
 ' ③ 当 this+26 子对象有效且 this+108 使能: 若 (this+28==2 且 this+116 非 0) 走',
 '    vt+532 快捷分支；否则逐轴 vt+40 读值 → vt+16 写入 + 置脏 → vt+8 提交。',
 '条件分支里再读 this+116/this+28 决定二次写轴 —— 这是"挂件跟随 + 延迟提交"循环。',
])
note('0x4A0B60', [
 '【推断】控制对象换绑（0x4A0650/0x4A0790 族的核心）。',
 '① 先用旧对象 this+104 逐轴 vt+40 读出 v15/v14/v12 三个分量；',
 '② 旧对象非空时调 (**v9)(v9,1) 一次（推测为"释放/引用减"）；',
 '③ 换写 this+104 = a2；a2 非空 → 向新对象 vt+68(this+152) 传参、逐轴 vt+16 读入',
 '   本次坐标、清除短暂标志（+116/+117..+119/+120）并 vt+8 提交，返回 0；',
 '④ a2 == 0（解绑）→ 把先前读出的三分量回写本地浮点 + 置脏 —— 保留最后位置。',
])
note('0x4A0CB0', [
 '【推断】CNGoblinCommon 构造函数残留桩（写 vftable；a2&1 → free）。',
 'IDA 符号里 RTTI 名在本文件缺失，按 vftable 归属推断为哥布林通用基类。',
])
note('0x4A0CE0', [
 '【推断】CNMonsterFriendShip vftable 写入桩（友方怪物基类析构/构造残片）。',
])
note('0x4A0CF0', [
 '【推断】CNGhoulOilSellerOlrek 构造（食尸鬼油商 Olrek）。',
 '先调 sub_58E0E0 基类构造 → 写 vftable → sub_58E3B0 收尾。',
])
note('0x4A0D70', [
 '【推断】写 this+608（vt+174 对偶槽）——攻击/动作资源句柄缓存。',
])
note('0x4A0EC0', [
 '【推断】"距上次事件毫秒"读取器。this+568 为上次时刻（-1 = 无记录 → 返回 1e9）。',
 '与 CNRDObject_vmtE4 的 +204/+208 时刻族同构（clockMain 域差值）。',
])
note('0x4A0EF0', [
 '【推断】读 this+572（最后一次受击/命中记录槽，见 IRDCollisionObject.h +572 注释）。',
])
# ---- 0x4A11xx-0x4A1Cxx 观察者/装备相关 ----
note('0x4A1150', [
 '【推断】返回 this+7232 —— IRDActiveObject 尾部另一内嵌子块基址。',
 '【待验证】7232 与 +7520（0x4A1160）成对，疑为 288 字节/双份的装备模式缓存区。',
])
note('0x4A1160', [
 '【推断】返回 this+7520 —— 与 0x4A1150 成对的内嵌块基址（0x4A11D0 析构其内容）。',
])
note('0x4A1170', [
 '【推断】读 this+7260（this+1815*4）。',
])
note('0x4A1180', [
 '【推断】读 this[2090]（+8360）指向对象的 +2 字节。2090*4 = +8360。',
 '【待验证】8360 接近 IRDActiveObject 尺寸尾段，疑为"配件/宠物"子对象。',
])
note('0x4A11A0', [
 '【推断】读 this[2090]（+8360）指向对象的 +3 DWORD —— 与 0x4A1180 同一对象的两字段。',
])
note('0x4A11C0', [
 '【推断】恒等函数（返回入参本身）—— 接口桩/适配器。',
])
note('0x4A11D0', [
 '【推断】CNGhoulOilSellerOlrek 析构（调 0x4A1200 + 可选 free）。',
])
note('0x4A1200', [
 '【推断】CNGhoulOilSellerOlrek vftable 重置 + sub_58E0D0 基类析构。',
])
note('0x4A1210', [
 '【推断】★ clong/cfloat 混合属性块的批量初始化序列（IDA 展开 100+ 次 sub_41A590）。',
 '模式: 每对 (偏移, 值) —— 先写 clong 槽再写 cfloat 槽（1065353216 = 1.0f 的位模式）。',
 '结构证据: +1/+18、+8/+46、+15/+74... 步长 14 字节 与 0x14 单位交错；',
 '内嵌 4 元素循环（a1+57 起，步长 14）与 16 元素循环（a1+127 起）——',
 '对应 0x4A1670 析构的 eh vector destructor iterator(...,0xEu,4/16,...)。',
 '初值 1000 出现在 +204/+211/+218（千分基准）与 100 在 +239 —— 属性默认值表。',
 '【待验证】这是 IRDActiveObject 属性块 (+6132 处, vt+672 返回) 的构造体（vmt10 初始化链）。',
])
note('0x4A1670', [
 '【推断】★ 属性块析构/重置（0x4A1210 的对偶）。逐槽 sub_41A630（解引用计数）',
 '后重写 vftable 为 clong/cfloat，最后 `eh vector destructor iterator` 清理 4 元素与',
 '16 元素两个内嵌数组 —— 数组元素大小 0xE(14) 字节，与 0x4A1210 的步长直证一致。',
])
note('0x4A1C30', [
 '【推断】双 clong 小构造（设 vftable + 初值 -1）—— 与 0x4A1CA0 同址族，',
 '疑为"HP 当前值/上限值"对（vmt188 读的就是 +60/+74 的这对）。',
])
note('0x4A1D60', [
 '【推断】属性块 +116 防篡读 × 浮点壳 +130 —— 与 0x4A1CA0/0x4A1CE0 同一族（vmt198）。',
])
note('0x4A1DA0', [
 '【推断】属性块 +1068 × 浮点壳 +1082（vmt19C 族 —— 与 MP 族 +1012/+1026 相隔 56B）。',
])
note('0x4A1DE0', [
 '【推断】属性块 +1334 整数读（vmt1A0）。vmt188 族步长 56 字节的规律: 60/116/1012/1068。',
])
note('0x4A1E00', [
 '【推断】属性块 +1166 整数读（vmt1A4）。',
])
note('0x4A1E70', [
 '【推断】属性块 +6146 防篡写封装的 setter（a4 非 0 时同时把 +6604 置 -1 使缓存失效）。',
])
note('0x4A1EB0', [
 '【推断】属性块 +32 × 浮点壳 +46（vmt2F4）。',
])
note('0x4A1EF0', [
 '【推断】读 this+7028 的 clong（vmt38C）—— 由 0x4A49B0 构造时以 sub_41A630+clong vftable 初始化。',
])
note('0x4A1F00', [
 '【推断】★ 属性块 62 槽全解构（sub_41A580 = clong/cfloat 析构单槽）。',
 '顺序 = 0x4A1670 构造的逆序，含 4/16 元素两个 eh vector 数组 ——',
 '这是 0x4A49B0（IRDActiveObject 构造）注册的析构回调（eh vector constructor iterator 的第 5 参）。',
])
note('0x4A22E0', [
 '【推断】双槽解构（+14 与 +0）—— 小块的 clong 对清理。',
])
note('0x4A2330', [
 '【推断】CNMagmaGhoul（熔岩食尸鬼）构造。基类 sub_58DFE0 → vftable → 内嵌块初始化。',
 '新发现字段: +8632 标志字节、+8628 IObject vftable、+8636 int、+8640 字节 =1。',
 'RTTI 名（vftable 直证）: CNMagmaGhoul。',
])
note('0x4A23D0', [
 '【推断】CNMagmaGhoul 析构（调 0x4A2400 + 可选 free）。',
])
note('0x4A2400', [
 '【推断】CNMagmaGhoul vftable 复位 + nengine::IObject 子对象复位 + sub_58E0D0 基类析构。',
 '注意 +8628（2157*4）处挂了 nengine::IObject 子对象 —— 怪物类的通用附属组件。',
])
note('0x4A2480', [
 '【推断】★ 怪物属性块大拷贝（vmt29C 槽的实现，0x4A2460 是它的 stdcall 壳）。',
 '逐字段 memcpy 式搬运 a2 → this（偏移+4 起），含 26+26 DWORD 的两段连续区 ——',
 '对应 sub_58DFE0 族构造里的属性表初始化。IDA 展开 115 行。',
 '【待验证】a2 来源（疑为 protocol/regen 表的静态实例）。',
])
note('0x4A2B30', [
 '【推断】属性块 2 槽赋值（vmt2A8）: 把 a2/a2+14 的 clong 值 push 进 this+6132/+6146。',
])
note('0x4A2C40', [
 '【推断】动画对象帧率态查询（vmt270 判定链的叶子）。返回动画对象 +112（帧率态）。',
 '【待验证】返回 2 = 冻结帧（见 IRDCollisionObject_vmt270 判据）。',
])
note('0x4A2C70', [
 '【推断】链表节点逐项释放（stdcall，Block = 链头）—— 循环 free，供 0x4A2DE0 清链。',
])
note('0x4A2CB0', [
 '【推断】链表节点逐项释放（第二种节点布局，同 0x4A2C70 对偶）—— 见 0x4A2DE0 双链清空。',
])
note('0x4A2CF0', [
 '【推断】申请 16 字节双向链表哨兵头（首/末指针自指）—— 0x4A49B0 用它初始化 +6544 光环链表。',
 '（与 docs/15 §3 "宿主 +6544/6548 = 光环显示链表头/计数" 对照）',
])
note('0x4A2D10', [
 '【推断】申请 20 字节双向链表哨兵头 —— 比 0x4A2CF0 多一个计数/类型字段。',
 '被 0x4AB840（CNSpikeKing 构造）用作 +8616 的内嵌链表初始化。',
])
note('0x4A2E50', [
 '【推断】std::string 三元组 + vector 清理（this+64/48/8 三块，MSVC string 的 15/16 容量布局）。',
])
note('0x4A2EB0', [
 '【推断】单个 MSVC std::string 析构（容量 >= 0x10 则 free 堆缓冲，复位为空短串）。',
 '被 0x4A3870/0x4A4170 用作"数组元素批量析构"的单位函数（元素步长 152 字节）。',
])
note('0x4A2EE0', [
 '【推断】48 字节固定头部 + 双向链表的初始化器（本文件 +0 处 200/1 等常数）。',
 '【待验证】字段: +0=200、+4=1000、+18(int)=50 —— 与属性块默认值风格一致。',
])
note('0x4A2D70', [
 '【已精修】受击盒使能 setter（vmt25C）—— 写 this+641（无敌开关）后遍历 +7076 附属物 vector，',
 '对每个附属物调其 vt+604(a2) 广播（研究 P35 定案）。完整逐行版见',
 'src/cnrd/IRDActiveObject.cpp IRDActiveObject_vmt25C。此处保留 L2 原文以维持本区函数计数。',
])
note('0x4A2F80', [
 '【推断】按名字查找链表节点（std::map 风格遍历 + sub_40BF00 字符串比较）。',
 '找到返回节点 [3]（值），未找到返回 -1。MSVC std::map<CString,int> 的查找模板实例。',
])
note('0x4A2FF0', [
 '【推断】结构体清零 + 两个 vector 释放（MSVC vector 三指针布局 @+40/+56）。',
])
note('0x4A3050', [
 '【推断】大结构体清零（2×104 字节 memset + 头尾字段）—— 尺寸约 288 字节。',
 '【待验证】与 0x4A3EB0 的 72 DWORD 拷贝对应（同为 288 字节对象）。',
])
note('0x4A30C0', [
 '【推断】std::length_error 抛出桩（"vector<T> too long"）—— MSVC 模板实例。',
 '被 0x4A3570/0x4A35C0 在 a2 > 0x15555555（12 字节元素的最大容量）时调用。',
])
note('0x4A3130', [
 '【推断】★ std::map/std::set 红黑树 single-node erase（MSVC _Tree::erase 模板实例）。',
 '特征: "invalid map/set<T> iterator" 异常对象、+21 红黑标志字节、+12 排序键、',
 '左右子指针 [0]/[1]/[2] 与 rebalance 调用链 —— 被 IRDCollisionObject_vmt12C（0x4E7480）',
 '用于命中目标集的移除。完整语义 = 标准库实现，本层只保留结构。',
])
note('0x4A33F0', [
 '【推断】含 CString(MSVC string @+4) 与两个 vector(@+48/@+64) 的对象初始化。',
 'sub_4015C0(ptr, szPassword, 0) = 空 CString 构造 —— szPassword 在此为空串模板。',
])
note('0x4A3450', [
 '【推断】★ 攻击上下文默认构造（IRDCollisionObject +388 属；被 IRDActiveObject_vmt158 调用）。',
 'IRDCollisionObject.h 记录: "vmt158 攻击上下文设置（sub_4B2970 拷贝 / sub_4A3450 默认）"。',
 '本函数即"默认"分支。初值: +5=4、+6/+7/+8=17、+19 字节=1、+21/+22=1120403456(=75.0f)。',
])
note('0x4A3520', [
 '【推断】攻击上下文重置（string 复位 + 0x4A3450 默认值）—— 复合重置器。',
])
note('0x4A3570', [
 '【推断】12 字节元素 vector 的 reserve（a2 > 0x15555555 抛 length_error）。',
 '【推断】元素 = 12 字节三元组（坐标/向量点）。',
])
note('0x4A35C0', [
 '【推断】与 0x4A3570 同构的第二个 reserve（另一元素类型/容器）。',
])
note('0x4A3610', [
 '【推断】红黑树递归析构（后序 free，MSVC _Tree 模板）—— 与 0x4A36D0 成对实现两种节点类型。',
])
note('0x4A36D0', [
 '【推断】红黑树递归析构（第二版）—— 与 0x4A3610 对偶（不同节点大小）。',
])
note('0x4A3790', [
 '【推断】红黑树容器清理（调 0x4A3610 析构全树 + free 头结点）。',
])
note('0x4A37D0', [
 '【推断】含 CString + vector 的对象重置（先清 15/16 容量串，再 sub_4A33F0 初始化）。',
])
note('0x4A3830', [
 '【推断】vector<T> 释放（sub_41DA40 逐元素析构 + free）—— MSVC vector 模板。',
])
note('0x4A3870', [
 '【推断】152 字节步长数组的元素批量析构（每元素 = 一个含 string 的结构，见 0x4A2EB0）。',
])
note('0x4A38A0', [
 '【推断】结构体拷贝赋值（含 CString @+4 sub_4014D0 与两个子对象 @+44/+60 sub_4ED3F0）。',
])
note('0x4A3900', [
 '【推断】命中记录写入（IRDCollisionObject_vmt164，研究已定案）。',
 '流程: 目标对象 vt+380 取记录值 → sub_4A3940 在红黑树中取槽 → 回写。',
])
note('0x4A3940', [
 '【推断】红黑树 lower_bound 式查找/插入（命中记录树的取槽器）。',
])
note('0x4A3990', [
 '【推断】红黑树查找（+21 红黑标志，键 @+12）—— MSVC _Tree::lower_bound 模板实例。',
])
note('0x4A39C0', [
 '【推断】红黑树插入重平衡（MSVC _Tree::_Insert 模板实例；多条旋转分支）。',
])
note('0x4A3B30', [
 '【推断】红黑树迭代器自减（MSVC _Tree::iterator-- 模板实例）。',
])
note('0x4A3B90', [
 '【推断】红黑树插结点 + 全树重平衡（MSVC _Tree::_Insert 的完整实现，110+ 行）。',
 '含 length_error 抛出路径 —— 标准库模板，语义与 STL 源码一致。',
])
note('0x4A3D40', [
 '【推断】红黑树插入查找点（返回可插入位置）。',
])
note('0x4A3E00', [
 '【推断】红黑树容器清理（调 0x4A36D0 版析构）。',
])
note('0x4A3E40', [
 '【推断】红黑树头结点初始化（sub_433AC0 分配 + 自指 + 红黑标志置 1=黑）。',
])
note('0x4A3EB0', [
 '【推断】★ 288 字节对象深拷贝（72 DWORD 搬运，含 26+26 两段循环）。',
 '与 0x4A3050（288 字节清零）配对；先调 0x4A3F80 做扩容/赋值准备。',
 '【待验证】本对象与"命中记录/伤害上下文"的关系（0x4A3F80 内含 vector reserve 逻辑）。',
])
note('0x4A3F80', [
 '【推断】★ 12 字节元素 vector 的拷贝赋值（MSVC operator= 模板）。',
 '三段式: ① 源空 → 清目标；② 源 > 现有容量 → 重新 reserve(0x4A3570) + 拷贝；',
 '③ 否则原地拷贝（多余元素截断）。被 0x4A3EB0 调用于深拷贝准备。',
])
note('0x4A4170', [
 '【推断】152 字节步长数组的析构+释放（每元素走 0x4A2EB0，与 0x4A3870 配对）。',
])
note('0x4A41C0', [
 '【推断】★ 48+ 字节配置块整体拷贝（this+7520 区）。逐字节搬运 17 字节后接',
 '连续 DWORD 段 —— 与 0x4A1160（返回 this+7520）同址，即 0x4A1150/0x4A1160 描述的内嵌块。',
 '【推断】装备/染色配置缓存，本函数 = 其"应用副本"。',
])
note('0x4A42B0', [
 '【推断】关联容器赋值（sub_41DA40 清旧 + 0x4A42E0 建新）—— MSVC map operator= 模板。',
])
note('0x4A42E0', [
 '【推断】红黑树拷贝（_Copy 模板实例，重挂根 + 极值指针修正）。',
])
note('0x4A4370', [
 '【推断】红黑树递归复制（_Copy 模板的递归体，sub_5648C0 分配结点）。',
])
note('0x4A4430', [
 '【推断】★ IRDCollisionObject 构造函数 —— 写 vftable（RTTI 直证）并调 sub_42AAB0 基类。',
 '被 0x4A49B0（IRDActiveObject 构造）作为第一步调用 —— 继承链构造顺序的实物证据。',
])
note('0x4A4560', [
 '【推断】写 this+641 —— IRDCollisionObject_vmt25C（受击盒使能）的单字节写版本',
 '（无附属物广播）。IRDCollisionObject.h: "dangerFlag +0x281(641)"。',
])
note('0x4A4590', [
 '【推断】IRDCollisionObject 析构体（被 0x4A4710 = vmt0 调用）。',
 '逐段释放命中列表/红黑树/vector —— 尺寸线索汇总在 docs/14。',
])
note('0x4A4740', [
 '【推断】★ 对象成员批量析构的第一段（IRDActiveObject 析构链前置）。',
 '连续对 this+0/4/8/12/16/20/24 等 8 个指针槽做 sub_42AF90（解绑）+ sub_657740（释放），',
 '再处理内嵌数组 —— 130 行 = 成员数量最多的析构段。',
 '【待验证】sub_42AF90 语义（疑为场景/渲染对象注销）。',
])
note('0x4A4940', [
 '【推断】0x4A4740 的收尾（再补两个 vector 释放）。',
])
note('0x4A49B0', [
 '【推断】★★ IRDActiveObject 构造函数（RTTI 直证）。完整成员布局直证点:',
 '① sub_4A4430 基类构造（IRDCollisionObject）→ 写 IRDActiveObject vftable；',
 '② eh vector constructor iterator(this+676, 0x554, 4, 0x4A1670, 0x4A1F00)',
 '   = **4 个 0x554(1364) 字节内嵌子对象**，构造 0x4A1670 / 析构 0x4A1F00；',
 '   —— 与 vmt2A0（0x4A0C80: this+1364*a2+676）完全互证！',
 '③ sub_4A1C30(this+6132) = 属性块（vt+672 返回处）；',
 '④ +6160 块（sub_4A2FF0）、+6316（sub_4A3450 攻击上下文）、+6544（sub_4A2CF0 光环链表）；',
 '⑤ +6560 内嵌动作子对象（off_6C2B88 内嵌对象 vftable）、+7076 附属物 vector；',
 '⑥ 收尾 sub_4D46C0 / sub_4D7140（0x4D 区战斗段初始化）。',
])
note('0x4A4BD0', [
 '【推断】IRDActiveObject 析构（RTTI 直证）。sub_4D7000 先做战斗段清理，',
 '再释放 +7076 附属物 vector 与 +7048 串 —— 与 0x4A4E70（vmt0 壳）配对。',
])
note('0x4A4EA0', [
 '【推断】★ IRDMonster 构造（怪物基类，RTTI 直证）。继承链: IRDActiveObject ← IRDMonster。',
 '① sub_4A49B0（IRDActiveObject 构造）→ 写 IRDMonster vftable；',
 '② +7172 置 15 / +7176 归零 —— 内嵌 string 初始化；',
 '后续大段成员初始化（84 行）。所有具体怪物类（CNGoblin*/CNSpider/...）的构造都从本函数出发。',
])
note('0x4A50F0', [
 '【推断】★ IRDMonster 析构体。逆向释放序: IObject 子对象(+8556)、string(+8532)、',
 'IObject(+8488)、vector(+8340)、29 元素×16 字节 eh vector(+7824)、string(+7740)...',
 '**元素计数直证: 29 × 0x10** 与 +8488/+8556 两处 nengine::IObject —— IRDMonster 尺寸 >8600B。',
])
note('0x4A5350', [
 '【推断】析构壳（调 0x4A50F0 + 可选 free）= 某个怪物类的 vmt0。',
])
note('0x4A5380', [
 '【推断】★ CNFighterIllusion（斗士幻影）构造 —— sub_5908C0/5908E0 专属初始化。',
 '构造链: sub_4A4EA0（IRDMonster）。RTTI 直证。',
])
note('0x4A5410', [
 '【推断】CNFighterIllusion 析构（vftable 复位 + 0x4A50F0）。',
])
note('0x4A5460', [
 '【推断】★ CNProvocationDoll（嘲讽人偶）构造（含 a4 参数传递 sub_590F30）。',
])
note('0x4A54F0', [
 '【推断】CNProvocationDoll 析构。',
])
note('0x4A5540', [
 '【推断】★ CNKeraha 构造（sub_5913F0/592650）。',
])
note('0x4A55D0', [
 '【推断】CNKeraha 析构。',
])
note('0x4A5620', [
 '【推断】★ CNVinoshu 构造（sub_5928E0/593FF0）。',
])
note('0x4A56B0', [
 '【推断】CNVinoshu 析构。',
])
note('0x4A5700', [
 '【推断】★ CNLuis 构造（sub_592BF0/593680）。',
])
note('0x4A5790', [
 '【推断】CNLuis 析构。',
])
note('0x4A57E0', [
 '【推断】★ CNLugaru 构造（sub_594340/594B10）。',
])
note('0x4A5840', [
 '【推断】CNLugaru 析构（sub_4E3360 = 战斗段公共清理，见多处怪物析构）。',
])
note('0x4A5890', [
 '【推断】CNLugaru 析构壳。',
])
note('0x4A58C0', [
 '【推断】★ CNGardener_Ral（园丁拉尔）构造 —— 继承 CNLugaru。',
])
note('0x4A58F0', [
 '【推断】★ 怪物 AI 决策函数（CNGardener_Ral 或其他某怪物覆写）。',
 '判据: vt+128 == 某值、vt+724 == 0（非冻结）、rand(88771231,100) < 10% 概率 ——',
 '随机触发分支。rand 种子 88771231 是 DFO 的固定全局随机源之一。',
 '【待验证】具体触发动作（在 10% 命中后做什么）。',
])
note('0x4A59E0', [
 '【推断】CNGardener_Ral 析构。',
])
note('0x4A59F0', [
 '【推断】★ CNBlackLugaru（黑拉鲁）构造（sub_594F90/595470）。',
])
note('0x4A5A80', [
 '【推断】CNBlackLugaru 析构。',
])
note('0x4A5AD0', [
 '【推断】★ CNWhiteLugaru（白拉鲁）构造（sub_595E10/596990）。',
])
note('0x4A5B60', [
 '【推断】CNWhiteLugaru 析构。',
])
note('0x4A5BB0', [
 '【推断】★ CNPenril 构造（sub_598350/598200）。',
])
note('0x4A5C40', [
 '【推断】CNPenril 析构。',
])
note('0x4A5C90', [
 '【推断】★ CNBloodLugaru（血拉鲁）构造（+8656 标志字节 / +8652 IObject）。',
])
note('0x4A5D60', [
 '【推断】CNBloodLugaru 析构（含 nengine::IObject @+8652 复位）。',
])
note('0x4A5DC0', [
 '【推断】★ CNGoblin 族构造共享前缀 —— 写 CNGoblinCommon vftable 于 +8600，',
 '+8604=0 / +8605=1。**这是全哥布林族的"第二 vftable"位置（this+8600）之直证**，',
 '解释了大量 *(this+2150) = &CNGoblinXXX::vftable 的写法（2150*4 = 8600）。',
])
note('0x4A5E70', [
 '【推断】CNGoblinStoner 析构壳（this−8600 回退到对象头，多继承布局修正）。',
])
note('0x4A5EB0', [
 '【推断】CNGoblinStoner 析构（双 vftable 复位 + string 释放）。',
])
note('0x4A5F10', [
 '【推断】哥布林族构造共享前缀（第二版，参数不同）。',
])
note('0x4A5FF0', [
 '【推断】★ CNGoblinGoblin 析构（双 vftable；+8600 的 vftable 换为 off_6D0E8C）。',
])
note('0x4A60F0', [
 '【推断】★ CNGoblinAssaulter 构造。',
])
note('0x4A6160', [
 '【推断】CNGoblinAssaulter 析构。',
])
note('0x4A6180', [
 '【推断】★ CNGoblinHodor 构造（+8708 标志）。',
])
note('0x4A6270', [
 '【推断】CNGoblinHodor 析构（含 IObject @+8704）。',
])
note('0x4A62A0', [
 '【推断】哥布林族构造共享前缀（第三版）。',
])
note('0x4A6380', [
 '【推断】★ CNGoblinKinol 析构相关。',
])
note('0x4A6480', [
 '【推断】★ CNBunker_Woker 构造（sub_59BF90）。',
])
note('0x4A6520', [
 '【推断】CNBunker_Woker 析构。',
])
note('0x4A6540', [
 '【推断】★ CNEliteGoblin 构造（+8680 标志）。',
])
note('0x4A6580', [
 '【推断】★ CNKingGalok 构造（继承 CNEliteGoblin 前缀布局，+8680 清零）。',
])
note('0x4A6600', [
 '【推断】CNEliteGoblin 析构。',
])
note('0x4A6620', [
 '【推断】哥布林族构造共享前缀（第四版）。',
])
note('0x4A66E0', [
 '【推断】★ CNGoblinGero 析构相关。',
])
note('0x4A67B0', [
 '【推断】★ CNPrag 构造（+8620 = 30076 动作 ID 常量）。',
])
note('0x4A6830', [
 '【推断】CNPrag 析构。',
])
note('0x4A6850', [
 '【推断】哥布林族构造共享前缀（第五版）。',
])
note('0x4A6940', [
 '【推断】★ CNGoblinMountainGab 析构相关。',
])
note('0x4A69E0', [
 '【推断】哥布林族构造共享前缀（第六版）。',
])
note('0x4A6AD0', [
 '【推断】★ CNGoblinIslandGab 析构相关。',
])
note('0x4A6B70', [
 '【推断】哥布林族构造共享前缀（第七版）。',
])
note('0x4A6C60', [
 '【推断】★ CNGoblinLupete 析构相关。',
])
note('0x4A6D00', [
 '【推断】哥布林族构造共享前缀（第八版）。',
])
note('0x4A6DE0', [
 '【推断】★ CNGoblinKagelBase（哥布林炸弹/飞镖 Kagel 系基类）构造（+8648 标志）。',
])
note('0x4A6E80', [
 '【推断】CNGoblinSharedoBase 析构。',
])
note('0x4A6EE0', [
 '【推断】★ CNGoblinSharedo 构造（sub_5A0530）。',
])
note('0x4A6F40', [
 '【推断】CNGoblinSharedoBase 析构（第二版，含 vftable 降级回 CNGoblinCommon 的细节）。',
])
note('0x4A6F60', [
 '【推断】★ CNGoblinBombKagel 构造（sub_5A0780）。',
])
note('0x4A7000', [
 '【推断】CNGoblinBombKagel 析构（含 IObject @+8644）。',
])
note('0x4A7090', [
 '【推断】★ CNGoblinSurikenKagel 构造（sub_5A0930）。',
])
note('0x4A7100', [
 '【推断】★ CNGoblinBullsEye 构造（+8688 清零）。',
])
note('0x4A71C0', [
 '【推断】CNGoblinBullsEye 析构（含对 +8704 对象的 vt 调用）。',
])
note('0x4A7280', [
 '【推断】哥布林族构造共享前缀（第九版）。',
])
note('0x4A7360', [
 '【推断】★ CNLightningKinol 析构相关。',
])
note('0x4A73D0', [
 '【推断】哥布林族构造共享前缀（第十版）。',
])
note('0x4A74B0', [
 '【推断】★ CNCowardGoblin（怯懦哥布林）析构相关（sub_5A20C0）。',
])
note('0x4A7520', [
 '【推断】★ CNTauArmy（牛头人军团）构造 —— 先写 CNTauFriendShip 再写本类。',
])
note('0x4A75D0', [
 '【推断】CNTauArmy 析构（vftable 降级回 CNMonsterFriendShip）。',
])
note('0x4A7640', [
 '【推断】★ CNTauBeast（牛头兽）构造。',
])
note('0x4A76F0', [
 '【推断】CNTauBeast 析构。',
])
note('0x4A7760', [
 '【推断】★ CNTauAssaulter（牛头突击兵）构造。',
])
note('0x4A77E0', [
 '【推断】CNTauAssaulter 析构。',
])
note('0x4A7880', [
 '【推断】★ CNRock_Moud 构造。',
])
note('0x4A78E0', [
 '【推断】CNRock_Moud 析构。',
])
note('0x4A7900', [
 '【推断】★ CNTauGuard（牛头卫兵）构造。',
])
note('0x4A79B0', [
 '【推断】CNTauGuard 析构。',
])
note('0x4A7A20', [
 '【推断】★ CNTauTangas（牛头唐加斯）构造。',
])
note('0x4A7AD0', [
 '【推断】CNTauTangas 析构。',
])
note('0x4A7B40', [
 '【推断】★ CNGolem（魔像族基类）构造（+8636=15/+8632 归零 string）。',
])
note('0x4A7BC0', [
 '【推断】含字符串常量 "STONE_DIE" 的分发调用 —— 魔像族死亡动画/音效触发。',
])
note('0x4A7BE0', [
 '【推断】CNGolem 析构（sub_5A5BF0 + string 释放）。',
])
note('0x4A7C90', [
 '【推断】★ CNGolemStone（石魔像）构造（sub_5A7770）。',
])
note('0x4A7D30', [
 '【推断】★ CNGolemSteel（钢魔像）构造（sub_5A78E0）。',
])
note('0x4A7DD0', [
 '【推断】★ CNGolemBronze（青铜魔像）构造（sub_5A7E30）。',
])
note('0x4A7E30', [
 '【推断】★ CNGolemClay（陶土魔像）构造（sub_5A7B50）。',
])
note('0x4A7ED0', [
 '【推断】★ CNGolemGold（金魔像）构造（sub_5A7CC0）。',
])
note('0x4A7F70', [
 '【推断】★ CNGolemPlatany（白金魔像）构造（sub_5A80A0）。',
])
note('0x4A8010', [
 '【推断】★ CNStoneBlade_Razor 构造（继承 CNGolemStone 前缀）。',
])
note('0x4A8040', [
 '【推断】★ CNDollMaster（人偶大师）构造（sub_5A82B0/5A9DA0）。',
])
note('0x4A80A0', [
 '【推断】CNDollMaster 析构（sub_5A8300）。',
])
note('0x4A8120', [
 '【推断】★ CNStoneMaster 构造。',
])
note('0x4A8190', [
 '【推断】★ CNSteelMaster 构造。',
])
note('0x4A8200', [
 '【推断】★ CNClayMaster 构造。',
])
note('0x4A8270', [
 '【推断】★ CNMasterDogri 构造。',
])
note('0x4A82E0', [
 '【推断】★ CNIceDoll_Sharedo（冰人偶）构造。',
])
note('0x4A8350', [
 '【推断】★ CNFlyDragon（飞龙）构造 —— 龙族继承链基。',
 '100 行初始化（内嵌结构多）。后续 CNDrakeSerimalion 等经此继承。',
])
note('0x4A8560', [
 '【推断】CNFlyDragon 析构（含 IObject @+8748）。',
])
note('0x4A8680', [
 '【推断】★ CNDrakeSerimalion 构造（+8788 标志）。',
])
note('0x4A8780', [
 '【推断】CNDrakeSerimalion 析构。',
])
note('0x4A87A0', [
 '【推断】★ CNDrakeLooseSerimalion 构造（参数 void* Block 复用）。',
])
note('0x4A88C0', [
 '【推断】CNDrakeLooseSerimalion 析构（含 +8812 指针的虚调）。',
])
note('0x4A8940', [
 '【推断】★ CNDrakeHarseSerimalion 构造（+8780=-1/+8776=0 计时器初始化）。',
])
note('0x4A89B0', [
 '【推断】CNDrakeHarseSerimalion 析构。',
])
note('0x4A89C0', [
 '【推断】★ CNDrakeSelionTheFireBreather（喷火塞利昂）构造。',
])
note('0x4A8A90', [
 '【推断】CNDrakeSelionTheFireBreather 析构（sub_5AA350 + IObject @+8612）。',
])
note('0x4A8AF0', [
 '【推断】★ CNHunter（猎人）构造（+8648=15/+8644 归零 string）。',
])
note('0x4A8B70', [
 '【推断】CNHunter 析构（string @+8628 释放）。',
])
note('0x4A8C20', [
 '【推断】★ CNHunterMaster 构造（sub_5ADFE0）。',
])
note('0x4A8CC0', [
 '【推断】★ CNHunterAlex 构造（sub_5AE160）。',
])
note('0x4A8D60', [
 '【推断】★ CNHunterHunter 构造（sub_5AE290）。',
])
note('0x4A8E00', [
 '【推断】★ CNZombie（僵尸）构造（sub_5AE3E0/5AF220）。',
])
note('0x4A8E60', [
 '【推断】CNZombie 析构。',
])
note('0x4A8EE0', [
 '【推断】★ CNZombieZombie 构造。',
])
note('0x4A8F50', [
 '【推断】★ CNZombieHollowEye（空洞眼僵尸）构造。',
])
note('0x4A8FC0', [
 '【推断】★ CNZombieGhoulgwish 构造（sub_5AE830 覆写加载）。',
])
note('0x4A9060', [
 '【推断】★ CNKalkazak 构造（+8652/+8656 双槽清零）。',
])
note('0x4A90D0', [
 '【推断】★ CNBurningHela（燃烧海拉）构造。',
])
note('0x4A9140', [
 '【推断】★ CNDraconian（龙人）构造（sub_5AF910/5B0660）。',
])
note('0x4A91A0', [
 '【推断】CNDraconian 析构（sub_5AF950）。',
])
note('0x4A9220', [
 '【推断】★ CNDracoMirkius 构造。',
])
note('0x4A9290', [
 '【推断】★ CNDracoLakius 构造（sub_5B1890）。',
])
note('0x4A9330', [
 '【推断】★ CNAuraHunter_Sius（光环猎人西斯）构造（sub_5AFDD0）。',
])
note('0x4A9390', [
 '【推断】★ CNLordofLightBase（光之领主基）构造（sub_5B1910/5B2720）。',
])
note('0x4A93F0', [
 '【推断】CNLordofLightBase 析构（sub_5B1940）。',
])
note('0x4A9470', [
 '【推断】★ CNLordofLight 构造（sub_5B3100）。',
])
note('0x4A9510', [
 '【推断】★ CNSpaceFishHead（太空鱼头）构造（sub_5B3230/5B3F30）。',
])
note('0x4A9570', [
 '【推断】CNSpaceFishHead 析构。',
])
note('0x4A95F0', [
 '【推断】★ CNSpaceFishShort 构造（sub_5B3400）。',
])
note('0x4A9690', [
 '【推断】★ CNSpaceFishLong 构造（sub_5B34D0）。',
])
note('0x4A9730', [
 '【推断】★ CNSpaceFishCrocuta 构造（sub_5B35C0）。',
])
note('0x4A97D0', [
 '【推断】★ CNStarFish（海星）构造。',
])
note('0x4A9850', [
 '【推断】CNStarFish 析构。',
])
note('0x4A98F0', [
 '【推断】★ CNStarFishRukuku 构造。',
])
note('0x4A9970', [
 '【推断】★ CNStarFishDevourer 构造。',
])
note('0x4A99F0', [
 '【推断】★ CNStarFishKing 构造（+8664 标志）。',
])
note('0x4A9A90', [
 '【推断】CNStarFishKing 析构（IObject @+8660）。',
])
note('0x4A9AC0', [
 '【推断】★ CNZealot（狂信徒）构造 —— 教团系（Raxy/Craxy/Father/Pope/Evangelist）继承基。',
])
note('0x4A9B70', [
 '【推断】CNZealot 析构（IObject @+8608，2152*4=8608）。',
])
note('0x4A9C20', [
 '【推断】★ CNRaxy_A 构造（+8600 → off_6E7860 特殊 vftable）。',
])
note('0x4A9C50', [
 '【推断】★ CNRaxy_B 构造。',
])
note('0x4A9CD0', [
 '【推断】★ CNCraxy 构造。',
])
note('0x4A9D00', [
 '【推断】★ CNFather（教父）构造。',
])
note('0x4A9D80', [
 '【推断】★ CNPope（教皇）构造。',
])
note('0x4A9E00', [
 '【推断】★ CNEvangelistOfDawn（黎明传道者）构造（+8672 = 30041 技能/动作 ID）。',
])
note('0x4A9E50', [
 '【推断】★ CNEvangelistOfLight（光之传道者）构造。',
])
note('0x4A9E80', [
 '【推断】★ CNEvangelistOfTwilight（暮光传道者）构造（+8676 标志）。',
])
note('0x4A9F10', [
 '【推断】★ CNEvangelistOfDark（暗之传道者）构造（+8672 = 30042）。',
])
note('0x4A9FA0', [
 '【推断】★ CNVehicle（载具基）构造（sub_5BA050/5BAC30）。',
])
note('0x4AA000', [
 '【推断】CNVehicle 析构（sub_5BA090）。',
])
note('0x4AA080', [
 '【推断】★ CNCatapult（投石车）构造（sub_5BAD00）。',
])
note('0x4AA0F0', [
 '【推断】★ CNLaserCatapult（激光投石车）构造。',
])
note('0x4AA150', [
 '【推断】★ CNFlameCatapult（火焰投石车）构造。',
])
note('0x4AA1E0', [
 '【推断】★ CNBattlIngram（战斗英格拉姆/机甲）构造（sub_5BAEF0）。',
])
note('0x4AA280', [
 '【推断】★ CNZepplin（飞艇）构造 —— 含 2 元素×0x1C(28) 字节内嵌数组（eh vector ctor）。',
])
note('0x4AA300', [
 '【推断】CNZepplin 析构（2×28B eh vector dtor @+8616）。',
])
note('0x4AA3A0', [
 '【推断】★ CNCannonZepplin（炮艇）构造（sub_599D80）。',
])
note('0x4AA440', [
 '【推断】★ CNNormalZepplin（普通飞艇）构造。',
])
note('0x4AA4E0', [
 '【推断】★ CNWraith（怨灵）构造（+8624=15/+8620 string）。',
])
note('0x4AA560', [
 '【推断】CNWraith 析构（string @+8624 区）。',
])
note('0x4AA610', [
 '【推断】★ CNWraithSpectress（女怨灵）构造（+8656/+8660 双槽）。',
])
note('0x4AA6F0', [
 '【推断】CNWraithSpectress 析构（sub_5BCFC0 + IObject @+8672）。',
])
note('0x4AA780', [
 '【推断】★ CNSpider（蜘蛛）构造（sub_5BEC80/5BECA0）—— 蜘蛛族基。',
])
note('0x4AA7F0', [
 '【推断】★ CNRollingSpider（滚动蜘蛛）构造（+8628 标志 / +8624 IObject）。',
])
note('0x4AA8C0', [
 '【推断】CNRollingSpider/CNSpider 析构（IObject @+8624 复位）。',
])
note('0x4AA8E0', [
 '【推断】★ CNPoisonSpider（毒蜘蛛）构造（sub_5C0400）。',
])
note('0x4AA970', [
 '【推断】★ CNAraclothoSpider（阿拉克洛蜘蛛）构造（+8664 标志 / +8660 IObject）。',
])
note('0x4AAAB0', [
 '【推断】CNAraclothoSpider 析构（sub_5C1270 + IObject @+8716）。',
])
note('0x4AAB30', [
 '【推断】★ CNRobotRX78（机器人 RX-78）构造（sub_5C1FC0）—— 机甲族。',
])
note('0x4AABC0', [
 '【推断】CNRobotRX78 析构（sub_5C1FF0）。',
])
note('0x4AAC10', [
 '【推断】★ CNRobotEz8 构造（sub_5C2830/5C2870）。',
])
note('0x4AACA0', [
 '【推断】CNRobotEz8 析构（sub_5C28A0）。',
])
note('0x4AACF0', [
 '【推断】★ CNRobotExs 构造（sub_5C3100/5C3760）。',
])
note('0x4AAD50', [
 '【推断】CNRobotExs 的覆写判定（转发基类 vt 调用 + 多条件）。',
])
note('0x4AADA0', [
 '【推断】CNRobotExs 析构（sub_5C3170）。',
])
note('0x4AADF0', [
 '【推断】★ CNDendroidBasic（机械树精基）构造（sub_5C3A30/5C57F0）。',
])
note('0x4AAE50', [
 '【推断】CNDendroidBasic 析构（sub_5C4780）。',
])
note('0x4AAED0', [
 '【推断】★ CNDendroid 系派生（off_6F04BC vftable）构造（sub_5C4080）。',
])
note('0x4AAF70', [
 '【推断】★ CNGoliden 构造（sub_5C4100）。',
])
note('0x4AB010', [
 '【推断】★ CNRodinglo 构造（sub_5C41B0）。',
])
note('0x4AB0B0', [
 '【推断】★ CNSpirit（精灵/魂）构造（+8624=15/+8620 string）—— 魂系基。',
])
note('0x4AB180', [
 '【推断】CNSpirit 析构（52 行，多成员释放）。',
])
note('0x4AB320', [
 '【推断】★ CNNaias（水泽仙女）构造（+8744 标志 / sub_5C8290）。',
])
note('0x4AB3C0', [
 '【推断】★ CNAdor 构造（sub_5C8470）。',
])
note('0x4AB430', [
 '【推断】★ CNSoulEter（食魂者）构造（sub_5C6590）。',
])
note('0x4AB4C0', [
 '【推断】★ CNStalker（跟踪者）构造（sub_5C8650）。',
])
note('0x4AB560', [
 '【推断】★ CNWisp（鬼火）构造（sub_5C8FE0）。',
])
note('0x4AB600', [
 '【推断】★ CNLotusBasic（莲花基）构造（+8644 标志 / +8640 IObject）—— 植物族（Lotus/MonsterFlower）。',
])
note('0x4AB6B0', [
 '【推断】CNLotusBasic 析构（sub_5C92A0 + IObject @+8640）。',
])
note('0x4AB740', [
 '【推断】★ CNLotus 构造。',
])
note('0x4AB7B0', [
 '【推断】★ CNMonsterFlower（怪物花）构造（sub_5CB970）。',
])
note('0x4AB810', [
 '【推断】CNMonsterFlower 析构壳（sub_5CBD40）。',
])
note('0x4AB840', [
 '【推断】★ CNSpikeKing（尖刺王）构造 —— +8616 = sub_4A2D10()（20 字节链表头），',
 '这是 CNSpikeKing 内嵌链表对象的实证，参照 0x4A2D10 注释。',
])
note('0x4AB960', [
 '【推断】CNSpikeKing 析构（sub_4E3360 + IObject @+8660）。',
])
note('0x4AB9F0', [
 '【推断】★ CNAbom（阿波姆）构造（+8600 IObject / +8604 标志）—— 巨人族基（Goliath/Atlas/Titan）。',
])
note('0x4ABAD0', [
 '【推断】CNAbom 析构（sub_5CD150 + IObject @+8648）。',
])
note('0x4ABB70', [
 '【推断】★ CNGoliath（歌利亚）构造（sub_5CD1E0/5CD220）。',
])
note('0x4ABC00', [
 '【推断】CNGoliath 析构（sub_5CD2A0）。',
])
note('0x4ABC50', [
 '【推断】★ CNAtlas（阿特拉斯）构造（sub_5CD360/5CD380）。',
])
note('0x4ABCE0', [
 '【推断】CNAtlas 析构。',
])
note('0x4ABD30', [
 '【推断】★ CNTitan（泰坦）构造（sub_5CD3C0/599D80）。',
])
note('0x4ABDC0', [
 '【推断】CNTitan 析构（sub_5CD3F0）。',
])
# ---- 0x4AExxx-0x4AFxxx：怪物子类批；0x4AEA30-0x4AEF：CNMonster 家族之外 ----
note('0x4AEA30', [
 '【推断】链表按"容器+4"标签查找（命中对象列表的成员函数; a2+8 为键）。',
 '找到后调目标 vt+4(a2) —— 疑为"按类型分发通知"。',
])
note('0x4AEA70', [
 '【推断】★ 环形缓冲逐格弹出（this+68/72/76/80 四字段 = 缓冲基/容量/头下标/计数）。',
 '每轮: 取缓冲[头] → 头++ (%容量) → 计数-- → 计数归 0 时下标清零 →',
 '弹出对象非空则先在双向链表中查找匹配节点（容器+4 键）并 vt+4 通知，',
 '再交全局 sub_49C800(&unk_9B9310, obj) —— 全局队列投递。',
 '语义 = 对象回收队列的逐帧排空（unk_9B9310 = 全局延迟释放/重生池）。',
])
note('0x4AEB30', [
 '【推断】0x4AEA70 排空 + sub_58B380(this+4) 后续清理。',
])
note('0x4AEB50', [
 '【推断】环形缓冲整体排空（this+4/+8/+12/+16 = 基/容量/头/计数）+ sub_406E40 收尾。',
 '与 0x4AEA70 对偶（一个逐格、一个整体）。',
])
note('0x4AEC00', [
 '【推断】容器重置（sub_498A00 + sub_58B450 + 清 4 字段 + 链头释放）。',
])
note('0x4AEC70', [
 '【推断】容器析构（sub_4AEB50 排空 + sub_58B4D0 + sub_4989D0 释放链）。',
])
note('0x4AECE0', [
 '【推断】动画切换: sub_491C70(a2) 取动画资源 → sub_493A60(this, res) 应用。',
])
note('0x4AED00', [
 '【推断】有效性判定: this+16 字节非 0 或 this+0 浮点 > 0.0。',
])
note('0x4AED20', [
 '【推断】帧数据 getter（与 IRDCollisionObject_vmt270 的帧检查同构）:',
 'this+642 非 0 → 直接返回 this+644；否则从动画对象 +112 读帧。',
])
note('0x4AED60', [
 '【推断】防篡改浮点自增（读 sub_41A5E0 → +a2 → 写 sub_41A590）。',
])
note('0x4AED90', [
 '【推断】转发 sub_47EFD0（this, a2, a3, a4, a4, a5）—— 六参版桥接。',
])
note('0x4AEDB0', [
 '【推断】全局清理: sub_4C4210 + sub_617B80 + 释放全局对象 dword_9B9410（vt+0(1) 后置空）。',
])
note('0x4AEDD0', [
 '【推断】★ IRDActiveObject 状态区大重置（vmt10 类初始化链的核心，109 行）。',
 '写入: +96 = 273（条件位集，与 CNRDObject_vmt28 的 273 掩码直证对应！）、',
 '+608 归零、+7512 = -1、+7516..+7608 连续清零、+7760 = 4（模式）、+7836 = 1000（千分基准）、',
 '+7924 = 1、+7728/+7730 标志、+8172 区 5×2 循环清 20 个 DWORD、+7428/+8212 收尾。',
 '**+96 = 273 是条件位掩码的初始值（受击态条件位）—— 与 docs/14 §2 "vt+40 条件位掩码"闭环。**',
])
note('0x4AF010', [
 '【推断】★ 属性表整体导入（93 行）。从 a2 结构体读值批量写属性块:',
 '① qmemcpy(this+7432, a2, 0x4F) = 79 字节原样拷贝（原始表区）；',
 '② (a2+4)/10 → this+708 双写（int + cfloat）/1.0；',
 '③ 4 元素循环（a2+16 起 int16，每项 /10）→ this+848 族；',
 '④ 16 元素循环（a2+24 起）→ this+960 族；',
 '⑤ 单项: a2+56/60/62/64/66/68/70/72/74 逐个 /10 换算写入 +1408..+1660 族；',
 '⑥ 最后 vt+676(0) 刷新。**模式 = 服务端属性表 → 客户端防篡改壳写入**（研究第 17 轮闭环）。',
])
note('0x4AF330', [
 '【推断】★ 属性重算 + 广播（vmt 链）。',
 '① sub_432FD0((*v3), 2) 取系数 v4；(dword_8F7A24+12) 是当前场景/频道类型，',
 '   非 4/6 时 v6 = this+7510（本对象自定义值），否则 v6 = 100；',
 '② this+7432 的 int × v4 × 0.1 → +680（双写）；',
 '③ v6 × this+7440/7442/7444/7446（4 个 int16） / 1000 → +736/+772/+808/+844（四属性）；',
 '④ vt+676(0) 刷新；⑤ a2 非 0 时: vt+740 + vt+732、vt+756 + vt+748 两组百分比应用。',
 '【推断】这是"属性表 → 实际战斗数值"的换算器（/10 与 /1000 两类比例）。',
])
note('0x4AF4B0', [
 '【推断】两态切换外壳（a2 = 0/非0）: 每态里若有动画对象则先 sub_4938F0 同步帧，',
 '取 anim+95 字节作参数、vt+792(this,9) 取值、调 sub_432600(态, a3, a4, this, v, 1, b) ——',
 '对偶形态的成对触发（进入/退出）。sub_58E750 是基类同名实现（前后各调一次）。',
])
note('0x4AF560', [
 '【推断】★ 双结构体差异应用（stdcall, a1=上下文, a2/a3=新旧两份表）。',
 '对 a2/a3 逐字段: 双方非空时 sub_494240(a1, a3.field) —— **只对"新表存在的项"做提交**。',
 '布局: a3+0..+16 与 a2+1..+4×4（交错 4 元素循环）、+36/+40/+44/+48 四尾项。',
 '【推断】装备/外观部件表差分应用（与 docs/15 的 vmt2B4 组件槽结构 9 单指针+数组段一致）。',
])
note('0x4AF650', [
 '【推断】★ 0x4AF560 的逆操作 —— 双结构体差异卸载（sub_492980 释放旧资源）。',
 '同样 4 元素交错循环 + +36..+48 尾项 —— 挂载/卸载对称对。',
])
note('0x4AF720', [
 '【推断】★ 高亮目标切换。this+7892(1973*4=7892) 保存当前高亮对象指针；',
 'v2 = this+29 非 0 时 sub_45EAC0(this) 求新对象；不同则: 旧对象 +976 标志清 0 且',
 '调其 vt+284(0, -16715521, 0, 1)（-16715521 = 0xFF00FFFF 颜色值），',
 '新对象 +976 置 1 且调 vt+284(1, 颜色, 0, 1)。**这是"选中高亮描边"的着色调用**。',
])
note('0x4AF790', [
 '【推断】★ 状态覆写批量扫描。对 190 个索引: 若 vt+1160(i) 返回对象的 +124 == 1，',
 '则 vt+1140(i) 取新值 → vt+1304(this, i, 值) 覆写；最后 vt+676(2) 刷新。',
 '**190 = 状态槽总数候选**（与 17 异常状态槽不同层）。',
])
note('0x4AF7F0', [
 '【推断】★ 按场景类型发出 UI 通知。vt+308 本地玩家判定通过后按 (dword_8F7A24+12):',
 '场景 3 → 事件 43; 场景 4 → 事件 58; 场景 6 → 事件 95 + sub_48FCE0(*(this+8016)) 一字节载荷。',
 'sub_423200/sub_423290/sub_423380 = UI 消息构造（类型/载荷/发送，见 0x4AF9A0 同族）。',
])
note('0x4AF8B0', [
 '【推断】★ 观察者槽差分读: (this[a2+1617] == this[a2+1879]) ? 0 : 前者。',
 '**+6468 与 +7516 是两套 18 槽表（1617 与 1879 = 相差 262 DWORD = 1048 字节）**，',
 '本函数返回"两表不同则取前者"—— 疑为"当前值 vs 基准值"的差分比较器。',
])
note('0x4AF8E0', [
 '【推断】★ 状态槽的惰性初始化（vmt 覆写）。先调基类 sub_4D4CE0(this, a2)；',
 '若 this+512 == 4: 用 vt+1176/vt+1180 重新取两值；若 this+520 == -1: 用 vt+1184 取初值；',
 '若 this+524 == -1082130432（-4.0f 位模式）: vt+1188 取浮点初值；',
 '最后: this+520 > 0 时用 (2.0 − 属性块+1292 的浮点) × this+520 修正 ——',
 '**防篡改壳（+1292）参与状态时长的缩放**。',
])
note('0x4AF9A0', [
 '【推断】★ UI 包构造: 事件 50。流程: vt+308 玩家判定 → vt+792(this, a2) 取对象 →',
 '发事件 50 → 载荷依次: vt+120(x, 2B)、vt+124(y, 2B)、字节 3(1B)、a2(2B)、1(4B) → 发送。',
 '**双坐标 + 类型 + 目标 ID 的纯坐标型通知**（疑似"标记/路径点"）。',
])
note('0x4AFA70', [
 '【推断】★ UI 包构造: 事件 18（与 0x4AF9A0 同族但载荷布局不同）:',
 '字节 3(1B) + 字节 1(1B) + a3(2B) + a2(2B) + 1(4B)。**a3 是新增参数**。',
])
note('0x4AFB30', [
 '【推断】攻击反应前置钩: 若 vt+724 == 0（非冻结）且 vt+308 本地玩家且 this+404 非 0，',
 '先 vt+1104(this, 8)；再转发基类 sub_4D6D90（=IRDActiveObject_vmt1F4 攻方完整反应）。',
])
note('0x4AFB80', [
 '【推断】★ 随机触发状态: 五重条件（this+7984 对象非空且其 vt+80 通过 + vt+1000 通过 +',
 'a4==0 + this+7988 浮点 ×100 > rand(87723146,10000)）→ this+7992 = 1。',
 'rand 种子 87723146 —— 第二处固定随机源（与 0x4A58F0 的 88771231 并列）。',
])
note('0x4AFBF0', [
 '【推断】受击反应包装: 调 sub_4D5A60(this, a2, a3)（0x4D5A60 = IRDActiveObject vmt288）；',
 '结果 != 3 且 this+7992 置位 → 返回 1（强制生效）。',
])
note('0x4AFC30', [
 '【推断】0x4AFBF0 的配对: 清 this+7992 后调 sub_4D4A80（IRDActiveObject_vmt204 观察者广播）。',
])
note('0x4AFC40', [
 '【推断】vt+308 玩家判定 → vt+792(this, a2) 取对象 → 调其 vt+268。',
])
note('0x4AFC80', [
 '【推断】★ 血量低于阈值时的场景分支。若 a2 < vt+736():',
 '场景 4 且 (dword_8F7A24+32)+36 == 3 → sub_4527A0(vt+1216 值, 差值)（疑为镜头/提示）；',
 '场景 6 → sub_42B510((dword_8F7A24+40))；最后转发 sub_4D5900（vmt2DC）。',
 '**这是"濒死镜头"或"低血提示"的触发点候选。**',
])
note('0x4AFD00', [
 '【推断】★ 可行动判定: vt+1196 == 0 且 vt+724 == 0（未冻结），',
 '且大状态(+6556) ∈ {0, 11} 或 (== 8 且 vt+1140(171) > 0) → true。',
 '**大状态 0/11/8 是"可自由行动"状态集**（0=待机、11=??、8=?? 且带 171 号条件）。',
])
note('0x4AFD50', [
 '【推断】★ 收招/起身触发: vt+308 判定后，v4 = (this+7730 != 0) && a2 != 0 && vt+1140(170) > 0；',
 '先 vt+1160(this, 169) 做一次查询（忽略结果），再 sub_47E440(v4)。',
 '**条件编号 169/170 与 0x4AFEC0 的 169/175/176/180 同族 —— 动作抑制条件集**。',
])
note('0x4AFDB0', [
 '【推断】★ 18 观察者 vt+144 广播 + 本地玩家的 vt+290/vt+324 链。',
 '① vt+77（=0x4C 字节，vt+308 玩家判定）通过 → 对 18 个观察者槽逐个调 vt+144(&a2, &a3)；',
 '② 若 this == dword_8F7C04（本地玩家）→ vt+290(this, a2) 取对象，其 +376 非 0 时',
 '调 vt+324(this, a2, a3)。**与 docs/15 的 vmt310/324 装备链呼应。**',
])
note('0x4AFE30', [
 '【推断】★ 三点标尺计算（a3 = 索引 0..N）。this+7188/+7192/+7196 = 三值:',
 'a2 − dword_9B7410[a3]、dword_9B7414[a3] − 基准、a2。a3==1 时基准 = 0。',
 '**dword_9B7410/9B7414 是两张全局标尺表（按 a3 索引）—— 血条/进度条刻度候选。**',
])
note('0x4AFE70', [
 '【推断】三槽写入（+7200/+7208/+7204）—— 与 0x4AFE30 三元组同区。',
])
note('0x4AFEA0', [
 '【推断】写 this+7730 标志后调 vt+1316() —— 状态变更通知（0x4AFEC0 会读 +7730）。',
])
note('0x4AFEC0', [
 '【推断】★★ 全局 UI/表现标志同步器（仅当 this == dword_8F7C04 本地玩家）。',
 '置全局字节矩阵: byte_9A0358/9A0384/9A03B0/9A03DC/9A0408/9A0434/9A0460 ——',
 '前四个初值 0，后三个 = (大状态 != 5)。',
 '再看 this+7730 分两支: 非 0 → 按大状态(+6556) switch:',
 '  case 0 → 9A03B0=1, 9A03DC=1（落入 LABEL_5: 9A0358=1, 9A0384=1 + 关 169/175/176/180 四条件）;',
 '  case 6 → 读 +6892/+6896/+6900 三级联 + this+6988 → 9A0358 = (6988==0);',
 '  case 8 → 9A0358=1, 9A0384=1 + vt+1328(this,1);',
 '  case 11 → LABEL_5 全开;',
 '否则（+7730 == 0）→ 大状态非 0 时 9A0384 = (状态==11)，为 0 时 9A03B0/9A03DC/9A0384 全置 1。',
 '**这七个全局字节是"UI 元素可见性/可交互性"总闸（本地玩家专属）—— 与 0x4AFD00/FD50/FEC0',
 ' 构成"本地玩家操作权限"三位一体。**',
])

# ---------------------------------------------------------------
# 4. 分区结构（按顺序的函数索引区间 + 标题）
# ---------------------------------------------------------------
SECTIONS = [
  (0x4A0180, '属性/渲染值同步族（对象子块: 挂点跟随 / 重力抛物线 / 时钟暂停桥 / 属性脏位同步）'),
  (0x4A0C80, 'IRDActiveObject & IRDCollisionObject 槽位 getter 族（本批 40 处与 src/cnrd L1 精修重复）'),
  (0x4A1100, None),
  (0x4A1210, 'clong/cfloat 属性块（构造 0x4A1210 / 析构 0x4A1670 / 62 槽解构 0x4A1F00）'),
  (0x4A1CA0, '属性读取族（vmt188/18C/190/194/198/19C/1A0/1A4/2E0/2E4/2EC/2F4/38C —— 防篡改管线消费侧）'),
  (0x4A22E0, '小结构解构族 + CNMagmaGhoul'),
  (0x4A2480, '怪物属性块大拷贝（vmt29C）'),
  (0x4A2B30, '关联容器/链表基础设施（refcount 封装 / 环形缓冲 / 红黑树模板实例 / std::string 辅助）'),
  (0x4A33F0, 'std::map/std::set 红黑树模板实例族（MSVC _Tree）'),
  (0x4A3E40, '红黑树容器构造/拷贝族 + 288 字节对象深拷贝'),
  (0x4A4430, '★ IRDCollisionObject 构造/析构（0x4A4430 / 0x4A4590 / 0x4A4710）'),
  (0x4A49B0, '★★ IRDActiveObject 构造/析构（0x4A49B0 / 0x4A4BD0 / 0x4A4E70）'),
  (0x4A4EA0, '★ IRDMonster 构造/析构（0x4A4EA0 / 0x4A50F0 / 0x4A5350）'),
  (0x4A5380, '怪物子类构造/析构批 A（CNFighterIllusion → CNGolemPlatany，0x4A5380-0x4A8010）'),
  (0x4A8040, '怪物子类构造/析构批 B（CNDollMaster → CNIceDoll_Sharedo）'),
  (0x4A8350, '怪物子类构造/析构批 C（CNFlyDragon 龙族 → CNTitan 巨人族，0x4A8350-0x4ABDC0）'),
  (0x4AEA30, '环形缓冲/容器排空族（0x4AEA30-0x4AF010，CNMonster 家族的回收/重置）'),
  (0x4AF010, '★★ IRDActiveObject 属性表导入/重算 + 本地玩家操作权限三位一体（0x4AF010-0x4AFEC0）'),
]

def section_for(va):
    """返回 (区起始 VA, 标题)，未命中返回 None"""
    best = None
    v = int(va, 16)
    for a, t in SECTIONS:
        if t and a <= v:
            best = (a, t)
    return best

# ---------------------------------------------------------------
# 5. 注入
# ---------------------------------------------------------------
src = open(SRC, 'r', encoding='utf-8').read()
if '// 【解读】' in src or '[已精修]' in src:
    import sys as _s; _s.exit('ABORT: 目标文件已含 L1 注释（注入非幂等），请先从 L2 备份恢复。')
lines = src.split('\n')
out = []
NEWHEADER = """// ============================================================
// 【40JP逆向源代码工程】src/game/rtti_core.cpp
// 模块: RTTI 类方法密集区（战斗对象核心 · 0x4A 区）
// 来源: work/dump_misc_a.txt  区块: 0x4A0000-0x4B0000（531 函数）
// 判定素材: work/decomp_core_vt.txt（核心三虚表 279 函数，含 RTTI 槽位名）
//
// 【L1 精修层】本文件已完成 rtti_core 段的 L1 精修（注释级），内容:
//   1) 与 src/cnrd/ 已精修函数的交叉引用（40 处，标 [已精修]，不重复正文）
//   2) RTTI 槽位名应用（55 处 IRDActiveObject__vmtXXX / IRDCollisionObject__vmtXXX）
//   3) 按功能块分区（17 段，见下方 SECTION 注记）
//   4) 30+ 行重点函数的开头解读注释（【推断】标记）
// 函数体逻辑与 L2 转写层逐字一致，未做任何改动。
//
// 本区的重要结构结论（本次精修推理）:
//   - 0x4A49B0 的 eh vector constructor iterator(this+676, 0x554, 4, ...) 直证
//     **IRDActiveObject 内嵌 4 个 1364 字节子对象**（0x554=1364），
//     与 vmt2A0（0x4A0C80: this+1364*a2+676）互证（docs/14 §4 记 5 个，本处实测 4）。
//   - 0x4AEDD0 写 +96 = 273（条件位集初值）—— 与 CNRDObject_vmt28（0x498DE0）的
//     273 掩码直证对应（docs/14 §2 交叉验证表闭环）。
//   - 0x4A5DC0 等哥布林族构造写 this+8600 第二 vftable，解释 (this+2150)=vftable 写法的由来。
//   - 0x4AF8B0 揭示两套 18 槽表（+6468 与 +7516）—— 当前值/基准值差分。
//   - 0x4AFEC0 七个全局字节（byte_9A0358..9A0460）按大状态 switch —— 本地玩家 UI 权限总闸。
//
// 函数数: 531（L1 精修覆盖 531；其中引用重复 40 / RTTI 命名 55 / 重点解读 331）
// ============================================================
"""


# 替换头部（前 9 行 = 旧 header）
hdr_end = 0
for i, l in enumerate(lines):
    if l.startswith('#include'):
        hdr_end = i
        break
out.extend(NEWHEADER.rstrip(chr(10)).split(chr(10)))
out.append('')
out.append('#include "../../common/arad_types.h"')
out.append('')
# 分区索引表
out.append('// ============================================================')
out.append('// SECTION 索引（按 VA 升序）')
out.append('// ============================================================')
for a, t in SECTIONS:
    if t is None:
        continue
    out.append('//   0x%06X — %s' % (a, t))
out.append('// ============================================================')
out.append('')
out.extend(lines[hdr_end + 1:])

# 逐函数注入注释
lines2 = out
res = []
i = 0
seen_secs = set()
n_refined = 0
n_rtti = 0
n_notes = 0
while i < len(lines2):
    l = lines2[i]
    # 原始结构: [分隔线] / "// 0xVA name" / [分隔线] / 函数体
    m = None
    if l.startswith('// ---') and i + 2 < len(lines2):
        mm = re.match(r'^// (0x[0-9A-F]{6}) (\S+)$', lines2[i + 1])
        if mm and lines2[i + 2].startswith('// ---'):
            m = mm
    if m:
        va, name = m.group(1), m.group(2)
        secinfo = section_for(va)
        if secinfo and secinfo[0] not in seen_secs:
            secva, sec = secinfo
            res.append('// ============================================================')
            res.append('// SECTION 0x%06X — %s' % (secva, sec))
            res.append('// ============================================================')
            seen_secs.add(secva)
        # 主注释行 —— 遵循任务规定的单行标注格式:
        #   // 0xVA name [RTTI: IRDActiveObject__vmtXXX · 槽位 xx] [已精修] 见 <文件> <槽位> —— <语义>
        r = rtti.get(va)
        ref = REFINED.get(va)
        n = NOTES.get(va)
        tail = ''
        if r:
            rname, slots = r
            n_rtti += 1
            tail += ' [RTTI: %s · 槽位 %s]' % (rname, slots)
        if ref:
            fn, slot, desc = ref
            tail += ' [已精修] 见 %s %s —— %s' % (fn, slot, desc)
            n_refined += 1
        if n:
            n_notes += 1
        res.append('// ------------------------------------------------------------')
        res.append('// %s %s%s' % (va, name, tail))
        if n:
            res.append('// ------------------------------------------------------------')
            res.append('// 【解读】')
            for nl in n:
                res.append('//   ' + nl)
        res.append('// ------------------------------------------------------------')
        i += 3          # 消费 分隔线 + 标记 + 分隔线
        continue
    res.append(l)
    i += 1

open(SRC, 'w', encoding='utf-8', newline='\r\n').write('\n'.join(res))
print('refined refs:', n_refined)
print('rtti annotated:', n_rtti)
print('notes:', n_notes)
print('sections:', len(seen_secs))
