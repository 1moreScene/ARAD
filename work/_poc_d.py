# -*- coding: utf-8 -*-
# L1 精修注释 · passive_object.cpp · D 段（0x616390-0x620000）
# 分区 10-11：法师投射物与陷阱 / 怪物 PO 弹幕·陷阱大全
C = {
"616390": """CNTombStoneRain 主循环槽（vmt14C）：sub_432FA0（UDP 广播）+ sub_6161F0（落点调度，消费 P9 时间域）——
墓碑雨（鬼泣「墓碑」大招的落石阵列）的落点时间域编排。""",
"616520": """CNTombStone 碰撞响应槽（vmt2A8）：sub_47E720/47EF20/47EFD0 —— 单块墓碑的砸落判定。""",
"6165C0": """CNTombStone 主循环槽（vmt14C）：字符串证据「TOMBTONE_STUCK | TOMBSTONE_DIG」（音效 TAG —— 插地/挖掘，
TOMBTONE 为原版拼写）+ sub_424120（震屏）—— 墓碑落地插地的震屏演出。""",
"6166E0": """CNDropBomb 消亡联动槽（vmt12C，88 行）：字符串证据「%s/%s」（路径拼接模板）。
sub_5C6A50/5C6A90/5DAE20 + sub_47E720 族 + sub_4E4A50 —— 落弹（空投炸弹）的爆炸分发【推断：弹坑/残留物生成】。""",
"616950": """CNDropBomb 主循环槽（vmt14C）：字符串证据「BOMB_DROP」（音效 TAG）+ sub_4600B0 —— 空投炸弹下落演出。""",
"616BA0": """CNAshenForkSub 碰撞响应槽（vmt2A8）：sub_47E9D0/47EFD0 —— 灰烬叉子判定（SAJAHU_SPREAD TAG 见 0x616C60）。""",
"616C60": """灰烬辅助：字符串证据「SAJAHU_SPREAD」（音效 TAG —— 萨朱扩散【推断：灰烬散布】）。
sub_4922F0 + sub_4015C0 —— 散布参数装配。""",
"616ED0": """CNGrandWave 碰撞响应槽（vmt2A8）：sub_4049C0（lerp）+ dword_795A0C ——
大波动（GrandWave）的浪高插值与判定。""",
"616FE0": """CNGrandWave 主循环槽（vmt14C）：sub_4600B0 + sub_699000 + dword_8F82A0 —— 大波动推进与池化槽。""",
"6170E0": """大波动生成辅助：字符串证据「AIR_SWORD」（音效 TAG —— 空中剑气）。sub_47E720/47E9D0 + sub_4922F0 ——
剑气浪的生成判定（上挑系剑气 PO 化【推断】）。""",
"617370": """CNDecoy 命中判定：字符串证据「PHASE_SHIFT」（音效 TAG —— 相位转移）。
sub_47E720/47EF20 + sub_4B3C90 —— 诱饵（Decoy）受击触发的相位闪避判定【推断：机械师诱饵被打时隐身】。""",
"6175B0": """CNChaser 碰撞响应槽（vmt2A8）：sub_47EF20/47EFD0 + dword_73C3CC ——
追击者/变换珠（Chaser，法师光/暗/冰/火珠）的环绕判定。""",
"6177F0": """珠体辅助：sub_42B420 + sub_699000 ——【推断：珠体环绕角速度的时间域积分】。""",
"6178D0": """CNChaser 初始化（vmt10）：珠体初值装配。""",
"617970": """珠体发射辅助（111 行）：字符串证据「CHASER_LAUNCH」（音效 TAG —— 珠体发射）。
sub_4049C0/4049F0（lerp 对）+ dword_73C3AC —— 珠体从环绕切向发射的参数插值。""",
"617B80": """珠体参数块读取：dword_9B93C4/9B93C8 ——【推断：珠体全局参数（与 9B93xx 活动装备块相邻）】。""",
"617C20": """珠体辅助：sub_4015C0 + sub_4922F0 ——【推断：珠体动画定位】。""",
"617D00": """珠体粒子装载（112 行）：字符串证据「Character/Mage/Effect/Particle/Chaser%s%d.ptl」——
按属性+等级格式化的粒子路径模板（光/暗/冰/火 × 编号）。
sub_428F30（发射器）+ sub_5CC5A0 + dword_73C3E0 —— 变换珠四属性粒子矩阵（dword_796E50 路径根）。""",
"617F50": """CNChaser 主循环槽（vmt14C，148 行）：sub_42F220（动画取值）+ sub_4E5F70（硬直率掷）+ sub_600070 +
sub_617970（发射）/617D00（粒子）+ sub_699000 —— 珠体环绕→锁定→发射的全状态机。
动画资源见 0x6183A0（Chaser/StartAttack.ani | Target.ani）。""",
"618260": """CNChaser 命中表现槽（vmt1F4）：sub_431230（字符串入表）+ sub_617D00（粒子）—— 珠体命中演出。""",
"6183A0": """★ CNChaser 受击表现槽（vmt1D8，222 行）：字符串证据「Character/Mage/Effect/Animation/Chaser/
StartAttack.ani | Target.ani」——珠体被攻击时播放的攻击/锁定动画。
sub_45EBE0（飘字）+ sub_5DC290 + sub_6000E0（基类飘字）+ sub_4A06D0（挂点跟随）——
【新发现】珠体是可以被打的（破坏敌方珠子的机制载体），受击走完整飘字链。""",
"618950": """CNMagicMissile 命中结算槽（vmt2BC）：sub_47E720/47EFD0 + sub_4049C0 —— 魔法飞弹（MAGIC_MISSILE TAG 见 0x618B60 区）结算。""",
"618A90": """CNMagicMissile 主循环槽（vmt14C）：sub_4600B0 —— 飞弹飞行演出；粒子见 0x618B60 区
「PassiveObject/Character/Mage/Particle/MagicMissileParticle.ptl」。""",
"618C20": """魔法飞弹辅助：sub_4E4A50（动作包）+ dword_78FE24/78FE28/78FE2C（攻击上下文邻块）——
【推断：飞弹伤害三参数（攻击/防御/修正）的装配】。""",
"618E40": """CNLaunchJOL 主循环槽（vmt14C，114 行）：字符串证据「Character/Mage/Particle/LaunchJOL.ptl」
（JOL = Jack-O-Lantern 南瓜灯【推断】）+ dword_8F84E4 —— sub_4A0650/sub_4B2580 ——
南瓜灯抛射体的飞行编排；音效 LANTURN_FIRE 见 0x6193B0 区。""",
"619110": """CNLaunchJOL 命中结算槽（vmt2BC，90 行）：sub_47E720/47EF20/47EFD0 + sub_4922F0 +
dword_78FE30/34 —— 南瓜灯落地爆炸的结算。""",
"619460": """冰霜魔偶头参数装载（137 行）：sub_423A20（UDP 广播）+ sub_4E4630 + sub_619410 + dword_796E50 ——
【推断：FrostHead 的弹跳/滚动参数】。""",
"6196E0": """CNLaunchFrostHead 命中结算槽（vmt2BC，91 行）：sub_47E720/47E9D0/47EF20/47EFD0 + sub_4922F0 +
sub_4049C0 —— 冰霜魔偶头的撞击结算。""",
"619980": """CNLaunchFrostHead 命中表现槽（vmt1F4）：字符串证据「Animation/LaunchFrostHeadHit.ani」。
sub_44B6C0 + sub_45FE80 —— 撞击碎裂演出。""",
"619AB0": """CNLaunchFrostHead 主循环槽（vmt14C）：字符串证据「Animation/LaunchFrostHeadJumpEffect.ani」+
「FROSTHEAD」（音效 TAG）。sub_4926A0（定时器）+ sub_619460 —— 魔偶头弹跳推进。""",
"619C80": """CNLaunchFrostHead 消亡联动槽（vmt12C）：字符串证据「Animation/LaunchFrostHeadDisappear.ani」——
魔偶头消失动画。""",
"619E70": """CNLaunchBlackCat 命中结算槽（vmt2BC）：sub_47E720/47EF20/47EFD0 + sub_4049C0 ——
黑猫（LaunchBlackCat，魔道学者黑猫【推断】）撞击结算。""",
"61A070": """黑猫辅助：sub_493D00（研究 20 P38-2 速率写者直证）+ dword_73C5A4 ——
黑猫运动速率的动态改写【推断：猫跳节奏控制】。""",
"61A1F0": """CNLaunchBlackCat 主循环槽（vmt14C，96 行）：sub_4A0650 + sub_619E00/61A070 + sub_699000 ——
黑猫往返跳跃的状态机。""",
"61A400": """黑猫辅助：字符串证据「LAUNCH_PLUTO」（音效 TAG —— 冥王星/普鲁托）+ sub_493D00（速率写者）——
黑猫发射播音与速率装配。""",
"61A570": """CNHalloweenBusterSub 命中结算槽（vmt2BC，74 行）：sub_47E720/47EF20/47EFD0 + sub_4049C0 ——
万圣破坏者子体的追踪判定。""",
"61A780": """CNHalloweenBuster 命中结算槽（vmt2BC）：sub_4922F0 + sub_4049C0 —— 万圣破坏者本体的撞击结算。""",
"61A960": """CNHalloweenBuster 主循环槽（vmt14C）：sub_424120（震屏）+ sub_4922F0 —— 万圣破坏者（HalloweenBuster，
万圣节南瓜炮【推断】）的爆炸震屏。""",
"61AAD0": """CNHalloweenBusterSub 主循环槽（vmt14C）：sub_4600B0 + sub_4922F0 —— 子体飞行演出。""",
"61ABD0": """万圣流星粒子：字符串证据「PassiveObject/Common/Particle/ShootingStarFire.ptl」+「SSTAR_DROP」（音效 TAG）+
sub_428F30 —— 万圣破坏者复用流星系粒子（资源复用直证）。""",
"61ACA0": """万圣流星爆粒子：字符串证据「ShootingStarExpSmoke.ptl | ShootingStarExp.ptl | ShootingStarExpEtc.ptl」+
「SSTAR_HIT」—— 与 0x60DFA0（ShootingStarExp）完全同款的粒子三件套复用。""",
"61AE90": """CNNapalmBomb 碰撞响应槽（vmt2A8）：sub_47E720/47E9D0/47EF20/47EFD0 + sub_607260 ——
凝固汽油弹（NapalmBomb）的落点判定。""",
"61AFC0": """CNNapalmBomb 初始化（vmt10）：sub_4049C0 lerp + sub_42B420 + sub_491F80/4938F0 + sub_699000 ——
凝固弹的燃烧起值插值。""",
"61B0D0": """凝固弹辅助（63 行）：sub_41C5E0 + sub_4E5F70（硬直率掷）+ sub_6072C0 ——【推断：燃烧 DOT 的结算】。""",
"61B1C0": """CNNapalmBomb 消亡联动槽（vmt12C）：sub_41C5E0 + sub_4BE520 + sub_4C20A0 ——
凝固弹熄灭/转燃烧区域的状态交接【推断】。""",
"61B260": """燃烧区域调度（110 行）：sub_4B3DF0/4BE520/4C20A0 + sub_4E4630 + sub_5FA510 ——
地面火海的持续燃烧与蔓延编排【推断】。""",
"61B440": """凝固弹粒子：字符串证据「Particle/NapalmBombFire.ptl | NapalmBombFireLight.ptl」（本体火+光晕双发射器）+
sub_4C3640 —— 弹体燃烧尾焰。""",
"61B590": """地面火海四件套（157 行）：字符串证据「Animation/NapalmBombAreaFire1.ani | AreaFire2.ani | %s%d.ani |
Animation/NapalmBombFireLight | Particle/NapalmBombAreaFireLight.ptl」——双动画层 + 编号动画模板 + 光粒子。
sub_445D00（156B 粒子分配，P36 直证）+ sub_499240（vmt8 方向/邻近计算）+ sub_4924C0 ——
落点周围按方向铺设火海（方向性燃烧圈）。""",
"61B980": """★ CNNapalmBomb 主循环槽（vmt14C，220 行）：字符串证据「SIZZLE_LOOP」（音效 TAG —— 咝咝燃烧循环）。
sub_424120（震屏）+ sub_47E720 族（判定）+ sub_4A0B60 + dword_9A0114（研究 20 P36 ⑤ 直证：全局粒子类型表）+
sub_61B260/61B590（火海调度/铺设）—— 弹道→落地→火海蔓延的完整编排，
是本文件对 P36 粒子系统消费最重的函数之一。""",
"61BE50": """CNTargetItBall 命中表现槽（vmt1F4）：字符串证据「Particle/TargetItBallPaint.ptl」（涂装粒子）+
sub_5DDBA0 + dword_9B9410 —— 标记球（TargetItBall）命中目标时的涂色标记【推断：标记类 PO】。""",
"61C020": """CNHunterSpittle 主循环槽（vmt14C）：猎人唾液（HunterSpittle，蜘蛛吐弹【推断】）的飞行推进。""",
"61C0F0": """CNHunterSpittle 命中表现槽（vmt1F4）：字符串证据「PassiveObject/Monster/Spider/Ani/PoisonWebLand.ani」
（资源路径直证：蜘蛛族毒网）+ sub_44B6C0/44E0D0 + sub_45FE80 —— 唾液落地的毒网铺设。""",
"61C240": """CNZombieEye 命中表现槽（vmt1F4）：sub_4E4A50 —— 僵尸眼球（ZombieEye）命中发包。""",
"61C3A0": """CNZombieEye 主循环槽（vmt14C）：字符串证据「ZOMBIE_EYEROLLING_LOOP」（音效 TAG —— 眼球滚动循环）——
悬浮眼球机关的循环维持。""",
"61C4D0": """CNKagelBomb 表现壳槽（vmt200，91 行）：sub_423A20（UDP 广播）+ dword_796E50 ——
卡格炸弹（KagelBomb）的状态表现与广播。""",
"61C7A0": """CNKagelBomb 主循环槽（vmt14C）：sub_4E4A50 —— 卡格炸弹引信推进；爆炸音 BOMB_EXPLOSION 见 0x61C860。""",
"61C860": """卡格弹爆炸辅助：字符串证据「BOMB_EXPLOSION」（音效 TAG）+ sub_4015C0 —— 播音 TAG 装配。""",
"61C8E0": """CNKagelBombSub 主循环槽（vmt14C，84 行）：sub_41ACF0 + sub_428F30（发射器）+ sub_4600B0 ——
卡格弹子体的分裂/爆炸演出（与 0x6107E0 CurseBombSub 同构）。""",
"61CA80": """CNSpaceFishBug 命中表现槽（vmt1F4）：sub_432FA0（UDP 广播）+ sub_496F80 ——
太空鱼虫（SpaceFishBug）命中广播。""",
"61CB60": """太空鱼虫辅助：字符串证据「SFISH_CAST」（音效 TAG —— 鱼虫抛出）+ sub_4049C0 lerp —— 抛射参数插值。""",
"61CCE0": """CNLOLLaser tick 槽（vmt2C4）：sub_699000 —— LOL 激光（LOL = LegOfLotus 莲花腿【推断】，
与 CNLegOfLotus 类族配套）的持续照射推进。""",
"61CD70": """CNBronzeGolemBullets 消亡联动槽（vmt12C）：sub_4E4A50 —— 青铜魔像弹幕（BronzeGolemBullets）消亡发包。""",
"61CE10": """CNBronzeGolemBullets 命中表现槽（vmt1F4）：sub_4A06D0（挂点跟随族）—— 魔像弹命中演出。""",
"61CF50": """CNIceWaveUp 主循环槽（vmt14C，141 行）：字符串证据「ICEWAVE_CREATE」（音效 TAG，冰浪系同款）。
sub_4DB200 + sub_4E77D0 + dword_8F82A4/A8/AC（冰浪池化槽）—— 上升冰浪（IceWaveUp）的推进；
与 0x611C70/0x612B70 构成冰浪三形态（追踪/多段/上升）。""",
"61D2B0": """CNPierceHeadLegOfLotus 主循环槽（vmt14C）：字符串证据「LOTUS_THRUST」（音效 TAG —— 莲花腿突进）——
莲花腿穿头（PierceHead）的突进推进。""",
"61D440": """莲花腿辅助：sub_4926A0/4939E0（定时器/动画）+ dword_9B941C ——【推断：抓取计时与目标缓存】。""",
"61D590": """CNGrabbingLegOfLotus 槽（vmt218）：sub_491C10 ——【推断：抓取动画绑定】。""",
"61D650": """CNGrabbingLegOfLotus 命中表现槽（vmt1F4）：字符串证据「TRAP_OCTOPUS_CATCH」（音效 TAG —— 章鱼陷阱捕获）+
sub_61D440 —— 抓取腿（GrabbingLegOfLotus）捕获目标。""",
"61D710": """莲花腿辅助（68 行）：sub_431230 ——【推断：抓取参数入表】。""",
"61D900": """抓取收紧辅助（107 行）：字符串证据「TRAP_OCTOPUS_TIGHTEN」（音效 TAG —— 收紧）。
sub_4926A0 + sub_4E5F70（硬直率掷）+ sub_61D380/61D540/61D8C0 ——
捕获后的绞紧状态机（持续伤害【推断】）。""",
"61DB50": """CNGrabbingLegOfLotus 主循环槽（vmt14C）：字符串证据「TRAP_OCTOPUS_SWING」（音效 TAG —— 甩动）+
sub_61D8C0/61D900 —— 捕获→绞紧→甩动的三段陷阱链（TRAP_OCTOPUS 三 TAG 齐全）。""",
"61DCC0": """CNRisingShuriken 主循环槽（vmt14C，80 行）：sub_423A20（UDP 广播）+ sub_45EBE0（飘字）——
升手里剑（RisingShuriken）的升空编排。""",
"61DE30": """CNFallingShuriken 表现壳槽（vmt200，93 行）：sub_423A20 + dword_796E50 ——
落手里剑（FallingShuriken）的状态广播【推断：升/落手里剑的空地联动】。""",
"61E040": """CNFallingShuriken 主循环槽（vmt14C）：字符串证据「JEWEL_HIT」（音效 TAG —— 宝石命中）+ sub_4939E0 +
sub_698F90/699000（时钟复位/读数）—— 落手里剑的落点编排。""",
"61E1C0": """CNGhoulSkullBomb 槽（vmt144）：sub_491690 + sub_4049C0 lerp + sub_699000 ——
食尸鬼头骨炸弹（GhoulSkullBomb）的抛物线参数插值。""",
"61E2B0": """CNGhoulSkull 状态槽（vmt1E4，35 行，无外部调用）——【推断：头骨滞空/爆散状态折算】。""",
"61E340": """CNGhoulSkullSub 主循环槽（vmt14C，84 行）：sub_41ACF0 + sub_428F30（发射器）+ sub_4600B0 ——
头骨子体的飞散演出（与 0x61C8E0/0x6107E0 同构的子体三兄弟）。""",
"61E5E0": """头骨子体演出：字符串证据「PassiveObject/Monster/Ghoul/Animation/SkullSub.ani」+「SKULL_BREAK」
（音效 TAG —— 碎裂）。sub_44B6C0/44E0D0 + sub_45FE80 —— 头骨碎裂演出（资源路径直证 Ghoul 族）。""",
"61E720": """头骨爆散调度（71 行）：sub_423A20（广播）+ dword_8F82A0/8F8520 + sub_61E5E0 ——
爆散子体的分布生成【推断：环形爆散】。""",
"61E970": """CNAraclothoMissile 命中表现槽（vmt1F4）：阿拉克洛（Araclotho，蛛网夫人系【推断】）飞弹命中。""",
"61EA00": """阿拉克洛辅助：sub_494F20 + sub_4049C0 lerp + sub_699000 ——【推断：丝线弹道的时间域插值】。""",
"61EB60": """CNAraclothoMissile 主循环槽（vmt14C）：sub_61EA00 + sub_698F90/699000 —— 飞弹推进；溅射见 0x61EC80。""",
"61EC80": """CNAraclothoMissile 溅射槽（vmt2B4）：字符串证据「Common/HitEffect/Particle/SmallHitParticle.ptl」——
惰性创建小型命中粒子发射器（与 0x60D5A0 IceNiddleGroup 同构的 0x12C 惰性发射器模式）。""",
"61ED80": """CNCobWeb 命中表现槽（vmt1F4，91 行）：字符串证据「PassiveObject/Monster/Spider/Ani/
BossSlowWebLand.ani | BossSlowWebEffect.ani」—— Boss 减速蛛网（CobWeb）的双动画。
sub_44B6C0/44E0D0/44E1F0（帧事件）+ sub_45FE80 —— 蛛网着陆+持续减速场演出（资源路径直证蜘蛛族）。""",
"61F060": """CNSpectressBat 消亡联动槽（vmt12C）：sub_4600B0 —— 幽魂蝙蝠（SpectressBat）消散演出。""",
"61F150": """CNSpectressBat 主循环槽（vmt14C）：sub_4600B0 + sub_698F90/699000 —— 蝙蝠群飞行推进。""",
"61F240": """蝙蝠粒子：字符串证据「PassiveObject/Monster/Wraith/Particle/FlyBat.ptl | FlyBatParticle.ptl」+
「BAT_FLAP」（音效 TAG —— 振翅）。sub_428F30 + sub_698F90 —— 蝙蝠群粒子（资源路径直证 Wraith 族）。""",
"61F360": """CNEvilEyeLOLLaser tick 槽（vmt2C4）：sub_4926A0/699000 —— 邪眼（EvilEye）激光的持续照射推进
（STONE_LASER_FIRE TAG 见 0x61F410）。""",
"61F410": """CNEvilEyeSmallLaser 主循环槽（vmt14C）：字符串证据「STONE_LASER_FIRE」（音效 TAG —— 石像激光）——
邪眼小激光的照射推进。""",
"61F4C0": """邪眼辅助（51 行）：sub_4926A0 + sub_699000 ——【推断：激光扫描计时】。""",
"61F5C0": """CNEvilEyeQuake 主循环槽（vmt14C）：sub_4600B0 + sub_699000 —— 邪眼震地（EvilEyeQuake，
Vib_dust/Vib_rock 粒子见 0x61F830 区）的推进。""",
"61FCB0": """CNMineTrap 生成槽（vmtC）：sub_4049C0 lerp + dword_8F8520 —— 地雷陷阱（MineTrap）的布雷初值。""",
"61FD30": """CNMineTrap 主循环槽（vmt14C，76 行）：sub_699000 —— 地雷待机/触发推进【推断：踩踏检测计时】。""",
"61FEA0": """CNMineTrap 受击表现槽（vmt1D8）：sub_4926A0/699000 —— 地雷被触发时的引爆链启动【推断】。""",
"61FF90": """CNMineTrap 表现壳槽（vmt200）：sub_4049C0 + sub_41ACF0 —— 地雷的伪装/显形表现【推断】。""",
}
