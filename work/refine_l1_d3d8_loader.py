# -*- coding: utf-8 -*-
"""d3d8_loader.cpp L1 精修：D3DX8 风格数学库（SSE + 3DNow! 双路径）注释升级。
纯插入式（函数体零改动），运行后自动做无损自验。"""
import sys
sys.path.insert(0, r"E:\DOF LX\【40JP逆向源代码工程】\work")
from refine_l1_tool import Refiner, parse_sections, parse_annots


def parse_sections(text):
    sec, cur = {}, None
    for ln in text.split("\n"):
        if ln.startswith("@0x"):
            cur = ln[1:].strip()
            sec[cur] = []
        elif cur and ln.strip():
            sec[cur].append(ln)
    return sec


def parse_annots(text):
    ann, cur = {}, None
    for ln in text.split("\n"):
        if not ln.strip():
            continue
        if ln.startswith("0x"):
            va, _, first = ln.partition("|")
            cur = va.strip()
            ann[cur] = ["// " + first.strip()]
        else:
            ann[cur].append("// " + ln.strip())
    return ann

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\platform\d3d8_loader.cpp"

HEADER = """\
// ============================================================
// 【40JP逆向源代码工程】src/platform/d3d8_loader.cpp
// 模块: 0x68A 区 —— D3DX8 风格数学库（SSE 路径 + 3DNow! 路径）
// 来源: work/dump_tail.txt  区块: 0x68A000-0x690000
//
// 【L1 精修层】本文件已完成 L1 精修（注释级），内容:
//   1) 分区组织（3 个 SECTION：SSE 变换群 / SSE 矩阵·平面群 / 3DNow! 矩阵群）
//   2) 全部 73 函数逐个一行以上语义注释（SIMD 指纹佐证，【推断】= 无自证串）
// 函数体逻辑与 L2 转写层逐字一致，未做任何改动（剥离插入注释行可逐字节还原）。
//
// 重要定位结论（本次精修推理）:
//   - 本文件不是 d3d8.dll 的加载器，而是 **D3DX8 静态数学库本体**：
//     0x68A4B0-0x68C6BE 为 SSE(__mm128) 实现，0x68C876-0x68FF1C 为
//     3DNow!(_m_p* 指令族) 实现 —— 两套同构函数群对应同一 D3DX API 的
//     CPU 厂商双分派（D3DX8 原版即带 _X86/_SSE 双实现）。
//   - "d3d8.dll/d3d8d.dll 加载 + DebugSetMute"、"Software\\Microsoft\\Direct3D"
//     注册表读取、"AuthenticAMD/UnknownVendr" CPU 检测的实际函数体在
//     missing_tail.cpp 0x68 区（0x680B13 RegOpenKeyA / ~0x680B00 区
//     LoadLibraryA("d3d8.dll")→GetProcAddress("DebugSetMute") /
//     0x680134、0x680380 CPU 品牌串），计划内不精修，只在此建立交叉引用。
//   - Direct3DCreate8 导入 thunk 在 missing_tail.cpp 0x67BA36；
//     消费方: src/nengine/log_dump.cpp:8259 Direct3DCreate8(220)。
//   - 与 socket_ime.cpp SECTION 1（0x691070 3DNow! 群）同一数学库家族。
// ============================================================"""

SECTIONS = """\
@0x68A4B0
// ============================================================
// SECTION 1  SSE 向量/矩阵变换（D3DX 风格 · Intel/SSE 路径）
//            （0x68A4B0-0x68AFA0）
// ------------------------------------------------------------
// __m128/_mm_* 指纹；成对 float 常数 0x3F800000=1.0f。
// D3DX8 数学库的 SSE 实现：向量变换（含 1/w 透视除法 _mm_rcp_ps +
// Newton 迭代）、4 顶点批量变换、4x4 矩阵乘、sqrt 归一化（x87 标量）。
// 与 SECTION 3 的 3DNow! 群互为同构实现。
// ============================================================
@0x68B06E
// ============================================================
// SECTION 2  SSE 矩阵运算/平面/透视（0x68B06E-0x68C6BE）
// ------------------------------------------------------------
// 平面归一化(sqrt)、透视投影(rcp)/反投影、4x4 矩阵求逆
// （0x68BC5E 伴随式 169 行巨型）、批量转置乘（0x68BF1E）。
// ============================================================
@0x68C876
// ============================================================
// SECTION 3  3DNow!/MMX 四元数与矩阵（AMD 路径 · 0x68C876-0x68FF1C）
// ------------------------------------------------------------
// _m_pf*（3DNow!）/_m_femms 指纹 + x87 标量混合；pfacc = 半和点积
// （16 次 = 4x4 矩阵乘）、pswapd/pnacc = 转置点积。
// 四元数→旋转矩阵（0x68E4DD，x²+y²+z² 与 1.0f 组合直证）、
// 仿射组合巨型（0x68E592 967 行 / 0x68FF1C 946 行，四元数×缩放×平移）。
// CPU 分派上游: missing_tail.cpp 0x680134/0x680380 (AuthenticAMD/UnknownVendr)。
// ============================================================"""

ANNOTS = """\
0x68A4B0 | 【L1】SSE vec4×矩阵：4x4 乘（shuffle 展开）+ 平移行（D3DXVec4Transform 形态）
0x68A4F0 | 【L1】SSE 变换 + 1/w 透视除法：_mm_rcp_ps + Newton 精化（D3DXVec3Project 前段形态）【推断】
0x68A550 | 【L1】SSE 变换（输出低 2 分量 = vec2 投影结果）【推断】
0x68A59E | 【L1】SSE 4x4 转置乘双路径：a3&0xF 判对齐/未对齐（行/列主序兼容），loadh/storeh 各 4
0x68A65E | 【L1】SSE 4 顶点批量矩阵变换：loadh/storeh/storel 各 4 全展开（矩阵每列点积直证）
0x68A7AE | 【L1】x87 标量 vec 归一化（sqrt 长度 + 除法）【推断】
0x68A87E | 【L1】x87 标量 vec 归一化变体（与 68A7AE 成对）【推断】
0x68A95E | 【L1】SSE 矩阵×矩阵（4x4 乘，loadh×3 重组）【推断】
0x68AA00 | 【L1】x87 标量归一化变体【推断】
0x68AAA0 | 【L1】SSE userpurge 变换（xmm 寄存器传参，编译器内联形态）【推断】
0x68AB60 | 【L1】SSE userpurge 变换变体【推断】
0x68ABF0 | 【L1】SSE userpurge 变换变体【推断】
0x68ACA0 | 【L1】SSE userpurge 变换变体【推断】
0x68AD60 | 【L1】SSE userpurge 变换变体【推断】
0x68AE00 | 【L1】SSE userpurge 变换变体【推断】
0x68AE90 | 【L1】SSE 矩阵装配/设置（shuffle×2，双精度入参）【推断】
0x68AEF0 | 【L1】SSE 4x4 矩阵乘（紧凑 13 行，D3DXMatrixMultiply 形态）【推断】
0x68AF40 | 【L1】双精度包装：同 68AEF0 的 double 接口形态【推断】
0x68AFA0 | 【L1】SSE userpurge 大变换（xmm5/xmm6 传参）【推断】
0x68B06E | 【L1】SSE 矩阵×向量展开（4 列点积，D3DXVec3Transform 形态）【推断】
0x68B13E | 【L1】SSE 平面归一化（sqrt 长度 + 全分量除，D3DXPlaneNormalize 形态）【推断】
0x68B22E | 【L1】SSE 平面变换 + 归一化（98 行，D3DXPlaneTransform 形态）【推断】
0x68B3BE | 【L1】SSE sqrt 归一化变体【推断】
0x68B490 | 【L1】SSE 变换小函数【推断】
0x68B520 | 【L1】x87 标量 sqrt 归一化变体【推断】
0x68B5CE | 【L1】SSE 小型矩阵/向量操作【推断】
0x68B630 | 【L1】x87 标量双 sqrt（距离/长度计算）【推断】
0x68B75E | 【L1】SSE 变换（shuffle×9 重组）【推断】
0x68B82E | 【L1】SSE 矩阵×矩阵变体【推断】
0x68B8EE | 【L1】SSE 矩阵×矩阵变体【推断】
0x68B9AE | 【L1】SSE 透视除法（rcp×2 = 1/w 与 1/w 精化，D3DXVec3Unproject 形态）【推断】
0x68BAAE | 【L1】x87 标量归一化变体【推断】
0x68BB7E | 【L1】x87 标量归一化变体【推断】
0x68BC5E | 【L1】★ SSE 4x4 矩阵求逆（169 行伴随式 + 行列式，D3DXMatrixInverse 形态）【推断】
0x68BF1E | 【L1】SSE 批量转置乘（71 行，loadh/storeh/storel 各 4，多顶点管线）【推断】
0x68C07E | 【L1】SSE 矩阵×向量（与 68B06E 同构）【推断】
0x68C12E | 【L1】SSE 稀疏标量混合（storel×3 = 分量写回）【推断】
0x68C1AE | 【L1】SSE 稀疏标量混合变体【推断】
0x68C23E | 【L1】SSE 平面归一化变体（sqrt，与 68B13E 同构）【推断】
0x68C32E | 【L1】SSE 平面变换 + 归一化（与 68B22E 同构）【推断】
0x68C4BE | 【L1】SSE sqrt 归一化变体【推断】
0x68C590 | 【L1】x87 标量双 sqrt 变体（与 68B630 成对）【推断】
0x68C6BE | 【L1】SSE 变换（shuffle×9，与 68B75E 同构）【推断】
0x68C876 | 【L1】3DNow! 归一化混合实现（_m_pfrcp + x87 sqrt/rsqrt 混合，AMD 入口）【推断】
0x68C929 | 【L1】3DNow! 大函数：四元数/轴角→矩阵（137 行，rcp4/subr2 直证旋转公式）【推断】
0x68CB45 | 【L1】3DNow! 四元数→矩阵变体（与 68C929 成对，另一寄存器布局）【推断】
0x68CD5A | 【L1】3DNow! userpurge（half=0.5f×2，半像素/半角缩放）【推断】
0x68CE1F | 【L1】3DNow! 小型变换（cvtsi 整数入参）【推断】
0x68CE8B | 【L1】3DNow! 中型变换（rcp+pfacc，矩阵×向量 + 1/w）【推断】
0x68CFC2 | 【L1】3DNow! 归一化（rcp×3 = 三分量倒数）【推断】
0x68D04F | 【L1】3DNow! 小型操作【推断】
0x68D0E1 | 【L1】3DNow! pfacc 点积归一化【推断】
0x68D145 | 【L1】3DNow! 变换 + 1.0f 组合（齐次 w 分量处理）【推断】
0x68D1EC | 【L1】3DNow! userpurge（rcp+sqrt+rsqrt 全套，综合变换）【推断】
0x68D291 | 【L1】★ 3DNow! 批量顶点变换管线（382 行，rcp/pfacc×20 = 多矩阵×多顶点全展开，软渲染顶点路径）【推断】
0x68DCEA | 【L1】★ 3DNow! 4x4 矩阵乘（pfacc×16 = 16 个点积直证，D3DXMatrixMultiply AMD 版）【推断】
0x68DF1C | 【L1】3DNow! 矩阵行列式/伴随计算（pfacc 密集）【推断】
0x68DFEB | 【L1】3DNow! 单位矩阵构造（1.0f 对角 + femms）【推断】
0x68E03B | 【L1】3DNow! 矩阵变换中型【推断】
0x68E126 | 【L1】3DNow! 矩阵构造/缩放（1.0f×3 对角）【推断】
0x68E202 | 【L1】混合路径小型变换（SSE+3DNow! 混编，整数入参 cvtsi×7）【推断】
0x68E265 | 【L1】混合路径小型变换变体【推断】
0x68E2C3 | 【L1】混合路径小型变换变体【推断】
0x68E31C | 【L1】3DNow! 矩阵装配（三整数入参 → 单矩阵写入，与 68E2C3 成对）【推断】
0x68E363 | 【L1】混合路径小型变换变体【推断】
0x68E3D2 | 【L1】3DNow! 矩阵转置/轻操作（cdecl 单参）【推断】
0x68E40B | 【L1】混合路径小型变换变体【推断】
0x68E4DD | 【L1】★ 3DNow! 四元数→旋转矩阵（x²/y²/z² 与 1-2(y²+z²) 组合直证，D3DXQuaternionRotationMatrix AMD 版）
0x68E592 | 【L1】★ 3DNow! 仿射组合巨型（967 行，7 参：四元数×缩放×平移全展开，D3DXMatrixAffineTransformation AMD 版）
0x68F72A | 【L1】3DNow! 四元数→矩阵（fabs×3 半角规范化 + 0.5f，与 68E4DD 同族）
0x68F8CF | 【L1】3DNow! 缩放+旋转+平移组合（对角 1.0f 基阵 → 四元数块 → 平移列，AffineTransformation 变体）
0x68FE03 | 【L1】3DNow! 组合变换（310 行，矩阵初始化 1.0f + 复合）【推断】
0x68FF1C | 【L1】★ 3DNow! 仿射组合巨型（946 行，7 参，与 68E592 成对的另一展开布局）【推断】"""

ref = Refiner(PATH)
ref.set_header(HEADER.split("\n"))
ref.build(parse_sections(SECTIONS), parse_annots(ANNOTS))
assert ref.verify_lossless(), "lossless verification failed"
ref.save(PATH)
ob, cb = Refiner.braces(PATH)
print("OK d3d8_loader.cpp: sections=3 annots=%d va=%d braces=%d/%d" % (
    len(parse_annots(ANNOTS)), Refiner.count_vas(PATH), ob, cb))
assert Refiner.count_vas(PATH) == 73
