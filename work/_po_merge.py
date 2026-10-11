# -*- coding: utf-8 -*-
# L1 精修插入脚本：文件头 + 分区横幅 + 函数头注释（仅追加 // 行，函数体零改动）
import re, importlib.util, shutil

SRC = r"src\combat\passive_object.cpp"
BAK = r"work\passive_object.cpp.L2.bak"

def load(p):
    spec = importlib.util.spec_from_file_location("m%d" % len(C), p)
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m.C
C = {}
for f in ["work/_poc_a.py","work/_poc_b.py","work/_poc_c.py","work/_poc_d.py"]:
    d = load(f)
    for k, v in d.items():
        if v is None: continue
        assert k not in C, "dup " + k
        C[k] = v

HEADER = """// ============================================================
// 【40JP逆向源代码工程】combat/passive_object.cpp
// 模块: PassiveObject 生成/召唤/精炼 + 第三类对象大家族（PO/附加器/对话窗/弹幕）
// 来源: work/dump_skills.txt  区块: 0x600000-0x620000
// 层级: L2 转写层 + L1 锚点精修（函数体保持 Hex-Rays 忠实输出，仅升级注释）
//        语义与 IDA Hex-Rays 逐字对应; 逐行精修层(L1)见 src/nengine|cnrd 与 docs/10-15
// 函数数: 621
// ------------------------------------------------------------
// ★ 本文件已按《21_打击感公式算法速查》P32/P36 与研究 20（§1.1/§5.1/§10、P24/29/33/35/36/37/38/41）锚点升级注释：
//   0x6000E0/0x6183A0 PO 受击表现槽 vmt1D8 → 调 sub_4D7960 飘字咽喉（§1.1 case 0x14）
//   0x6001C0 PO 攻击判定槽 vmt214 → 消费 [ATTACK BOX] float×6 帧表（P37-2）+ vt+600/608 无敌门（P35）→ sub_4E5F70 硬直率掷（§5.1）
//   0x600C20/0x6011B0 PO 生成工厂（「PASSIVEOBJECT CREATE FAIL!」双路径）+ sub_4A3450/sub_4B2970 attackinfo（P29）
//   0x6014F0 精炼炉 cube 脚本解析器（[cube index]/[legacy info]/[skill info]/[skill prob]/[cube creation const]）
//   0x610C00 召唤阵三段动画 + 四属性 Spirit 召唤（0x60 区字符串证据本体）
//   0x6004B0 PO 数量自适应透明度【新发现】；0x60E900 死亡衍生 PO 基类【新发现】；0x60C8C0 粒子工厂 PO【新发现】
//   全文件：出生/循环/命中音效 TAG 走 vt+212 播音链（P32）；.ptl 发射器经 sub_428F30（P36）；
//           演出经 sub_4600B0→sub_45FE80 演播层（P33）；震屏经 sub_424120（P41）；
//           状态槽写入经目标 vt+760（§10 17 槽异常状态）；防篡改读写 sub_41A590/41A5E0（P24）
// ------------------------------------------------------------
// 分区总览（11 区，横幅见正文）：
//   1 PO 基类核心与生成工厂  2 掉落搬运与 MP 恢复  3 附加器(IRDAppendage)道具/装备族
//   4 临时判定与霸体  5 NPC 对话窗  6 神枪手子弹系  7 连接体/草丛/地图物件
//   8 元素与自然现象 PO  9 召唤/投技/鬼神/气功 PO  10 法师投射物与陷阱  11 怪物 PO 弹幕/陷阱大全
// ------------------------------------------------------------
// 字符串证据总类（详见各函数头注释）：PO 生成失败双串、cube 脚本五标签、四属性 Spirit 召唤动画、
//   SummonCircle 三段法阵、SuperArmor 双动画、ForceField 四段资源、Ghost 三鬼神 TAG、
//   TRAP_OCTOPUS 三段陷阱 TAG、冰浪 ICEWAVE_CREATE 三形态、NapalmBomb 燃烧四件套、
//   Gunner 弹种×枪种动画矩阵、MarchingFortress 无 —— 各资源路径均标注归属（PassiveObject/Common|Monster|Character 系）
// ============================================================
"""

BANNERS = [
("6000B0", """// ============================================================
// 【分区 1】PO 基类核心与生成工厂（0x6000B0-0x601A50）
// CNRDPassiveObject 虚槽族（生成/受击飘字/攻击判定/主循环/表现壳）+ 工厂双路径
// （按文件名 0x600C20 / 按索引 0x6011B0，失败串 PASSIVEOBJECT CREATE FAIL!）
// + 精炼炉 cube 脚本解析器 0x6014F0（[cube index]/[skill info] 等五标签唯一定义点）
// ============================================================"""),
("601A50", """// ============================================================
// 【分区 2】掉落/搬运 PO 与 MP 恢复（0x601A50-0x602380）
// CNStorageBird（搬运鸟）/ CNFallingStorage（落宝箱）/ CNStorageSub / CNChangeMp（MP 恢复精灵）
// 协议证据：动作包 51201（0x601E40）—— PO 专有包号
// ============================================================"""),
("602380", """// ============================================================
// 【分区 3】附加器（IRDAppendage）道具/装备族（0x602380-0x6056B0）
// 注意：本区并非 PO 本体 —— 0x602810 析构链直证 vftbl_IRDAppendage 基类。
// CNRemoveActiveStatus / CNActiveStatusStackable / CNOmozzaOfWorld / CNLotteryStackable /
// CNAntiFreezePotion / CNSpecificEnemyEquipment / CNEventEquipment 等 Stackable 族：
// 激活入口统一为 vmt88 → 记录逐条写入目标 vt+760（研究 20 §10 17 状态槽写入器）。
// 音效/UI 证据：RECIPE_NOTICE、USE_DEBUFF、ALERT_1、RemoveActiveStatus.ptl
// ============================================================"""),
("6056B0", """// ============================================================
// 【分区 4】临时判定与霸体 PO（0x6056B0-0x605FD0）
// CNTemporaryHit（一次性 hitbox）/ CNSuperArmor（霸体光环，SuperArmorEffect/SuperArmorBody 双动画）
// ============================================================"""),
("605FD0", """// ============================================================
// 【分区 5】NPC 对话窗 CNRDDialogueWindow（0x605FD0-0x607420）
// UI 类混入本区：控件树重建/对话树遍历/按钮处理。
// 字符串证据：调试断言「if (g_userManager->getMyUser() == NULL)」、音效 TAG CLICK_BUTTON2
// ============================================================"""),
("607420", """// ============================================================
// 【分区 6】神枪手子弹系（0x607420-0x609720）
// CNGunnerBullet（主武器弹）+ CNGunnerSubWeaponBullet（Musket/BowGun 副武器弹）。
// 资源矩阵直证：属性（Flame/Freeze/Silver）× 枪种（Pistol/Musket/BowGun）的
//   弹体动画 0x6081B0 与命中特效 0x6083 查表；银弹拖尾 SilverBulletParticle.ptl；
//   散弹枪 EX 音效三 TAG（DIFFUSION_GUN_EX/CHARGE、R_DIFFUSION_GUNA）
// ============================================================"""),
("609720", """// ============================================================
// 【分区 7】连接体/可割草丛/地图物件（0x609720-0x60B920）
// CNRDJoint（连接体）/ CNCutableGrass（可割草丛）/ CNMapObject（可破坏地图物件，
//   定义表装载与 monster_parser 同族）—— 地图可破坏生态的 PO 侧
// ============================================================"""),
("60B920", """// ============================================================
// 【分区 8】元素与自然现象 PO（0x60B920-0x610320）
// 火弹/火爆/冰弹/落雷/粒子工厂/爆风/地震/毒瓦斯/冰针群/地火/流星/回旋镖/弹跳体/
// 死亡衍生基类/陷阱石/火轮/黏土泡泡/预警圈/念气罩/附属圈 —— 地图机关与元素演出层
// 粒子证据：FireMissile/IceMissile/ShootingStar/NapalmBomb 各 .ptl 族（P36）
// ============================================================"""),
("610320", """// ============================================================
// 【分区 9】召唤/诅咒/投技/鬼神/气功 PO（0x610320-0x616390）
// ★ 0x610C00 = SummonCircle 三段法阵 + 四属性 Spirit（火/水/暗/光）召唤动画本体；
// CNBodyHit/CNSuplexSub 投技判定；冰浪三形态（追踪/多段/上升）；CNGhost 三鬼神
// （GHOST_KAJAN/SAYA/BREMEN 音效 TAG）；CNSpiralNen/CNNenGuard 念气系
// ============================================================"""),
("616390", """// ============================================================
// 【分区 10】法师投射物与陷阱（0x616390-0x61AE90）
// 墓碑雨/空投炸弹/大波动/诱饵/变换珠 CNChaser（含可被打的珠体 0x6183A0）/
// 魔法飞弹/南瓜灯/冰霜魔偶头/黑猫/万圣破坏者 —— 魔道与召唤技能的 PO 投射物
// ============================================================"""),
("61AE90", """// ============================================================
// 【分区 11】怪物 PO 弹幕/陷阱大全（0x61AE90-0x620000）
// 凝固弹/标记球/蜘蛛毒网/僵尸眼/卡格弹/莲花腿章鱼陷阱（TRAP_OCTOPUS 三段）/
// 手里剑/食尸鬼头骨/蛛网/蝙蝠/邪眼/地雷 —— 怪物技能 PO 的名录制
// ============================================================"""),
]

shutil.copyfile(SRC, BAK)
lines = open(SRC, encoding="utf-8").read().split("\n")

# 1) replace header: lines 0..8 (old header is 8 lines + blank)
assert lines[0].startswith("// ====") and lines[7].startswith("// ===="), lines[:10]
assert lines[8] == ""
new = HEADER.rstrip("\n").split("\n")
new.append("")  # exactly one blank line before #include
lines = new + lines[9:]

# 2) walk markers, insert banners (before marker's dash line) and comments (after dash line)
out = []
i = 0
n = len(lines)
inserted_banner = set()
inserted_comment = 0
seen = 0
while i < n:
    l = lines[i]
    m = re.match(r"^// 0x([0-9A-F]{6})\b", l)
    if m:
        va = m.group(1)
        seen += 1
        assert lines[i+1].startswith("// ---"), lines[i+1]
        for bva, btext in BANNERS:
            if bva == va:
                # 若标记自带前导横线（out 末尾），把它移到横幅之后
                moved_dash = None
                if out and out[-1] == "// " + "-" * 60:
                    moved_dash = out.pop()
                out.extend(btext.split("\n"))
                if moved_dash:
                    out.append(moved_dash)
                inserted_banner.add(bva)
        out.append(l)
        out.append(lines[i+1])
        if va in C:
            for cl in C[va].split("\n"):
                out.append("// " + cl)
            inserted_comment += 1
        i += 2
        continue
    out.append(l)
    i += 1

assert seen == 621, seen
miss = [b for b, _ in BANNERS if b not in inserted_banner]
assert not miss, miss
open(SRC, "w", encoding="utf-8", newline="\r\n").write("\n".join(out))
print("markers:", seen, "comments inserted:", inserted_comment, "banners:", len(inserted_banner))
print("lines: %d -> %d" % (n, len(out)))
