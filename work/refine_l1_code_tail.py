# -*- coding: utf-8 -*-
"""code_tail.cpp L1 精修：0x6A-0x6C 代码尾/数据混区（CRT 桩群）注释升级。
纯插入式（函数体零改动），运行后自动做无损自验。"""
import sys
sys.path.insert(0, r"E:\DOF LX\【40JP逆向源代码工程】\work")
from refine_l1_tool import Refiner, parse_sections, parse_annots

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\platform\code_tail.cpp"

HEADER = """\
// ============================================================
// 【40JP逆向源代码工程】src/platform/code_tail.cpp
// 模块: 0x6A-0x6C 代码尾/数据混区 —— CRT x87 数学助手 + 全局对象
//       动态初始化/析构桩群
// 来源: work/dump_tail.txt  区块: 0x6A0000-0x6C1000
//
// 【L1 精修层】本文件已完成 L1 精修（注释级），内容:
//   1) 分区组织（2 个 SECTION：x87 数学/异常群、动态初始化/析构桩群）
//   2) 全部 61 函数逐个一行语义注释（【推断】= 无自证串）
// 函数体逻辑与 L2 转写层逐字一致，未做任何改动（剥离插入注释行可逐字节还原）。
//
// 定位结论（本次精修推理）:
//   - 0x6A02C6-0x6A0A9B: MSVC x87 双精度比较/异常内部库（unknown_libname
//     18/19/28/29/30/31/32/33 = _CMP/FUNCI 比较包装与 FPREM 规约核心，
//     __87except = x87 异常 → _handle_exc/_raise_exc/errno 分派）。
//   - 0x6BE880-0x6C0200: MSVC /GX 全局对象动态初始化器与析构桩
//     （_dynamic_initializer / _dynamic_atexit_destructor 形态）：
//     JUMPOUT(0x657880/0x6578AD) 双出口 = 条件析构桩；sub_401000 =
//     nengine::IObject 首槽（见 src/nengine/log_dump.cpp 0x401000）。
//   - 两个 RTTI 直证: 0x6BF4D0 Camera::vftable、0x6BF4E0 CNRDAStar::vftable
//     （docs/03 类总表的镜头系统与寻路类）。
//   - 0x6C00BE/EE/11E/14E 的 InterlockedDecrement(&Addend) 守卫与
//     272-275 的 ostream/istream vtable 装配 = MSVC iostream 静态 Init/灭。
// ============================================================"""

SECTIONS = """\
@0x6A02C6
// ============================================================
// SECTION 1  CRT x87 双精度比较/异常助手（0x6A02C6-0x6A0A9B）
// ------------------------------------------------------------
// st0/st1 寄存器传参（usercall）+ FPREM 规约 + fnstenv 状态字读取，
// 属 MSVC "net runtime" 比较库（_CMP 内部）；__87except 为 x87 异常
// 到 errno 的桥（_handle_exc/_raise_exc/_ctrlfp/_set_errno）。
// ============================================================
@0x6BE880
// ============================================================
// SECTION 2  全局对象动态初始化/析构桩（0x6BE880-0x6C0200）
// ------------------------------------------------------------
// MSVC /GX 对 C++ 全局对象的 _dynamic_initializer / atexit 析构桩群。
// 识别特征: 单一构造转发、sub_4166A0/sub_42CC00 等容器清空 + j__free、
// JUMPOUT(0x657880/0x6578AD) 双出口条件析构、SSO string 析构
// （容量 >= 0x10 才 free）。RTTI 直证: Camera / CNRDAStar vtable 装配。
// ============================================================"""

ANNOTS = """\
0x6A02C6 | 【L1】CRT x87 比较包装 A：st1/st0 → unknown_libname_16（FUCOMI 路径）
0x6A02D9 | 【L1】CRT x87 比较包装 B：操作数交换序（a2/a3 对调）【推断】
0x6A04EC | 【L1】CRT x87 比较包装 C（__spoils edx,ecx/st0 变体）【推断】
0x6A0501 | 【L1】CRT x87 比较包装 D（同族交换序变体）【推断】
0x6A0516 | 【L1】CRT x87 精细比较核心：FPREM 规约循环 + 64 位手工相减比较，返回 FPU 状态字 & 0x4300
0x6A071C | 【L1】CRT 双精度比较入口：subnormal 缩放 2^63（dbl_79BF74）后转 0x6A0516
0x6A07CE | 【L1】CRT x87 精细比较核心变体（与 6A0516 同体双入口）【推断】
0x6A09D4 | 【L1】CRT 双精度比较入口变体（与 6A071C 成对）【推断】
0x6A0A9B | 【L1】__87except：x87 异常码分派(8/4/17/18/16) → _handle_exc/_raise_exc → _ctrlfp + _set_errno
0x6BE880 | 【L1】★ clong 对象数组动态初始化：8 组 {4,4000,12,12000,32,40000,64,100000}（clong vftable），
          配 sub_422E70(1160-1163) 字符串 id【推断：经验/数值配置表】，atexit → 0x6BFD70
0x6BEE80 | 【L1】动态初始化转发：sub_423A20(&unk_9B8DD8)
0x6BEFD0 | 【L1】8 组 0x800 字节环形缓冲初始化（word_9B9D50 起步长 1032，容量 768）【推断：消息环】
0x6BF19A | 【L1】动态初始化转发：sub_669C81(&unk_9D24ED)
0x6BF2D0 | 【L1】全局缓冲析构：sub_4166A0 + j__free(dword_8F7A68)，三字段清零
0x6BF390 | 【L1】全局缓冲析构同族（dword_8F7C58）【推断】
0x6BF450 | 【L1】字符串数组析构：unk_8F8140 链式 free + sub_6599B0 计数递减
0x6BF4D0 | 【L1】★ Camera 全局 vtable 装配（off_77FD68 = &Camera::vftable，docs/12 镜头系统直证）
0x6BF4E0 | 【L1】★ CNRDAStar 全局 vtable 装配（off_77FD90，寻路类直证）
0x6BF530 | 【L1】动态初始化转发：sub_437850(&dword_8F8558)
0x6BF540 | 【L1】动态初始化转发：sub_437950(&unk_8F85B4)
0x6BF560 | 【L1】动态初始化转发：sub_65D270(&unk_9789A0)
0x6BF570 | 【L1】动态初始化转发：sub_444020(&unk_97B248)
0x6BF590 | 【L1】IObject 槽初始化转发：sub_401000(&unk_97B6B4)（首函数 sub_401000 见 log_dump.cpp）
0x6BF5A0 | 【L1】IObject 槽初始化转发：sub_401000(&unk_97B698)
0x6BF5B0 | 【L1】条件析构桩：unk_97B6E4 非空走 JUMPOUT(0x657880) 否则 0x6578AD（Hex-Rays 尾跳转形态）
0x6BF5C0 | 【L1】条件析构桩同族（unk_98B708）【推断】
0x6BF670 | 【L1】GDI 资源析构：DeleteObject(ho) + 位图/调色板字段组（byte_98B8C8..dword_98B8F8）清零
0x6BF740 | 【L1】析构转发：nullsub_3(&unk_98BB68)
0x6BF760 | 【L1】IObject 槽初始化转发：sub_401000(&unk_99FBE0)
0x6BF880 | 【L1】IObject 槽初始化转发：sub_401000(&unk_99FC80)
0x6BF910 | 【L1】IObject 槽初始化转发：sub_401000(&unk_99FD70)
0x6BF9D0 | 【L1】条件析构桩同族（unk_9A023C）【推断】
0x6BFA80 | 【L1】动态初始化转发：sub_437950(&unk_9A0330)
0x6BFA90 | 【L1】动态初始化转发：sub_437950(&unk_9A035C)
0x6BFAE0 | 【L1】全局缓冲析构：sub_42CC00 + j__free(dword_9A05AC)
0x6BFB40 | 【L1】全局缓冲析构：sub_4B4950 + j__free(dword_9A05BC)
0x6BFBA0 | 【L1】std::string 全局析构：容量 >= 0x10 才 free（SSO 直证），复位 15/0/空
0x6BFD00 | 【L1】全局缓冲析构：sub_42CC00 + j__free(dword_9B7C64)
0x6BFD60 | 【L1】析构转发：sub_432D10(&unk_9B7D10)
0x6BFD70 | 【L1】0x6BE880 数组的 atexit 回调：eh vector destructor iterator(元素 0x20, 4 个, sub_50D520)
0x6BFDB0 | 【L1】条件析构桩同族（unk_9B7FD4）【推断】
0x6BFDC0 | 【L1】条件析构桩同族（unk_9B7FF8）【推断】
0x6BFDD0 | 【L1】条件析构桩同族（unk_9B801C）【推断】
0x6BFF00 | 【L1】std::string 全局析构（SSO 同 6BFBA0，dword_9B9388 组）【推断】
0x6BFF30 | 【L1】指针三元组析构（dword_9B93AC/B0/B4 双 free）【推断】
0x6C0090 | 【L1】析构转发：sub_5E5260(&dword_9B9474)
0x6C00BE | 【L1】iostream 静态析构守卫：InterlockedDecrement(&Addend) < 0 → 遍历
          critical section 数组（stru_9D21E8..9D2248, sub_66A30D）
0x6C00C8 | 【L1】std::ostream 动态初始化：vftable 装配（dword_9D2320）+ sub_65CCD0（MSVC ios Init 族）
0x6C00E4 | 【L1】析构转发：sub_6697B6(&unk_9D22C0)
0x6C00EE | 【L1】iostream 静态析构守卫同族（与 6C00BE 同体）【推断】
0x6C00F8 | 【L1】std::ostream 动态初始化同族（dword_9D23C0）【推断】
0x6C0114 | 【L1】析构转发：sub_6697B6(&unk_9D2360)
0x6C011E | 【L1】iostream 静态析构守卫同族【推断】
0x6C0128 | 【L1】std::ostream 动态初始化同族（dword_9D2458）【推断】
0x6C0144 | 【L1】析构转发：sub_6697B6(&unk_9D23F8)
0x6C014E | 【L1】iostream 静态析构守卫同族【推断】
0x6C0158 | 【L1】std::istream 动态初始化：&std::istream::vftable 装配（dword_9D24F0）+ sub_65CCD0
0x6C0174 | 【L1】析构转发：sub_6697B6(&unk_9D2490)
0x6C017E | 【L1】析构转发：sub_66A262(&dword_9D252C)
0x6C0192 | 【L1】_Init_atexit 析构（CRT atexit 注册器收尾）
0x6C0200 | 【L1】CRT atexit 表槽位估算：((a1-3)/0xC+1) 组 → (12n - a1)/2 + 5【推断：_Init_atexit 容量】"""

ref = Refiner(PATH)
ref.set_header(HEADER.split("\n"))
ref.build(parse_sections(SECTIONS), parse_annots(ANNOTS))
assert ref.verify_lossless(), "lossless verification failed"
ref.save(PATH)
ob, cb = Refiner.braces(PATH)
print("OK code_tail.cpp: sections=2 annots=%d va=%d braces=%d/%d" % (
    len(parse_annots(ANNOTS)), Refiner.count_vas(PATH), ob, cb))
assert Refiner.count_vas(PATH) == 61
