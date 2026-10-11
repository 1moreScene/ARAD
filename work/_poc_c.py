# -*- coding: utf-8 -*-
# L1 精修注释 · passive_object.cpp · C 段（0x60B920-0x616390）
# 分区 8-9：元素与自然现象 PO / 召唤·诅咒·投技·鬼神·气功 PO
C = {
"60B920": """CNFireMissile 主循环槽（vmt14C，109 行）：sub_4600B0（演播层）+ sub_600890（基类出生播音）——
火焰导弹（FireMissile）的飞行编排；出生粒子与音效见 0x60BA80（FireMissileParticle.ptl + SIZZLE_CREATE）。""",
"60BA80": """火弹出生粒子：字符串证据「PassiveObject/Common/Particle/FireMissileParticle.ptl |
FireMissileParticleLight.ptl」（双发射器：本体+光晕）+「SIZZLE_CREATE」（音效 TAG）+ sub_4049C0（P36 出场 lerp 插值）。
对应研究：P36 发射器资源生态直证。""",
"60BC50": """CNFireExplosion 命中结算槽（vmt2BC，103 行）：sub_4922F0（动画定位）+ sub_4E5F70（硬直率掷）+
dword_78FE30/34（全局攻击上下文【推断：伤害倍率对】）——火焰爆炸对命中目标的结算。""",
"60BE80": """CNFireExplosion 主循环槽（vmt14C）：sub_4600B0 + sub_4922F0 + sub_4E77D0 ——
爆炸扩散的动画推进【推断：火焰圈半径随帧扩张】。""",
"60BFF0": """火爆生成辅助：字符串证据「NAPALM_EXPLOSION | FIRE_EXPLOSION_CREATE」（双音效 TAG：凝固弹爆+火爆生成）。
sub_5CC5A0 + sub_41ACF0/sub_4C3640（动画对象）——火爆体的实例化。""",
"60C140": """CNIceMissile 主循环槽（vmt14C）：sub_4600B0 —— 冰弹飞行编排；出生粒子见 0x60C230（IceMissileParticle 族 + ICICLE_CREATE）。""",
"60C230": """冰弹出生粒子：字符串证据「PassiveObject/Common/Particle/IceMissileParticleLight.ptl |
IceMissileParticleTail.ptl」（光晕+拖尾双发射器）+「ICICLE_CREATE」（音效 TAG）+ sub_4049C0 lerp。""",
"60C3F0": """CNThunderBolt 碰撞响应槽（vmt2A8）：sub_47E720/47EF20/47EFD0（命中几何扫描族）——
落雷（ThunderBolt）对目标的判定；落地震屏与 EarthQuake.ani 在 0x60C6A0。""",
"60C590": """落雷生成辅助：字符串证据「THUNDER_STRIKE」（音效 TAG）+ sub_4049C0 lerp —— 雷击参数插值。""",
"60C6A0": """CNThunderBolt 主循环槽（vmt14C）：字符串证据「PassiveObject/Common/Animation/EarthQuake.ani」+
sub_424120（研究 20 P41 定案：[shake screen] 震屏入口）+ dword_8F82B0 —— 落雷落地时的震屏与余波。""",
"60C8C0": """CNParticleFactory 主循环槽（vmt14C，82 行）：sub_425050 + sub_4600B0 ——
【新发现】『粒子工厂 PO』：以 PO 形态周期性发射粒子（P36 发射器的对象化封装，地图机关的喷泉/毒雾源）。""",
"60CA50": """粒子工厂装载：sub_41ACF0 + sub_4C3640 —— 工厂体的动画/伴随对象实例化。""",
"60CB80": """CNBlast 初始化（vmt10）：爆风（冲击波）PO 的初值装配。""",
"60CC40": """CNBlast 主循环槽（vmt14C，156 行）：字符串证据「Common/CommonEffect/Animation/LevelUp.ani |
Blast1.ani | Blast2.ani」+「LEVEL_UP | SHOUTING」（双音效 TAG：升级/怒吼）。
sub_44B6C0/44E0D0/44E1F0（帧事件族，P26）+ sub_4910A0/49C5F0/49C620 + sub_600890 ——
升级冲击波/怒吼冲击波的扩散演出（双动画层 Blast1/2 叠放，P33 叠放风格）。""",
"60CF90": """CNEarthQuake 主循环槽（vmt14C）：字符串证据「TAU_EQUAKE | TAU_STONEDROP」（音效 TAG —— ToU 氏族地震/落石）+
sub_424120（震屏）——地图机关地震 PO。""",
"60D260": """CNDangerousGas 主循环槽（vmt14C，91 行）：字符串证据「PassiveObject/Common/%s」（路径模板格式化）+
「CLOUD_KILL_LOOP」（音效 TAG —— 毒云循环）。sub_428F30（粒子发射器）+ dword_796E50 ——
毒瓦斯云的周期喷发（云杀 = 猛毒地带机关）。""",
"60D4C0": """CNIceNiddleGroup 主循环槽（vmt14C）：sub_4E4A50（动作包）+ sub_699000（clock_now）——
冰针群（IceNiddleGroup）的齐射调度；命中溅射见 0x60D5A0（WaterHitParticleLIght.ptl）。""",
"60D5A0": """冰针命中溅射槽（vmt2B4）：寿命检查（+170<=0 → vt+312 自毁）+ 惰性创建粒子发射器
（首次才 operator new(0x12C=300B) → sub_428F30，字符串证据「Common/HitEffect/Particle/WaterHitParticleLIght.ptl」）
→ vt+12 复位 / vt+40 设位 / vt+92 取名 → sub_4600B0 演播层（层 2）。与 0x61EC80（AraclothoMissile）同构。""",
"60D6A0": """冰针光晕装载：字符串证据「PassiveObject/Common/Animation/IceNiddleGlow.ani」+「ICE_NIDDLE_HIT」（音效 TAG）。
sub_42FCB0（资源 228 门）→ sub_431860 生成 → vt+204 绑定；波次参数 sub_4049C0 按波号插值（+181）。""",
"60D840": """冰针参数插值：sub_4049C0 ——【推断：波次间冰针尺寸/间隔插值】。""",
"60D9A0": """CNEarthFire 主循环槽（vmt14C）：字符串证据「EARTHFIRE_LOOP」（音效 TAG —— 地火循环）+
「PassiveObject/Common/Particle/FireMissileParticleFloor.ptl」（地面火粒子，复用火弹地面层）。
sub_428F30 发射器 + sub_699000 —— 地面持续燃烧带（地火机关）。""",
"60DB20": """CNShootingStar 主循环槽（vmt14C）：sub_424120（震屏）+ sub_4600B0 ——
流星（ShootingStar）坠落演出；出生粒子见 0x60DC60（ShootingStarFire.ptl + SSTAR_DROP）。""",
"60DC60": """流星出生粒子：字符串证据「PassiveObject/Common/Particle/ShootingStarFire.ptl」+「SSTAR_DROP」（音效 TAG）+
sub_4049C0 lerp —— 流星火尾的出场插值。""",
"60DDC0": """CNShootingStarExp 主循环槽（vmt14C）：sub_432FA0（UDP 广播）+ sub_4600B0 ——
流星爆的落点广播与爆炸演出；粒子三件套见 0x60DFA0。""",
"60DFA0": """流星爆粒子三件套：字符串证据「PassiveObject/Common/Particle/ShootingStarExpSmoke.ptl |
ShootingStarExp.ptl | ShootingStarExpEtc.ptl」+「SSTAR_HIT」（音效 TAG）+ sub_4049C0 lerp ——
烟/爆心/飞散三层粒子（P36 资源生态直证）。""",
"60E120": """CNBoomerang 主循环槽（vmt14C）：字符串证据「BOOMERANG_SHOOT_LOOP | BOOMERANG_BACK_LOOP」
（去程/回程双循环音效 TAG —— 掷出与折返各挂一个循环音）。sub_4910A0 —— 回旋镖往返的音画同步。""",
"60E2D0": """弹跳体参数装载（93 行）：sub_423A20（UDP 广播）+ sub_432FA0 + sub_4926A0（定时器复位）+ dword_796E50 ——
【推断：刺猬弹（HEDGEHOG，见 0x60E790）的弹跳参数与路径装载】。""",
"60E4E0": """CNBouncerLooseSerimalion 主循环槽（vmt14C）：sub_4926A0 —— 弹跳计时复位。
【推断：LooseSerimalion = 松脱弹跳体变体】。""",
"60E640": """CNBouncerLooseSerimalion 消亡联动槽（vmt12C）：sub_4600B0（演播层）+ sub_4E77D0 +
dword_8F82A8/8F82AC（弹跳体池化槽）——弹跳体消失演出。""",
"60E790": """CNBouncerLooseSerimalion 攻击判定槽（vmt214）：字符串证据「HEDGEHOG_BUMP」（音效 TAG —— 刺猬碰撞）+
dword_8F8280/8F8284（研究 20 P33 定案的池化火花槽 8F8280/84 = 钝器双层火花）——
弹跳体碰撞时复用池化火花演出（零分配，P33 池化机制直证）。""",
"60E900": """CNCreateOnDestroyObject 消亡联动槽（vmt12C）：自身消亡时生成后继 PO 的包装器（sub_4E4A50 发包收尾）——
【新发现】『死亡衍生』机制的基类：爆炸物→残留火海等链式生成的设计。""",
"60E9A0": """死亡衍生辅助（27 行）：后继 PO 的参数转抄【推断】。""",
"60EA70": """CNSunkenRock 主循环槽（vmt14C，101 行）：字符串证据「DOLL_STONE | TRAP_STONE_BREAK」（音效 TAG ——
人偶石/陷阱石破裂）。sub_4E4A50（动作包）——沉陷岩石陷阱的破裂状态机。""",
"60EC70": """人偶石就绪辅助：字符串证据「DOLL_STONE_READY」（音效 TAG）+ sub_4049C0 lerp —— 石块升起动画参数。""",
"60ED40": """CNRedRock 主循环槽（vmt14C）：字符串证据「FIRELOCK_ROLLING_LOOP」（音效 TAG —— 火轮滚动循环）+
dword_8F8520 —— 红石（火轮）滚动机关的循环维持。""",
"60EE50": """CNRedRock 命中表现槽（vmt1F4）：sub_4600B0 —— 火轮碾压目标的演出。""",
"60EF20": """火轮投掷辅助：字符串证据「THROW_FIRELOCK」（音效 TAG）+ sub_4049C0 lerp —— 投掷初速参数。""",
"60EFF0": """CNDollMasterBubble 命中表现槽（vmt1F4）：字符串证据「CLAY_HIT」（音效 TAG —— 黏土命中）。
sub_44B6C0/44E0D0（帧事件）+ sub_45FE80（演播层）——人偶师黏土泡泡被击破的演出。""",
"60F110": """CNMagicTargetWarning 命中结算槽（vmt2BC）：sub_493D00（研究 20 P38-2 定案的动画速率写者）——
警示圈以速率控制收缩节奏（魔法瞄准 AOE 预警圈的 PO 载体【推断】）。""",
"60F1E0": """CNMagicTargetWarning 初始化（vmt10）：警示圈初值装配。""",
"60F270": """CNMagicTargetWarning 主循环槽（vmt14C）：sub_4E4A50（动作包）+ sub_4E4C10 ——
预警圈的同步与收缩推进【推断】。""",
"60F410": """CNForceField 初始化（vmt10）：念气罩初值装配。""",
"60F510": """CNForceField 主循环槽（vmt14C，143 行）：字符串证据「PassiveObject/Common/Animation/ForceField/
dirt.ani | blast.ani | chargecircle.ani | Field.ani」——蓄力圈→罩体→碎裂四段资源全套。
sub_44B6C0/44E0D0（帧事件）+ sub_45FE80（演播层）+ sub_600890 —— 念气罩全生命周期演出。
音效 TAG 见 0x60F510 区（FORCE_FIELD_CIRCLE / FORCE_FIELD_FIRE）。""",
"60F8E0": """CNCircleAppendageMaker 碰撞槽（vmt2C8）：sub_47E720/47EF20（命中几何扫描族）——
圆环附属生成器的范围判定【推断：抓取圈扫描】。""",
"60FC30": """附属伤害输入（68 行）：sub_4E4630 + sub_4E5F70（硬直率掷）——
附属圈对圈内目标的持续伤害输入【推断：DOT 圈】。""",
"60FDE0": """★ CNCircleAppendageMaker 命中表现槽（vmt1F4，250 行）：字符串证据「Common/CommonEffect/BuffIcon.img」（buff 图标图集）+
「BUFF_FASTFIST | BUFF_FATAL_POINT」（音效 TAG —— 快拳/要害）。
sub_517740/sub_5C24B0/sub_5C2D60（buff 应用族）+ sub_4B3C90/sub_4C0400 + sub_60FA60/60FB40 ——
给命中目标挂 buff 图标与状态的双分支（快拳/要害二选一）。
【推断：气功师/武神的范围增益圈——圈内队友获增益状态的 PO 载体】。""",
"610320": """CNToothSFish 命中表现槽（vmt1F4，71 行）：食人鱼齿（ToothSFish）咬合目标的演出【推断：地图水道食人鱼陷阱】。""",
"6106B0": """CNCurseBomb 主循环槽（vmt14C）：sub_4E4A50（动作包）——诅咒炸弹的滞空/引信推进；爆炸音 BOMB_EXPLOSION 见 0x610740。""",
"610740": """诅咒炸弹爆炸辅助：字符串证据「BOMB_EXPLOSION」（音效 TAG）+ sub_4015C0（字符串构造）——爆炸播音的 TAG 装配。""",
"6107E0": """CNCurseBombSub 主循环槽（vmt14C，80 行）：sub_41ACF0 + sub_428F30（粒子发射器）+ sub_4600B0 ——
诅咒炸弹子体（分裂弹）的飞行/爆炸演出。""",
"610AB0": """CNSummonMonsterWithEffect 主循环槽（vmt14C）：召唤兽带特效生成（SummonMonsterWithEffect）的存活推进；
三段法阵+四属性召唤动画在其 vmt2BC（0x610C00）。""",
"610C00": """★★ 召唤阵演出槽（vmt2BC，138 行）—— 0x60 区字符串证据的『SummonCircle 三段动画 + 四属性 Spirit 召唤』本体：
字符串证据「Common/CommonEffect/Animation/SummonCircleBack.ani | SummonCircle.ani | SummonCircleFront.ani」
（法阵背景/中景/前景三段叠层，P33 叠放风格）+「Monster/Spirit/FireAnimation/Summon.ani |
WaterAnimation/Summon.ani | DarkAnimation/Summon.ani | LightAnimation/Summon.ani」（火/水/暗/光四属性精灵召唤动画）。
sub_42ABE0 + sub_42FCB0（资源 228 门）→ sub_431860 批量生成 + sub_5FFF80 ——
召唤兽出场：法阵三层 + 按属性选精灵动画的完整演出管线。""",
"610FF0": """CNBodyHit 命中结算槽（vmt2BC）：sub_41A5E0（P24 防篡改写）——
投技身体判定（BodyHit）命中时直写属性槽【推断：投技伤害按目标属性折算】。""",
"611200": """投技摔击动画选择：字符串证据「Wall | Floor」+「Character/Fighter/Effect/Animation/ThrowEnemy%s%d.ani」——
摔墙/摔地二分支 × 动画编号（格式化路径拼接）——柔道家投技的落点动画矩阵（dword_796E50 路径根）。""",
"611400": """CNBodyHit 主循环槽（vmt14C，219 行）：sub_492E80/sub_4938F0（动画族）+ sub_4E5F70（硬直率掷）+
sub_6111A0/611200（摔击动画选择）+ sub_600890（出生播音）——
抓投过程中身体判定盒的跟随与伤害结算主流程（柔道/摔技系共用）。""",
"6119F0": """投技表现参数装载（92 行）：sub_423A20（UDP 广播）+ sub_45EBE0（飘字参数）+ dword_796E50 ——
【推断：投技命中点的广播与飘字装配】。""",
"611C70": """CNTrackingIceWave 主循环槽（vmt14C，234 行）：字符串证据「ICEWAVE_CREATE」（音效 TAG）。
sub_4923C0/4926A0（定时器）+ sub_6119F0 + dword_8F82A4/8F82A8/8F82AC（冰浪池化槽三连）+ sub_698F90（时钟复位，P36）——
追踪型冰浪（自动寻敌）的推进与池化复用。""",
"6120F0": """冰浪路径粒子：字符串证据「PassiveObject/Monster/Soldier/Particle/IceWaveMovePath.ptl」
（资源路径直证：冰浪 PO 归属 Monster/Soldier 系）+ sub_445B90/sub_445D00（156B 粒子分配，研究 20 P36 ⑤ 直证）+
sub_4A0B60 —— 冰浪移动轨迹的粒子铺设。""",
"6122A0": """CNSingleIceWave 命中结算槽（vmt2BC）：sub_4922F0 + dword_78FE30/34（全局攻击上下文）——
单体冰浪的命中结算。""",
"612390": """CNSingleIceWave 主循环槽（vmt14C，161 行）：字符串证据「ICEWAVE_CREATE」。
sub_4922F0/sub_493D00（速率写者，P38）+ sub_4E77D0 + sub_607260/6072C0 + dword_8F82A4/A8/AC ——
单体冰浪的推进（速率控制浪宽）。""",
"612770": """CNEventBarrier 主循环槽（vmt14C，118 行）：sub_60B6B0（物件族，与 CNMapObject 共享）+ sub_4E4A50 +
sub_698F90/699000（时钟复位/读数）——事件屏障（副本机关墙）的开合状态机。""",
"6129E0": """CNIceWave 碰撞响应槽（vmt2A8）：sub_47E720/47E9D0/47EF20/47EFD0（命中几何族）+ sub_4924C0 +
dword_795BE0 —— 冰浪对目标的推移判定【推断：浪推位移】。""",
"612B70": """CNIceWave 主循环槽（vmt14C，198 行）：字符串证据「ICEWAVE_CREATE」。
sub_4E4A50 + sub_4E77D0 + dword_795BE0 + dword_8F82A4/A8/AC —— 多段冰浪的推进/消亡主流程。""",
"612FC0": """CNSuplexSub 命中结算槽（vmt2BC）：sub_47EF20/47EFD0 + sub_4922F0 —— 背摔子判定的结算。""",
"6130C0": """CNSuplexSub 主循环槽（vmt14C，182 行）：字符串证据「Character/Fighter/Effect/Animation/SuplexDust.ani |
JumpSuplexCrashUnder.ani | JumpSuplexCrashOver.ani | Animation/PoweredSuplexSub.ani」+ 调试格式串「SIZE RATE %f」
（运行时尺寸缩放输出直证）——柔道家背摔/跳背摔的全套动画（尘土+落地下/上层双画）。
sub_4924C0（尺寸缩放）+ sub_600890。""",
"6134D0": """CNEnergyBall 命中表现槽（vmt1F4）：sub_4E4A50 —— 能量球（EnergyBall）命中表现【推断：念弹系】。""",
"6135F0": """CNThrowSand 碰撞响应槽（vmt2A8）：sub_47E720/47EF20/47EFD0 —— 撒沙（ThrowSand）致盲判定【推断】。""",
"6136C0": """CNThrowSand 主循环槽（vmt14C）：sub_4600B0 + dword_8F82C0 —— 撒沙尘土演出与池化槽。""",
"613800": """CNEnergyField 碰撞响应槽（vmt2A8）：sub_491C70/4920E0/4921E0（动画属性族）+ sub_607260/607290 ——
能量场（EnergyField，ENERGY_EXPLOTION TAG 见 0x6139F0 区）对场内目标的判定。""",
"613960": """CNEnergyField 主循环槽（vmt14C）：sub_4910A0 + sub_6072C0 —— 能量场的维持 tick。""",
"613A70": """CNBlastBlood 主循环槽（vmt14C）：字符串证据「FIRE_PILLAR_UP」（音效 TAG —— 血爆升起的火柱）。
sub_47DCD0/47DD30 + sub_4910A0 —— 狂战士系血爆（BlastBlood）的火柱升起演出。""",
"613B70": """血爆生成辅助：字符串证据「BLOOD_CIRCLE_CREATE」（音效 TAG —— 血圈生成）+ sub_491C70 + sub_607260/607290 ——
血爆中心的血圈演出装配。""",
"613D70": """CNBlastBloodSub 命中表现槽（vmt1F4，74 行）：字符串证据「Animation/BlastBloodHit1.ani | BlastBloodHit2.ani」
（双层命中动画叠放）。sub_44B6C0 + sub_45FE80（演播层）——血爆子体的飞溅演出。""",
"613FB0": """鬼神辅助：sub_4E4630 ——【推断：鬼神附加体的位移/朝向同步】。""",
"614180": """鬼神数据辅助（38 行，无外部调用）——【推断：鬼神等级/持续时间的字段折算】。""",
"6141F0": """CNGhost 初始化（vmt10）：sub_4049C0 lerp + sub_491F80/4938F0（动画复位）+ sub_699000 ——
鬼神（Ghost）出场参数插值。""",
"6142A0": """鬼神选型辅助（60 行，无外部调用）——【推断：卡赞/萨亚/布雷门三鬼神的分支参数】。""",
"6143B0": """鬼神辅助（31 行，无外部调用）——【推断：鬼神光环半径参数】。""",
"614460": """鬼神阵生成辅助：字符串证据「G_GHOST_CIRCLE」（音效 TAG —— 鬼神阵）。
sub_491C10 + sub_607260 —— 鬼泣召唤阵的演出装配。""",
"614660": """鬼神目标 map 管理（106 行）：STL 异常串「map/set<T> too long」直证 std::map 使用 +
sub_4296E0/sub_433130/sub_5648C0 —— 鬼神作用目标的有序表维护（插入/查重）。""",
"614810": """鬼神目标表消费（59 行）：sub_4A3B30 + sub_614660 —— 遍历目标表施加鬼神效果【推断】。""",
"6148D0": """鬼神辅助：sub_41C5E0 + sub_52E960 + sub_614180 —— 目标效果施加的收尾【推断】。""",
"614990": """鬼神目标表预处理（52 行）：sub_4B3C90 + sub_614090 ——【推断：目标过滤/优先级】。""",
"614AB0": """CNGhost 消亡联动槽（vmt12C）：sub_6142A0/614930 —— 鬼神消散时的状态回滚【推断】。""",
"614B30": """鬼神主体装配（99 行）：sub_4BE520 + sub_614050/6143B0/614810/614990 ——
卡赞（力）/萨亚（冰）/布雷门（束缚）三鬼神共用的装配管线（分支在 6142A0）。""",
"614C90": """★ CNGhost 主循环槽（vmt14C，150 行）：字符串证据「GHOST_KAJAN | GHOST_SAYA | GHOST_BREMEN」
（音效 TAG —— 鬼泣三鬼神：卡赞/萨亚/布雷门，研究 20 系外的独立音效证据）。
sub_47E720/47EFD0（命中几何）+ sub_4E5F70（硬直率掷）+ sub_613FB0/614B30 ——
鬼神持续期间的周期判定与效果施加。""",
"615000": """CNFireWave 碰撞响应槽（vmt2A8）：sub_47E720/47EF20/47EFD0 + sub_4921E0 —— 火浪（FireWave）推移判定。""",
"615090": """CNFireWave 命中表现槽（vmt1F4）：sub_4E4A50 —— 火浪命中发包。""",
"615180": """CNFireWave 主循环槽（vmt14C）：字符串证据「FIREWAVE_MOVING」（音效 TAG —— 火浪推进循环）。
sub_4600B0 + sub_4920E0 —— 火浪推进演出；出生粒子四件套见 0x6152E0。""",
"6152E0": """火浪粒子四件套：字符串证据「Particle/FireWaveParticle.ptl | FireWaveParticleTail.ptl |
FireWaveParticleLight.ptl | FireWaveParticleFloor.ptl」+「FIREWAVE_CREATE」（音效 TAG）——
本体/拖尾/光晕/地面层四发射器（P36 发射器组合的典型布局）。""",
"6154D0": """CNSpiralNen 主循环槽（vmt14C）：sub_6072C0 + sub_699000 —— 螺旋念气（SpiralNen，气功师念气系【推断】）推进。""",
"6155D0": """CNSpiralNen 碰撞响应槽（vmt2A8）：sub_47E720/47E9D0/47EF20/47EFD0 + sub_607260 —— 螺旋念气命中判定。""",
"615760": """CNClosePunchSub 消亡联动槽（vmt12C）：近身拳子判定（ClosePunchSub）的收尾。""",
"6157C0": """CNClosePunchSub 碰撞响应槽（vmt2A8）：sub_428220 —— 近身拳（贴身打击）判定响应。""",
"615930": """CNGunnerLaser 主循环槽（vmt14C，148 行）：字符串证据「G_LASERGUNB | G_LASERGUNA」（音效 TAG ——
激光枪 B/A 双版本）。sub_4926A0/sub_4938F0 + sub_600890 —— 激光束的持续照射编排（激光 PO 化直证）。""",
"615D10": """激光辅助（41 行）：sub_47E720/47EF20（命中几何）+ sub_4922F0 + sub_698F90（时钟复位）——
激光束扫描的时钟重置【推断：束宽帧同步】。""",
"615E20": """CNNenGuard 消亡联动槽（vmt12C）：sub_41C5E0 + sub_4BE520 + sub_52E6A0 ——
念气护盾（NenGuard，气功师念气罩的护盾侧）消散时的状态回收。""",
"615ED0": """CNNenGuard 主循环槽（vmt14C，136 行）：sub_491C70/493A60（动画族）+ sub_52E6A0 + sub_614810（鬼神目标表复用）+
sub_615CA0 + sub_698F90/699000（时钟）—— 护盾持续期间的吸收/反弹编排【推断】。""",
"6161F0": """落点调度器（71 行）：消费 dword_7930D0（研究 20 P36 ⑤ 直证的 P9 全局时间域 tick 基址）+ unk_9B93A8 参数块 +
sub_490410/sub_6BFF30 —— 墓碑雨（TombStoneRain，下一分区）落点的时间域插值调度【推断】。""",
}
