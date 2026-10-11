# -*- coding: utf-8 -*-
# L1 精修注释 · passive_object.cpp · B 段（0x6056B0-0x60B920）
# 分区 4-7：霸体 / NPC 对话窗 / 神枪手子弹系 / 连接体·草丛·地图物件
C = {
"605B30": """霸体状态辅助：sub_42AF90（动画对象操作）+ sub_657740（资源/音效释放【推断】）——
CNSuperArmor 状态切换的公共收尾（被 vmt1C/vmt20/vmt14 多处调用）。""",
"605BB0": """CNSuperArmor 周期槽（vmt1C）：字符串证据「SUPERARMOR」（音效 TAG）。
sub_4920E0/sub_4926A0（定时器复位，vmtCC 族）/sub_4938F0（动画复位）——霸体光环的周期维持与超时解除。""",
"605D10": """CNSuperArmor 表现槽（vmt78 = 基类调 vt+116(0) 槽位）：字符串证据「Common/CommonEffect/Animation/SuperArmorBody.ani」。
sub_42AC90/sub_42FCB0（资源 228 门）/sub_431860（.ani 生成，P41）/sub_4924C0 —— 霸体躯体附体动画的生成与绑定。
SuperArmorEffect.ani（0x6059A0 侧）+ 本体两层动画构成霸体双表现。""",
"605EB0": """霸体数值辅助（纯字段操作 40 行）：霸体剩余量/层次计数的多槽位读写【推断：霸体叠加层数管理】。""",
"605FD0": """CNRDDialogueWindow 析构（vmt0 标量删除析构）：sub_491530 = 研究 20 P37-5 定案的渲染状态消费点
（枚举→D3D 渲染状态映射表）+ sub_491630 + sub_605F40/sub_69C690（UI 控件释放）——对话窗关闭时恢复渲染状态并析构控件树。""",
"6061D0": """对话窗辅助：无外部调用的字段操作（29 行）——【推断：当前选项/页索引推进】。""",
"606240": """CNRDDialogueWindow 每帧表现槽（vmt14）：dword_8F8520（主管理器）联动 ——
【推断：对话窗跟随 NPC/打字机刷新 tick】。""",
"606340": """对话窗控件批量操作（114 行）：sub_69A5F0/69A840/69C3E0/69C420/69C440/69C780/69C7B0/69CA20/69CA50（UI 控件族九连调）+
dword_99FCC8/99FCCC（对话 UI 状态全局）——【推断：选项按钮构建与文本装配】。""",
"606650": """对话文本表重建（与 0x602E80 同族）：sub_41FEB0 清空 + sub_44BD00/44BD30 逐词读取 + sub_558090（对话表查询）+
sub_5E51F0/sub_63A7C0 —— NPC 对话脚本的文本装载。""",
"6068A0": """对话表项读取：sub_44BD30 + sub_558520 —— 单条对话文本取出。""",
"606940": """对话脚本读取（67 行）：sub_558520 表项 + sub_472EA0/sub_47E1D0（脚本执行族）——
【推断：条件分支对话的脚本求值】。""",
"606B00": """对话树遍历（103 行，含递归调用自身）：sub_41A5E0（P24 防篡改写）+ sub_429830 + sub_606A90/6068A0/6061D0 +
word_77F804（配置字）——【推断：对话分支条件求值与跳转】。""",
"606CA0": """对话窗关闭路径：内嵌调试断言串「if (g_userManager->getMyUser() == NULL)」（IDA 反编译内联的调试表达式直证）+
sub_484F10（用户管理器查询）+ sub_605FA0/6061D0/606B00（树收尾）+ sub_5E5260（字符串析构）。""",
"606DF0": """对话窗布局重建（125 行）：sub_69C340/69C360/69C3A0/69C400/69C500/69C680（控件族）+ sub_606940（脚本读取）——
NPC 对话窗的控件树重建主流程。""",
"607020": """CNRDDialogueWindow 点击槽（vmt8）：字符串证据「CLICK_BUTTON2」（音效 TAG）+ sub_505FF0 + sub_69A650 +
sub_606DF0（布局重建）——按钮点击 → 播音 → 下一页刷新。【推断：对话推进按钮处理】""",
"6072C0": """子弹静态辅助（33 行，无外部调用）——【推断：枪种/弹种参数快查表】。""",
"607420": """CNGunnerBullet 攻击上下文装配槽（vmt158 = IRDCollisionObject 攻击上下文设置，研究 P29 已引 sub_4E7440）：
sub_47E720/47EF20/47EFD0（0x47 区命中几何扫描族【推断：攻击盒迭代/重叠测试】）+ sub_4C0FD0/sub_53F7A0。
子弹出膛时装配 attackinfo 上下文的主流程（187 行）。""",
"607A70": """子弹命中盒扫描变体：sub_47E720/sub_47EFD0 —— 判定盒迭代（0x607420 同族精简版）。""",
"607B00": """CNGunnerBullet 每帧 tick（vmt10）：sub_491F80/492080/4938F0（动画对象复位/绑定族）+ sub_699000（clock_now，P9 时间域）+
dword_7930D4（时间域 y 压缩系数，与 vmtA4 朝向角同源）——【推断：弹道位移的时间域积分】。""",
"607CD0": """CNGunnerBullet 观察者通知槽（vmt1F8 = 基类 18 观察者通知族，IRDActiveObject.h 定案）：
sub_4DAB60（combat_core 播报族）+ sub_607A70 —— 子弹命中/消亡时的观察者播报。""",
"607E70": """CNGunnerBullet 主循环槽（vmt14C，169 行）：字符串证据「Character/Gunner/Effect/Animation/Bullet/RandomShoot.ani」
（随机弹射动画）。sub_431860 动画生成 + sub_4600B0 演播层 + sub_600890（基类出生播音，P32）+ sub_4A0B60/sub_607990 ——
子弹飞行中的动画/演出编排。【推断：弹道动画随机化】""",
"6081B0": """弹种×枪种动画路径拼接：字符串证据「Character/Gunner/Effect/Animation/Bullet/」+ 属性名
「Flame | Freeze | Silver」× 枪名「Pistol.ani | Musket.ani | BowGun.ani」——
九种组合的路径查表（dword_796E50 = 路径根）。属性弹与枪种的二维矩阵直证。""",
"608300": """命中特效路径拼接（与 0x6081B0 同构）：字符串证据「Character/Gunner/Effect/Animation/HitEffect/」+
「Flame | Freeze | Silver | Normal」×「BowGun.ani」族 —— 子弹命中特效的属性×枪种查表。""",
"608430": """CNGunnerBullet 消亡联动槽（vmt12C，124 行）：字符串证据
「Character/Gunner/Effect/Animation/HitEffect/CollisionFloor.ani | CollisionWall.ani」——撞地/撞墙两种消亡特效。
sub_45FE80 演播层 + sub_600790（基类移除联动）+ sub_608300（命中特效查表）+ sub_4E4A50（动作包）——
子弹寿命终结的完整演出链。""",
"608730": """银弹拖尾粒子：字符串证据「Character/Gunner/Effect/Particle/SilverBulletParticle.ptl」（研究 20 P36 .ptl 资源）+
sub_428F30 发射器生成 + sub_6081B0（弹种动画）+ sub_41A5E0（P24 写）——银弹专属粒子拖尾的装载。""",
"608890": """CNGunnerBullet 命中表现槽（vmt1F4，174 行）：sub_45FE80 演播层 + sub_608300（命中特效查表）+
sub_44B6C0/sub_44E0D0（帧事件族，P26）/sub_4A06D0（挂点跟随族）/sub_5FF9C0/sub_607990/607A70 ——
子弹命中目标的特效+帧事件装配主流程。""",
"608CD0": """CNGunnerBullet 碰撞响应槽（vmt2A8，70 行）：sub_6073E0/sub_6078B0（弹道反弹族【推断】）+ sub_608730（银弹粒子）+
sub_41A5E0（P24 写）——子弹撞墙反弹并残留拖尾。""",
"608FE0": """CNGunnerSubWeaponBullet 主循环槽（vmt14C，101 行）：sub_608EA0/6072C0 + sub_4E5F70（硬直率掷）+ sub_699000 ——
副武器（Musket/BowGun）子弹的飞行/命中编排，与主武器 CNGunnerBullet 共享几何族。""",
"6091B0": """CNGunnerSubWeaponBullet 消亡联动槽（vmt12C）：字符串证据「DIFFUSION_GUN_EX」（音效 TAG —— 散弹枪 EX）。
sub_42F220 + sub_4918E0（动画盒世界坐标，P37-2 obj+60 边界族）——散弹 EX 的消亡演出。""",
"6093E0": """CNGunnerSubWeaponBullet 命中表现槽（vmt1F4）：字符串证据「DIFFUSION_GUN_CHARGE」（蓄力音 TAG）+ sub_4A06D0（挂点跟随）。""",
"609550": """CNGunnerSubWeaponBullet 碰撞响应槽（vmt2A8）：字符串证据「R_DIFFUSION_GUNA」（音效 TAG）+ sub_608F10。""",
"609630": """蓄力参数插值：sub_4049C0（P30/P36 同款 lerp）——【推断：散弹枪 EX 蓄力 0→1 进度的插值】。""",
"609820": """蓄力辅助（35 行）→ 转调 sub_609630 —— 蓄力曲线消费侧。""",
"609980": """CNRDJoint 命中结算槽（vmt2BC）：sub_45EBE0（飘字参数）+ sub_609920 ——
连接体（Joint，链条/绳索类 PO【推断】）被攻击时的结算。""",
"609BA0": """连接体/草丛辅助（28 行，无外部调用）——【推断：状态字段复位】。""",
"609C20": """定时器复位辅助：sub_4926A0（vmtCC 定时器链表复位，CNRDObject.h 定案）——草丛摇摆计时复位。""",
"609CF0": """CNCutableGrass 表现壳槽（vmt200）：sub_609CA0 + sub_4600B0（演播层）——
草丛被攻击时的受击表现（可割草 PO）。""",
"609E20": """CNCutableGrass 主循环槽（vmt14C，104 行）：sub_609AF0/sub_609C20 ——
【推断：风中摇摆 + 被割状态机（割草小游戏的 PO 载体）】。""",
"609FF0": """草丛定义表重建（与 0x602E80/0x606650 同族）：sub_41FEB0/44BD00/44BD30 逐词读取 + sub_52E630 +
sub_5E51F0/sub_63A7C0 —— 地图可割草丛的参数装载。""",
"60A310": """草丛/物件资源装载（72 行）：sub_41ACF0（资源管理器）+ sub_42FD40 + sub_493A60 + sub_4C3640（动画对象构造，
与 0x600C20 生成工厂同款的 0x12C 构造）——【推断：草丛动画实例化】。""",
"60A4C0": """草丛被割表现（87 行）：sub_45FE80（演播层）+ sub_4E2130 —— 割断瞬间的叶片飞散演出【推断】。""",
"60A670": """CNCutableGrass 消亡联动槽（vmt12C）：sub_609CA0 + sub_60A4C0（被割表现）+ sub_4600B0 —— 草丛消失演出链。""",
"60A800": """地图物件状态广播：sub_432FA0（UDP 广播）+ byte_73B39C（状态字）——
CNMapObject（可破坏地图物件）的破坏状态同步包【推断】。""",
"60A950": """地图物件状态机片段（30 行，无外部调用）——【推断：破坏阶段推进】。""",
"60AA00": """地图物件辅助：sub_41BFE0 ——【推断：物件资源/属性查询】。""",
"60AB60": """地图物件辅助（31 行，无外部调用）——【推断：破坏掉落参数】。""",
"60ABE0": """状态广播包装（35 行）→ 转调 sub_60A800 —— 物件破坏广播的对外入口。""",
"60AC80": """CNMapObject 表现槽（vmt2DC）：sub_41ACF0（资源管理器）+ sub_4600B0（演播层）——
物件破坏时的碎块/烟尘演出【推断】。""",
"60ADE0": """CNMapObject 槽（vmt2B4）：sub_4926A0（定时器复位）——物件受击计时复位。""",
"60AE90": """CNMapObject 表现槽（vmt2E0）：sub_41ACF0 + sub_4600B0 —— 物件另一表现分支（与 0x60AC80 成对）。""",
"60AFA0": """地图物件定义表装载（111 行）：sub_4DA030/sub_5043B0/sub_504450（表驱动族）+ sub_52E630 + sub_59EE30 +
sub_5E51C0/sub_623C10 —— 与怪物解析器（monster_parser）同族的表装载管线。【推断：物件 hp/掉落/破坏形态配置】""",
"60B3D0": """CNMapObject 装载槽（vmt2E4）：sub_41ACF0/sub_41B0D0（资源管理器双查）+ sub_4C3640（动画对象构造）——
物件动画实例化。""",
"60B520": """物件破坏分发（93 行）：sub_428220 + sub_60A9B0/60AAB0 ——
【推断：按破坏形态分发碎块/掉落/广播】。""",
"60B710": """物件破坏表现（70 行）：sub_45FE80（演播层）+ sub_4E2130 + sub_4E7E40 ——
破坏瞬间的演出与位移同步【推断】。""",
}
