# -*- coding: utf-8 -*-
"""socket_ime.cpp L1 精修：ISocket/nIIme/NCEdit 三自证类 + 全函数注释升级。
纯插入式（函数体零改动），运行后自动做无损自验。注释以文本块定义，规避引号转义。"""
import re
import sys
sys.path.insert(0, r"E:\DOF LX\【40JP逆向源代码工程】\work")
from refine_l1_tool import Refiner, BANNER, VA_RE, parse_sections, parse_annots

PATH = r"E:\DOF LX\【40JP逆向源代码工程】\src\platform\socket_ime.cpp"

HEADER = r"""
// ============================================================
// 【40JP逆向源代码工程】src/platform/socket_ime.cpp
// 模块: 0x69 区 —— ISocket/nIIme/NCEdit 三自证类 + 3DNow! 数学 + CRT 尾
// 来源: work/dump_tail.txt  区块: 0x690000-0x6A0000
//
// 【L1 精修层】本文件已完成 L1 精修（注释级），内容:
//   1) 分区组织（9 个 SECTION，按 VA 升序，见下方索引）
//   2) ISocket 网络栈全链标注（Init/Connect/OnSocketMsg/Send/Recv/Close/Cleanup，
//      与 src/nengine/net_socket.cpp 发包 API 的两级关系见 SECTION 4 说明）
//   3) NCEdit 聊天输入控件 + nIIme IME 组合处理链标注（SJIS 双字节逐字节佐证）
//   4) CWSBuffer 环形缓冲（ISocket 底层）方法体标注
//   5) 其余函数逐个一行语义注释（【推断】= 无字符串直证，按调用/常数佐证）
// 函数体逻辑与 L2 转写层逐字一致，未做任何改动（剥离本文件插入的注释行可
// 逐字节还原 L2 原文；自验见 work/refine_l1_socket_ime.py 运行时断言）。
//
// 本区重要结构结论（本次精修推理）:
//   - 三个 Neople 自证类全在本文件: ISocket(0x6985F0-0x698F80，网络) /
//     nIIme(IME 组合泵，主体 sub_69BE10 IsEndComposition) /
//     NCEdit(0x69AB50-0x69BE10，聊天输入控件)。自证串: "Disconnect %d" /
//     "ISocket::Send() : nWOULD_BLOCK_COUNT > 10000" / "NCEdit::Empty() call" /
//     "nIIme::IsEndComposition()...pCompositBuffer_*"。
//   - CWSBuffer（vtable off_749D94）= ISocket 收发环形缓冲，方法体物理上散落
//     文件尾 SECTION 8（0x69DE60-0x69DF70），由 ISocket 直接字段访问。
//   - SJIS 双字节判断以"负字节 == -127..-97 / -32..-22 / -19/-18/-6/-5/-4"
//     的巨型 or 链直证（0x81-0x9F / 0xE0-0xEF 首字节表），日化客户端关键证据。
//   - 0x691070-0x6957C0 的 3DNow!(_m_pfmul/_m_pfacc/_m_pswapd) 数学群与
//     d3d8_loader.cpp 的 SSE 群同构 —— 同一 D3DX 风格数学库的 AMD 分支，
//     与 missing_tail.cpp 0x680134/0x680380 (AuthenticAMD/UnknownVendr)
//     CPU 检测构成"厂商分派"关系。
//
// 交叉引用（0x69 区缺漏 24 函数在 missing_tail.cpp，计划内不精修）:
//   导入 thunk: ImmGetContext/ImmNotifyIME/ImmReleaseContext/
//   ImmGetCompositionStringA/ijl15_3/VerQueryValueA/GetFileVersionInfoA/
//   IcmpSendEcho/gethostbyname/ntohl/FSOUND_* —— 与本文件 SECTION 2 同族；
//   小桩: sub_69A4C0(虚表+0x10 转发，紧邻 69A4D0)、sub_69C690/69C6A0
//   (*(this+28)+112/+116 getter，紧邻 69C6B0)、sub_69CFB0(→sub_69C7B0 转发)。
//
// 函数数: 269
// ============================================================"""

SECTIONS = r"""
@0x691070
// ============================================================
// SECTION 1  3DNow!/MMX 向量与调色板数学（0x691070-0x6957C0）
// ------------------------------------------------------------
// _m_pfmul/_m_pfadd/_m_pfacc/_m_pswapd/_m_pfrcp* 为 AMD 3DNow! 指令，
// 成对 float 语义（0x3F000000=0.5f / 0x3F800000=1.0f 双字对）。
// 与 d3d8_loader.cpp 的 SSE 群（0x68A4B0-0x68C6BE）互为 CPU 厂商分支；
// 分派上游证据: missing_tail.cpp 0x680134/0x680380 (AuthenticAMD/UnknownVendr)
// 及 0x6759F7 ___sse2_available_init。尾段 0x694AC0 起为 SSE exp2 多项式。
// ============================================================
@0x695BB8
// ============================================================
// SECTION 2  导入 thunk + COM/BSTR 助手（0x695BB8-0x697B00）
// ------------------------------------------------------------
// DirectInput8Create / Imm(IME) / ijl15(Intel JPEG) / Icmp(ping) /
// GetFileVersionInfo 导入转发表；0x697630 起为 comutil（_com_error /
// ConvertStringToBSTR）支撑例程。配套 thunk（ImmGetContext/IcmpSendEcho/
// gethostbyname/ntohl/VerQueryValueA 等）在 missing_tail.cpp 0x69 区。
// ============================================================
@0x697B20
// ============================================================
// SECTION 3  字符串键缓存/定时器映射助手（0x697B20-0x6983E0）
// ------------------------------------------------------------
// std::map<std::string,T>（MSVC _Tree 红黑树）实例化方法群 +
// timeGetTime 过期驱逐的"带 TTL 资源缓存"（0x697D00/0x6983E0）。
// 消费方为渲染/声音资源的句柄缓存（【推断】无自证串）。
// ============================================================
@0x6985F0
// ============================================================
// SECTION 4  ★ ISocket 网络栈 + CWSBuffer 环形缓冲（0x6985F0-0x698F80）
// ------------------------------------------------------------
// ISocket = Neople 自研 TCP 封装（WSAAsyncSelect 异步模型）。
//   对象布局: +0 vtable / +4 已初始化(BYTE) / +8 已连接(BYTE) / +12 HWND
//     / +16 WSADATA(400B) / +416 IP串[16] / +432 端口 / +436 SOCKET
//     / +440 接收 CWSBuffer* / +444 发送 CWSBuffer*
//   错误码: 0x80000001 未初始化 / 0x80000002 重复初始化 / 0x80000008 new失败
//     / 0x80000050 WSAStartup失败 / 0x80000051 socket失败 / 0x80000053 已连接
//     / 0x80000054 未连接 / 0x80000055 套接字错误 / 0x80000000 一般失败
// 与上层（src/nengine/net_socket.cpp 0x423200/0x423380 packet Begin/Commit）
// 的两级关系:
//   应用层 Begin(type) ──► sub_698D70 申领发送上下文(=发送环读指针)
//                      ──► sub_698C80(hdr,12) 包头入发送环（sub_69DF10 Write）
//   应用层 Commit()    ──► sub_698A50 ISocket::Send 真正 send() 下网卡
//   接收方向: WndProc(0x464 消息) ──► sub_698F80/698CA0 OnSocketMsg
//             ──► sub_698B50 Recv 收入接收环 ──► sub_698C50 取包(sub_69DF70 Read)
//   连接/断开: sub_698DC0 Connect / sub_6989F0 Close("Disconnect %d" 自证)
// CWSBuffer（vtable off_749D94）对象布局: +0 vtable / +4 就绪(BYTE)
//   / +8 缓冲基址 / +12 容量 / +16 读指针 / +20 写指针；
//   其方法体在 SECTION 8（0x69DE60-0x69DF70）。
// ============================================================
@0x698F90
// ============================================================
// SECTION 5  UI 基础设施：光标闪烁/子系统注册/截屏/数字标签/CNUIObject
//            （0x698F90-0x69AB00）
// ------------------------------------------------------------
// ·0x698F90-0x699070 光标闪烁节拍（dword_79D100[16] 时间表，NCEdit +132 内嵌）；
// ·0x6990D0-0x6992C0 子系统注册表（按 bank 登记条目，【推断】）；
// ·0x6993C0-0x69A110 截屏器（单例 dword_9D3BDC/9D3BE0）+ 数字标签；
// ·0x69A4D0-0x69AB00 CNUIObject 基类（渲染/命中/拖动递归）。
// 全局: dword_9D3BC8 渲染器(vt+72 Begin/+76 End/+80 DrawText, sub_416430)
//       dword_9D3BD8 字体引擎(vt+16 测宽/+24 适配截断)
//       dword_89D26C 鼠标消息态(1=按下/8/16)  dword_9D3BCC 控件 id 分配器
//       dword_79BE3C/40/44/54/58/5C/60/64 按下/拖动/悬停/焦点控件 id 组
// ============================================================
@0x69AB50
// ============================================================
// SECTION 6  ★ NCEdit 聊天输入控件 + nIIme IME 处理链（0x69AB50-0x69BE10）
// ------------------------------------------------------------
// NCEdit（"NCEdit::Empty() call" 自证）= 单行聊天/输入框控件。
//   对象布局: +4 激活(BYTE) / +8 类型(=8) / +12 控件id / +16 可见 / +17 焦点
//     / +104 最大输入字节 m_MaxLen / +108 缓冲尺寸 / +112 滚动基准
//     / +116 组合提交中(BYTE) / +117 组合进行(BYTE) / +120 光标 m_dwCaretPos
//     / +128 颜色 / +132 光标闪烁对象(0x698F90 群)
//     / +164..+176 主串{+164就绪,+168指针,+172长度,+176容量}
//     / +184..+196 次串(IME 组合中) / +200 渲染用 std::string / +208 m_String
//     / +212 长度 / +220 内容变更(BYTE) / +221 IME重聚焦(BYTE) / +224 IME下划线光标
// nIIme = IME 组合泵（"nIIme::IsEndComposition()" 自证），全局状态:
//   byte_8A6A98[0x400] 组合串A / byte_8A6698[0x400] 组合串B
//   byte_8A6298[0x400] 虚键缓冲 / byte_8A6E98 提交标志 / byte_8A6E9A 上屏标志
//   byte_8A6E9B 串B有效 / byte_8A6E9C 组合中 / dword_8A6EA8 组合长度
//   dword_8A6EB8 字体引擎备份 / byte_8A6EAC 引擎切换标志
// 处理链（逐帧）: WndProc 填 IME 全局 ──► sub_69BE10 IsEndComposition
//   ──► sub_69B590 宽度适配(字体引擎 vt+24) ──► 光标处拼接(sub_402DE0/402CB0)
//   ──► 超长则 sub_407400 completeIMEComposit 强制上屏。
// 键盘链: sub_69BAF0 方向/行首行尾(35 End/36 Home/37 Left/39 Right 直证)
//   ──► sub_69B7B0 退格 / sub_69B970 删除（均 SJIS 双字节感知，2 字节回退）。
// SJIS 首字节表: -127..-97(0x81-0x9F) / -32..-22(0xE0-0xEF) / -19/-18/-6/-5/-4
// ============================================================
@0x69C2D0
// ============================================================
// SECTION 7  UI 派生控件族（0x69C2D0-0x69DE30）
// ------------------------------------------------------------
// 类型号 = 对象 +8 字段（ctor 直证）: 9 文本列表 / 3 图片框 / 1 按钮
//   / 5 滑条 / 7 标签页 / 6 图片组 / 2 标签。均为 SECTION 5 CNUIObject 派生，
//   纹理经 sub_418DD0 句柄表解析；绘图走渲染器 dword_9D3BC8 或图片 vt+152/+128。
// ============================================================
@0x69DE60
// ============================================================
// SECTION 8  CWSBuffer 方法体（ISocket 底层环形缓冲）+ ijl15 JPEG 编码
//            （0x69DE60-0x69E010）
// ------------------------------------------------------------
// 物理位置在文件尾、逻辑归属 SECTION 4 的 ISocket 收发缓冲类
// （vtable off_749D94，由 0x6985F0 构造）。布局: +4 就绪 / +8 基址
// / +12 容量 / +16 读 / +20 写。0x69E010 为截屏 JPG 路径的 Intel ijl15 编码。
// ============================================================
@0x69E1A8
// ============================================================
// SECTION 9  CRT 尾部：FSOUND/文件/longjmp/x87 数学（0x69E1A8-0x69FE27）
// ------------------------------------------------------------
// MSVC 静态 CRT 与 FMOD 导入 thunk（配套在 missing_tail.cpp）。
// 计划内以标准库语义注释，不逐行还原（docs/04 §CRT 替代原则）。
// ============================================================"""

ANNOTS = r"""
0x691070 | 【L1】3DNow! vec2 仿射变换：2x2 矩阵(_m_pswapd 转置)×两点 + 平移
0x691352 | 【L1】3DNow! vec2 投影变换：含 1/w 透视除法（_m_pfrcp/_m_pfrcpit2 迭代精化）
0x69174A | 【L1】3DNow! vec2 仿射变换变体（与 691070 同族，系数排布不同）
0x691A7E | 【L1】3DNow! vec2 变换 + 0.5 偏置半像素对齐（×0x3F000000=0.5f）
0x691B72 | 【L1】3DNow! vec2 双分量缩放/插值基元
0x691BD1 | 【L1】3DNow! 三点插值混合（a5/a6 = 两轴插值参数）
0x691C42 | 【L1】3DNow! 三点插值混合变体（控制点独立传入）
0x691D16 | 【L1】3DNow! 三点插值混合变体（与 691C42 成对）
0x691E00 | 【L1】3DNow! vec2 投影：mv 变换 + 1/w（_m_pfrcpit1/pit2 Newton 迭代）
0x691E8B | 【L1】userpurge 多顶点变换流水线（编译器内联展开的批量变换，寄存器传参）【推断】
0x692052 | 【L1】thunk → sub_692057（同体跳转）
0x692057 | 【L1】userpurge MMX 大混合体：多源像素/顶点混合流水线【推断，无自证串】
0x6921DF | 【L1】userpurge MMX 大混合体变体【推断】
0x692405 | 【L1】userpurge MMX 大混合体变体（最长 ~1.4KB）【推断】
0x6928FA | 【L1】3DNow! vec2 叉积/旋转组合（_m_pfacc 半和半差指令）
0x692A2C | 【L1】3DNow! 双线性插值 (1-u-v)A + uB + vC（a5/a6 = u/v 定点输入）
0x692A9D | 【L1】3DNow! 三次插值：t^3-2t^2 基函数（0x40400000=3.0f 系数直证 Hermite/Bezier）
0x692B65 | 【L1】3DNow! 三控制点插值变体（u^2/u^3 基展开）
0x692E28 | 【L1】3DNow! vec2 线性变换（含 _m_pswapd 转置乘）
0x692E8C | 【L1】3DNow! vec2 变换 + 倒数（_m_pfrcp）
0x692F0A | 【L1】float[16] 序列 min/max 归一化（调色板动态范围统计）【推断】
0x692F56 | 【L1】fastcall float 数组范围统计（min/max 输出）【推断】
0x692FD4 | 【L1】usercall 双 float 数组范围统计变体【推断】
0x693004 | 【L1】Gamma/RAMP 重建辅助（693121 调用前的表初始化）【推断】
0x693027 | 【L1】thiscall 颜色表对象方法（返回表指针）【推断】
0x693121 | 【L1】Gamma/RAMP 重建：16 槽浮点表 min/max → 8 段索引重映射（a4=6/8 两种格式）
0x693426 | 【L1】Gamma/RAMP 重建变体（整数索引路径，与 693121 成对）
0x693991 | 【L1】调色板量化：16 项 RGBA 浮点 → 统计 <0.5 透明项 → 输出索引色（RGB5551 方向）【推断】
0x694195 | 【L1】DXT 调色板展开：8 项 4bit 索引 → float RGBA（×0.06666667 = /15）
0x6942D3 | 【L1】DXT1 调色板展开 + 两端色 4bit 解码（调 sub_694195）
0x694356 | 【L1】DXT 调色板展开：3bit 索引（×0.14285715 = /7）+ RGB 灰度两端插值
0x694492 | 【L1】调色板索引表写入（int 表 → float 表搬运）【推断】
0x6944AA | 【L1】调色板索引表写入变体（uint32 表）【推断】
0x6945BE | 【L1】标量系数 × 调色板缩放（亮度调节方向）【推断】
0x694A10 | 【L1】DXT 调色板展开变体（与 694195 同构）
0x694A56 | 【L1】调色板索引表写入变体（uint32 输入表）
0x694A8A | 【L1】标量 × 调色板缩放变体
0x694AC0 | 【L1】SSE exp2 多项式近似（2^x，4 路并行，系数表 xmmword_79A8*）
0x694CB0 | 【L1】SSE exp2 userpurge 变体（xmm0 寄存器传参，整数格点修正）
0x694DA0 | 【L1】SSE 2^x 精化实现（5 阶多项式，与 _pow_pentium4 配套）
0x694FC0 | 【L1】空桩（nop/对齐，IDA 空函数）
0x6950C0 | 【L1】空桩（同上）
0x6952C0 | 【L1】空桩（同上）
0x6956A0 | 【L1】3DNow! fabs：mm0 低双字符号位清零（v & 0x80000000 ^ v）
0x6957C0 | 【L1】3DNow! fabs（与 6956A0 完全同体，双入口）
0x695BB8 | 【L1】导入 thunk → __imp_DirectInput8Create（输入系统 API 面）
0x697606 | 【L1】导入 thunk → __imp_ImmSetCompositionWindow（nIIme 组合窗口定位 API）
0x697618 | 【L1】导入 thunk → __imp_ImmGetCandidateListA（nIIme 候选字列表 API）
0x69762A | 【L1】导入 thunk → __imp_ijl15_2（Intel JPEG 库 ijlInit，截屏 JPG 用）
0x697630 | 【L1】comutil 抛错助手：off_79BE04(hr,0)（noreturn，com_error 路径）【推断】
0x697640 | 【L1】_com_util::ConvertStringToBSTR：MultiByteToWideChar + SysAllocString（>4KB 走堆，带 SEH）
0x6977A0 | 【L1】_com_util::ConvertBSTRToString：WideCharToMultiByte（失败链 → 697630 抛错）
0x697900 | 【L1】_com_raise_error：构造 _com_error{wcode,hresult} 并 _CxxThrowException
0x697940 | 【L1】_com_error 构造：vftable + wcode(+4)/hresult(+8)/ErrorInfo AddRef
0x697980 | 【L1】_com_error 析构：ErrorInfo Release
0x6979B0 | 【L1】_com_error scalar deleting destructor（a2&1 → free）
0x697A07 | 【L1】_com_error::ErrorInfo 读取（this+2 槽）【推断】
0x697A37 | 【L1】_com_error 辅助方法（WCode/Description 路径）【推断】
0x697AE2 | 【L1】导入 thunk → __imp_IcmpCloseHandle（ping 探测；配套 IcmpSendEcho 在 missing_tail）
0x697AEE | 【L1】导入 thunk → __imp_IcmpCreateFile（ICMP 句柄打开）
0x697B00 | 【L1】导入 thunk → __imp_GetFileVersionInfoSizeA（版本查询；配套 VerQueryValueA 在 missing_tail）
0x697B20 | 【L1】std::string 键节点清理助手（红黑树节点析构前段）【推断】
0x697B30 | 【L1】映射节点构造：key=字符串(+8 SSO 布局)/值 a2、a4 入槽，容量 15 直证 std::string
0x697BD0 | 【L1】std::map clear：红黑树递归释放（erase 路径）
0x697BE0 | 【L1】varargs 格式化 → 虚调用 +8（格式化查询/日志入口）【推断】
0x697C10 | 【L1】varargs 格式化到全局 byte_9D38C8 → 虚调用 +8（带静态缓冲的格式化入口）
0x697C40 | 【L1】std::map<std::string>::lower_bound：memcmp 字典序下降（+45 头哨兵判据）
0x697D00 | 【L1】★ TTL 缓存轮询：timeGetTime() 驱逐过期条目（+66=存活时限, +68=条目链, 虚+44 删除回调）
0x697D80 | 【L1】map find-or-insert 前置：lower_bound + 键比较（sub_40BF00 = char 比较）
0x697DF0 | 【L1】缓存条目构造：key/value/期限入槽，+44=红黑树颜色位，+45=哨兵位
0x697E50 | 【L1】operator new(0x30) + 条目构造（std::map 节点分配包装）
0x697EE0 | 【L1】std::map insert + 红黑树再平衡（"map/set<T> too long" 0x7FFFFFE 容量哨兵直证）
0x698090 | 【L1】缓存条目删除：虚 +52 通知 + free（erase 收尾）
0x6980D0 | 【L1】std::map erase + 红黑树删除再平衡（与 697EE0 成对）
0x698250 | 【L1】全局字符串表查找/登记（unk_9D39C8，资源名归一化）【推断】
0x6983E0 | 【L1】★ 带 TTL 的资源缓存查询：key = 前缀(+8)+Src，命中刷新 timeGetTime，未命中虚 +48 加载并 insert
0x6985F0 | 【L1】CWSBuffer 构造：vtable=off_749D94，调 sub_69DE60 清零五字段
          （+4 就绪/+8 基址/+12 容量/+16 读/+20 写）。ISocket 在 sub_698870 中
          operator new(0x18) 后构造两个实例：+440 接收环、+444 发送环。
0x698640 | 【L1】CWSBuffer scalar deleting destructor（调 sub_698670 析构后按 a2&1 free）
0x698670 | 【L1】CWSBuffer 析构体：释放缓冲（vtable 槽 1，Cleanup 经 (**v2)(v2,1) 虚调用到达）
0x6986C0 | 【L1】Concurrency::_AutoDeleter<_TaskProcHandle> 析构（CRT/PPL 符号混入，与本区无关，计划内保留原名）
0x6986D0 | 【L1】std::allocator<char> 式助手（返回 this）【推断】
0x698720 | 【L1】CWSBuffer 析构前段（被 698840 调用；与 698670 同族另一实例布局）
0x6987C0 | 【L1】★ ISocket::Cleanup：WSACleanup() + 释放收发两环（虚调用槽1）+ 清字段
          +8 已连接=0、memset(+16,0x190) 清 WSADATA、+416..436 清零。返回 1。
0x698840 | 【L1】CWSBuffer deleting destructor（调 sub_698720，a2&1 → j__free）
0x698870 | 【L1】★ ISocket::Init(hwnd, Size)：网络栈启动
          流程: 重复初始化检查(+4→0x80000002) → 存 HWND(+12) → operator new(0x18)x2
          构造接收环(+440)/发送环(+444) → 两次 sub_69DE80(Size) 分配环形缓冲 →
          WSAStartup(0x202) 成功置 +4=1 返回 1；
          失败: WSA→0x80000050，分配失败→sub_6987C0 清理后透传/0x80000008。
0x6989F0 | 【L1】★ ISocket::Close：断开连接（"Disconnect %d" 自证串）
          流程: off_77F3FC 日志 → closesocket(+436) → SOCKET=0 →
          发送环(+444)与接收环(+440)读/写指针(+16/+20)清零 → +8 已连接=0。返回 1。
0x698A50 | 【L1】★ ISocket::Send：把发送环数据真正 send() 下网卡
          （自证串 "ISocket::Send() : nWOULD_BLOCK_COUNT > 10000"，IDA 串名 aIsocketSendNwo）
          流程: 未初始化→0x80000001 / 未连接→0x80000054；
          待发 = 写指针(+20)-读指针(+16)（对 +444 发送环）；v4==0 直接返回 1；
          send() 循环推进读指针，满圈(+16==+20)归零对；
          WSAEWOULDBLOCK(10035) → Sleep(1) 重试计数；
          >10000 次 → 打日志返回 0x80000000；
          其他错误 → 打 "ISocket::Secv() : SocketError %d"（原版串拼写如此，实为 Send）
          → sub_6989F0 Close → 返回 0x80000055。上层调用方: net_socket.cpp Commit()。
0x698B50 | 【L1】★ ISocket::Recv：ioctlsocket(FIONREAD=0x4004667F) 查可读量并 recv() 入接收环
          流程: 未初始化→0x80000001 / 未连接→0x80000054；FIONREAD==0 → 返回 0；
          recv 循环写入 +440 环（基址+8 + 写偏移+20，容量+12 封顶），累计 >= argp 止；
          WSAEWOULDBLOCK → 返回 0（正常轮询路径）；其他错误 → 打
          "ISocket::Recv() : SocketError %d" → Close → 0x80000055。
0x698C50 | 【L1】★ ISocket::取包：激活且已连接时 sub_69DF70(接收环, len) = Read/出队
          （读指针 +16 前进 len，读完归零对；越界写 SocketLog.log "Devector Error"）。
0x698C80 | 【L1】★ ISocket::发写入队：激活时 sub_69DF10(len, buf, size) = Write/入队；
          上层 net_socket.cpp 的 packet_send_begin 用它入 12 字节包头框架。
0x698CA0 | 【L1】★ ISocket::OnSocketMsg：0x464 自定义窗口消息处理器（WSAAsyncSelect 回调）
          a5 低字=事件: 16=FD_CONNECT → 高字错误码非 0 → Close+日志；
          成功 → 重新 WSAAsyncSelect(+436, HWND+12, 0x464, 35=FD_READ|FD_WRITE|FD_CLOSE)
          并置 +8=1（已连接）；32=FD_CLOSE → 日志+Close 返回 1；其他事件忽略。
0x698D70 | 【L1】★ ISocket::GetSendReadPtr：返回发送环(+444)当前读位置（基址+8+读偏移+16）；
          即 net_socket.cpp 注释中的"申领发送上下文"（12 字节包头写入起点）。
0x698D80 | 【L1】★ ISocket::GetRecvReadPtr：返回接收环(+440)当前读位置（基址+8+读偏移+16）
0x698D90 | 【L1】★ ISocket::IsRecvPending：已初始化 && 接收环就绪(+4) && 读(+16)<写(+20)
0x698DC0 | 【L1】★ ISocket::Connect(ip, port)：异步 TCP 连接（docs/13 网络层衔接）
          流程: 未初始化→0x80000001 / 已连接→0x80000053；
          strcpy IP 到 +416、端口入 +432 → socket(AF_INET=2, SOCK_STREAM=1, IPPROTO_TCP=6)
          → setsockopt(SOL_SOCKET=0xFFFF, SO_SNDBUF=4097, 100000)
          → setsockopt(SO_RCVBUF=4098, 0x989680【原值直写，数值 0x989680=10,000,000，推断 10MB】)
          → setsockopt(optname=1【原值，WinSock 语义 SO_DEBUG】, 1)
          → WSAAsyncSelect(0x464 自定义消息, FD_CONNECT=16)
          → inet_addr(+416)+htons(+432) → connect()；
          connect 返回 -1 时仅取 WSAGetLastError 不视为失败（异步模型：结果经
          0x464/FD_CONNECT 回调 698CA0）。socket 失败→0x80000051。
0x698F80 | 【L1】ISocket::OnSocketMsg 转发桩：原参数尾调 sub_698CA0（第二个入口）
0x698F90 | 【L1】光标闪烁对象·焦点获得：+12 基准有效(+13=1)时记录锚点
          +16 = a3 - dword_79D100[bank(+8)]；拖动分支(+14)记 +20=a3、+24=a2。
          NCEdit 于 +132 内嵌此对象（sub_69AB90 Create 时 sub_698F90(this+132,1,0)）。
0x698FE0 | 【L1】光标闪烁·焦点丢失：清 +14 并把 +20 锚点换算回 +16（- dword_79D100[bank]）
0x699000 | 【L1】光标闪烁·取当前节拍：+14 置位返回 +20，否则 +16 + dword_79D100[bank]
0x699030 | 【L1】光标闪烁·刷新：按 +16+dword_79D100[bank] 重算 +20 并置 +14=1
0x699070 | 【L1】光标闪烁·换 bank：a2<0x10 时迁移基准（+16/+20 换算保持相位）
0x6990D0 | 【L1】子系统注册表·激活：置 bank id(+8) 并 +4=1【推断】
0x699100 | 【L1】子系统注册表·清空条目（步进 4/28 双数组，std::string 赋空串释放）【推断】
0x6991D0 | 【L1】子系统注册表·释放内存（free 指针数组 + sub_4166A0 清第二数组）【推断】
0x699230 | 【L1】子系统注册表·销毁全部：逐项虚调用槽1释放后 free 数组、+4=0【推断】
0x6992C0 | 【L1】子系统注册表·登记：new(0x68)+sub_419480(bank) 构造，拷贝名称 Src【推断】
0x6993C0 | 【L1】截屏·探测空闲序号：sprintf("%s%05d", 前缀, i) 后缀 .BMP/.JPG，
          CreateFileA(GENERIC_READ, OPEN_EXISTING) 探测，两者都不存在即返回 i。
0x6994E0 | 【L1】截屏·D3DFORMAT → 扩展名映射：0/1/2→dword_749ED4/749ECC/.TGA，
          3→.PNG / 4→DDS / 5→.PPM / 6→.DIB / 7→.HDR / 8→.PFM
0x699550 | 【L1】截屏单例 scalar deleting destructor（vtable off_749F18，refcount 清零）
0x699580 | 【L1】截屏单例 Release：--dword_9D3BDC 归零时虚删 dword_9D3BE0
0x6995B0 | 【L1】截屏单例 Acquire：refcount++ 或 new(0x60) 构造
          （vtable off_749F1C；3 个 std::string SSO 成员 +8/+32/+56，容量 15 直证）
0x699620 | 【L1】截屏单例 deleting destructor（调 sub_699650 析构体）
0x699650 | 【L1】截屏单例析构体：释放 3 个 std::string（容量>=0x10 才 free）+ refcount 清零
0x6996D0 | 【L1】std::string 拼接助手：a2 串 + a3 串 → 结果写回 a1（sub_4014D0 assign）
0x699760 | 【L1】截屏参数装载：Singleton() 后写 名称1(+4)/名称2(+32)/标志(+88)/id(+92)
0x6997C0 | 【L1】★★ ScreenShot 主函数（自证串 "ScreenShot File [" / "ScreenShot Capture FAIL!!"）
          usercall: this=截屏配置@ecx, a2=D3D 设备@edi。
          流程: 探测序号(sub_6993C0) → 拼 "%s%05d"+扩展名(sub_6994E0) →
          按配置[22] 分派: =1 且设备就绪 → 设备 vt+128/+108 取 640x480 前后缓冲 →
          按表面格式(0x15/0x16=32bit / 0x17-0x19=16bit R5G6B5/R5G5B5X1) 软件转换
          3 字节 BGR → sub_69E010(ijl15) 编码 JPG → CreateFileA+WriteFile 落盘；
          其他格式 → sub_680134（missing_tail 0x68 区）直接存 .BMP →
          拼 "ScreenShot File [...is Saved." 日志串返回；
          失败返回 "ScreenShot Capture FAIL!!"。
0x699E20 | 【L1】数字标签（CNumberLabel 类）构造：字段清零，+20=-1(未知值)，返回 0
0x699E40 | 【L1】数字标签 Create：+8/+12 位置、+28 字体句柄表、+32 位数上限；
          new(a5+2) 数字缓冲（+16），memset 清零，+4=1。
0x699EC0 | 【L1】数字标签 Destroy：free 数字缓冲(+16) 并清字段，+20=-1
0x699F00 | 【L1】数字标签 SetNumber：_itoa(10 进制) 后逐字节 -47 映射为字形索引
          （'0'=48→1 .. '9'=57→10；'-'+45→-2，缺字/负号占位），+24=逐字填充模式开关。
0x699FC0 | 【L1】数字标签 GetWidth：按模式 0/2 累计每字形宽度
          （sub_418DD0 字形句柄 + 36 处宽度 + 字距 +40），负值走 '-' 字形占位。
0x69A110 | 【L1】数字标签 Draw：按对齐/着色分派图片 vt+152（普通）或 vt+128（10 参含 Alpha 版）
          （1065353216=1.0f / 1333788672=0x4F800000 原值保留）；位数 >9 时 clamp 到 10^9-1。
0x69A4D0 | 【L1】CNUIObject::IsMouseOver：可见(+17) && dword_79BE3C==id(+12) && dword_89D26C==1
          （1 = WM_LBUTTONDOWN 活动态）
0x69A500 | 【L1】CNUIObject::IsHover 查询：可见(+17) && dword_79BE44==id(+12)
0x69A520 | 【L1】CNUIObject 构造：字段清零，+16/+17 可见与激活置 1，
          id(+12) = dword_9D3BCC++（全局控件 id 分配器），+64=1，sub_6569D0(+92) 初始化子结构
0x69A580 | 【L1】CNUIObject::RemoveChild：子链(+24 环形双链)按值摘除并 free，计数(+25)--
0x69A5F0 | 【L1】CNUIObject::MoveWindow(dx,dy)：位置(+5/+6)平移并记旧值(+9/+10)，
          递归平移全部子项（子 +20/+24 相对偏移补偿）
0x69A650 | 【L1】CNUIObject::DragWindow：dword_79BE40 拖动态管理；
          按下时(+17 且 dword_79BE3C==id)记录抓取点(+44/+48 = pt - pos(+20/+24))，
          移动帧按 dword_79BE40==id 调 MoveWindow(pt - 抓取点)
0x69A6E0 | 【L1】CNUIObject::Render 递归：虚槽 +24 绘制自身 → 可见(+21) 时经
          dword_9D3BD8 遮罩或渲染器 vt+72 Begin/+80 DrawText/+76 End 绘制纹理(+13..+20)
          → 遍历子链递归（子可见时 Render，且拖动态调 DragWindow）
0x69A7B0 | 【L1】std::list<T> 尺寸增长哨兵：0x3FFFFFFF 上限，超额抛 length_error（"list<T> too long"）
0x69A840 | 【L1】CNUIObject::PushChild：list 尾插（sub_40C0F0 + 69A7B0 计数）
0x69A890 | 【L1】CNUIObject::HitTest 递归：深度优先命中矩形(+20/+24/+28/+32)，
          dword_89D26C 分派 1/8/16（左键/拖动/滚轮）：命中后按子类型(+8: 2=子容器递归删插、
          8=焦点 id dword_79BE54)写 dword_79BE3C/34/38/44/4C/50/54/58/5C/60/64 状态组
0x69AB00 | 【L1】CNUIObject::Update：可见(+64)时清 dword_79BE34/38/3C/64/44/48 全局状态组，
          然后 HitTest(69A890) + Render(69A6E0) —— 每帧入口
0x69AB50 | 【L1】NCEdit::Init：类型(+8)=8、光标/滚动/IME 标志全清、byte_9D3BE4=0
0x69AB90 | 【L1】NCEdit::Create(maxChars, bufSize)：重复创建→0x80000002；
          最大输入 +104=a2；缓冲尺寸 +108 = (a3==-1 ? 5*a2 : a3)；
          sub_403080 分配主串与滚动表；光标闪烁对象 sub_698F90(this+132,1,0) 初始化；+4=1
0x69AC10 | 【L1】NCEdit::Destroy：全字段复位（与 Init 对应），byte_9D3BE4=0，+4=0
0x69AC60 | 【L1】分页查询：dword_77F400 != -1 即有分页数据【推断】
0x69AC70 | 【L1】分页查询：取当前页最大行号（36 字节/项表 @9032376=0x89D668，dword_77F404=页大小）
0x69ACE0 | 【L1】NCEdit::DrawPageNumbers：a2<600 时按页渲染页码列表（sub_416430 文本绘制，
          当前页 -16725766 高亮色），页脚 "%d/%d"
0x69AEA0 | 【L1】NCEdit::Render 主绘制：SJIS 首字节判定选取主/次串(+160/+180) →
          sub_4030C0 取显示窗口切片 → sub_402AE0 写渲染串 → 遮罩/直绘两条路径；
          焦点态(dword_79BE54==id)时按 sub_699000 节拍（274877907e 低位 / 0x1F4 偶数双路径）
          在测量宽(dword_9D3BD8 vt+16)之后画 "|" 光标
0x69B2E0 | 【L1】NCEdit::滚动窗口：a2=0 右滚 / =1 左滚，步进 = 缓冲(+27)/5，
          滚动时逐字节 SJIS 双字节感知（首字节表）避免劈开全角字符
0x69B590 | 【L1】nIIme 组合串宽度适配：字体引擎 dword_9D3BD8 vt+16 测宽，
          超宽时 vt+24 逐字截断（返回截断长度，a2[result-1]=0）
0x69B600 | 【L1】NCEdit::CaretLeft：光标(+30)左移，跨过 SJIS 双字节则 -2（防劈字）
0x69B740 | 【L1】NCEdit::GetMaxLength?：sub_402EA0(10,32) 后返回主串长度(+168 槽)【推断】
0x69B780 | 【L1】NCEdit::ClearString(str)：memset(str.ptr, 0, cap+1)，长度(+12)=0
0x69B7B0 | 【L1】NCEdit::Backspace：主串(+168)末字节 SJIS 判定 → 退 2 或 1 字节
          （sub_402DE0 区间置 0），长度 -2/-1，随后 69B2E0 左滚同步；
          次串(+184 就绪)存在时优先清次串
0x69B970 | 【L1】NCEdit::Delete：光标处 SJIS 双字节则先 sub_402DE0(+1)，再删 1 字节；
          清 byte_8A6298[0x400] 虚键缓冲与 byte_8A6E9A
0x69BAF0 | 【L1】NCEdit::MoveCaret(key)：37 '%'/VK_LEFT → CaretLeft(69B600)；
          39 '''(VK_RIGHT) → +1 或 SJIS +2；36 '$'/VK_HOME → 光标=0(+112=0)；
          35 '#'/VK_END → 光标=串长(+172)。a2 指向 byte_8A6298 虚键缓冲。
0x69BCC0 | 【L1】NCEdit::Empty（自证串 "\n NCEdit::Empty() call"）：
          全串清零（主/次/三块 IME 全局），标志位清零后 sub_40BE20 打调试串 +
          sub_407430 收尾，byte_9D3BE4=0
0x69BD90 | 【L1】NCEdit::SetStringf：vsprintf → 写主串（sub_402EF0），光标置末尾(69B2E0)，
          次串就绪时清次串。外部注入文本入口。
0x69BE10 | 【L1】★★ nIIme::IsEndComposition：IME 组合泵（自证串 x5）
          仅焦点控件(dword_79BE54==+12)处理；逐帧链:
           1) byte_8A6E98 提交标志 → +116=1；
           2) byte_8A6E9A 上屏标志 → sub_69BAF0 处理虚键，'.'(46)→Delete(69B970)，
              清 A/B/虚键三缓冲与标志；
           3) 否则 +221 IME 重聚焦分支清主串，或恢复字体引擎(dword_8A6EB8)；
           4) 组合串 A(byte_8A6A98) 非空: 首字节==8(VK_BACK)→Backspace(69B7B0)；
              非 ESC(27) → "Composition Success" 路径: 宽度适配(69B590)、
              m_MaxLen(+104) 封顶、光标处拼接(dword_9D3BE8=IME 插入偏移)、+220 变更=1；
           5) byte_8A6E9C 组合中 → 串 B 上屏提交；
           6) 串 B(byte_8A6698) 有效(byte_8A6E9B): 超长 → sub_407400
              completeIMEComposit 强制上屏；否则边组合边拼入主串（+117=1 组合中）；
           7) IME 下划线光标 +224 = dword_8A6EA8 + caret - dword_9D3BE8；
          尾部: 主/次串共存时清次串；最后虚槽 +28 基类 Update。
0x69C2D0 | 【L1】文本列表控件 ctor（类型 9）：+8=9，子槽(+26)=0
0x69C2E0 | 【L1】文本列表 dtor：释放 +104 列表对象（虚删），+8=9 复位，+4=0
0x69C310 | 【L1】文本列表 Render：可见(+16)时把 +20/+24 位置写入列表对象 +32/+36，
          尾调 sub_69D780（GDI/渲染器绘制）
0x69C340 | 【L1】文本列表 Clear：激活时 sub_69D6C0(n)（删 n 行，-1 全清）
0x69C360 | 【L1】文本列表 AddString：激活时 sub_69DB10(文本, 模式)（多行拆分入列）
0x69C380 | 【L1】文本列表 GetCount：+104 对象 +68 槽（行数）
0x69C3A0 | 【L1】文本列表 GetScrollPos：+104 对象 +72 槽
0x69C3C0 | 【L1】文本列表 setter：+104 对象 +92 槽（描边/阴影开关?）【推断】
0x69C3E0 | 【L1】文本列表 setter：+104 对象 +72 槽（滚动位置）
0x69C400 | 【L1】文本列表 setter：+104 对象 +68 槽（可见行数?）【推断】
0x69C420 | 【L1】文本列表 setter：+104 对象 +64 槽（行高?）【推断】
0x69C440 | 【L1】文本列表 setter：+104 对象 +80 槽（滚动上限?）【推断】
0x69C460 | 【L1】文本列表 setter：+104 对象 +62 字节（对齐标志?）【推断】
0x69C480 | 【L1】文本列表 setter：+104 对象 +88 槽（字体颜色 B?）【推断】
0x69C4A0 | 【L1】文本列表 setter：+104 对象 +76 槽（字体颜色 A?）【推断】
0x69C4C0 | 【L1】控件子槽 setter：*(this+26) 槽 +56 = a2
0x69C4D0 | 【L1】文本列表 SetText：虚槽 +12 转发 + 子槽(+26) +8 记录 a2
0x69C500 | 【L1】文本列表 GetCount2：列表对象首尾指针差 >>2（vector<T*> size）
0x69C520 | 【L1】文本列表 Create：new(0x60) → sub_50F0B0(渲染器, a3, a2) 构造行对象入 +104
          （失败→0x80000008；成功 +4=1）
0x69C5D0 | 【L1】控件复位：子槽(+26/+27/+28)清零
0x69C5E0 | 【L1】控件复位：+104/+108/+112 清零，+4=0
0x69C600 | 【L1】滑条/列表滚轮：可见且激活时按 dword_79BE34==id 判定，
          经 +112 分组对象 +112 槽 ±1 调 sub_69CAF0(SetScrollPos)
0x69C650 | 【L1】控件 Enable(a2)：+112 组对象虚槽+40，+104/+108 两子对象 +17 = a2!=0
0x69C680 | 【L1】sub_69CAF0 的 stdcall 包装（导出/回调形态）
0x69C6B0 | 【L1】图片框 ctor（类型 3）：+8=3，子槽(+26)=0
0x69C6C0 | 【L1】图片框 SetTexture：sub_418DD0(a3) 句柄解析入 +104，
          宽高取句柄 +36/+40 到 +28/+32；a2=0 时跳过纹理
0x69C710 | 【L1】图片框 dtor：+104=0，+4=0，+8=3
0x69C730 | 【L1】图片框 SetTexture 变体：a2!=0 && a3!=-1 时解析句柄（同 69C6C0）
0x69C780 | 【L1】按钮 ctor（类型 1）：+8=1，+104..+124 与 +128/+129 清零
0x69C7B0 | 【L1】按钮 SetTextures：解析 常态(+104)/悬停(+112)/按下(+108)/禁用(+116) 四纹理句柄，
          a7=0 用默认 dword_9D3BD0、a8=0 用 dword_9D3BD4；宽高取常态句柄 +36/+40
0x69C850 | 【L1】按钮 dtor：与 69C780 对应的全字段复位
0x69C880 | 【L1】按钮 Render：三态绘制 —— 激活(69A4D0 命中)→悬停图(+108)，
          焦点态(+129 未画过)补 sub_414530；非命中→常态图(+104)；
          禁用分支画 +116；+128/+129 一次性触发位
0x69C980 | 【L1】按钮 SetTextures 变体：四个 -1 哨兵跳过解析（保留旧纹理），a7/a8 直接赋值
0x69CA20 | 【L1】滑条 ctor（类型 5）：+8=5，+120 double=0.0（滑块比例），+132=1
0x69CA50 | 【L1】滑条 SetTexture：a3!=-1 解析滑块纹理(+104)，a4!=-1 解析轨道(+108)
0x69CAB0 | 【L1】滑条 dtor：与 69CA20 对应复位
0x69CAF0 | 【L1】滑条 SetScrollPos：clamp [0, +29 最大值] 后写 +28
0x69CB10 | 【L1】滑条 SetTextures 变体：解析 +26 滑块/+27 轨道，宽高 +7/+8 槽，
          尾调虚槽+40 刷新(+29)
0x69CB80 | 【L1】滑条 SetRange：a2 上限；+116=a2，比例 +120 = (+32 值域)/a2 - 滑块(+40)/a2
          （double 几何换算，轨道像素映射）
0x69CC30 | 【L1】滑条 Update：虚槽+28 基类更新 → 可见(+17) && +132 时虚槽+36 绘制
0x69CC60 | 【L1】滑条 Render：滑块图(+104) 于 +20/+24；轨道图(+108) 于
          +24 + (+112 范围 × +120 比例 + 0.5 取整)（vt+152）
0x69CCE0 | 【L1】滑条 ThumbDrag：dword_79BE40 命中/拖动判定，
          pt.y 反算比例 → sub_69CAF0(pos)（(pt.y - 轨道中心 - +24)/+120）
0x69CD70 | 【L1】组合控件 Create：+104=a2 主图/+108=a3 副图/+112=a4 组对象，
          高度 = 主+32 + a4[8] + 副+32；sub_69A5F0 定位两个子项，
          sub_69A840 依次 push 主/副/组三项
0x69CE10 | 【L1】标签页 ctor（类型 7）：+8=7，+104=-1（无选中），+108=0
0x69CE30 | 【L1】标签页 dtor：与 69CE10 对应复位
0x69CE50 | 【L1】标签页 SelectTab(a2)：遍历子链(+96)给每项 +132 写选中布尔（==a2 判定），
          +104=a2，+108=1
0x69CEB0 | 【L1】标签页 DeselectAll：子链全部 +132=0，+104=-1
0x69CEF0 | 【L1】标签页::dword_79BE58 联动：全局选择 id 命中本页时清 dword_79BE58，
          按 dword_79BE64 查找对应子项并 SelectTab
0x69CF50 | 【L1】标签页 Select 包装：+4=1 后 sub_69CE50(a2)
0x69CF80 | 【L1】图片组 ctor（类型 6）：+8=6，+104..+132 全清
0x69CFC0 | 【L1】图片组 dtor：与 69CF80 对应复位
0x69D000 | 【L1】图片组::切换态翻转：dword_79BE64==-1 && dword_79BE34==id 时 +132 取反；
          尾调 Concurrency::SpinYield（CRT 符号混入的基类虚槽调用，按原样保留）
0x69D02B | 【L1】Concurrency::details::ExternalContextBase::SpinYield（虚槽+28 转发；CRT 符号混入）
0x69D030 | 【L1】标签(类型 2) Render：可见(+16)时 —— 激活(+132 翻转态)画 +108；
          悬停(69A4D0)画 +112 且 +120 一次性触发 sub_414530(+129 位)；
          常态画 +104；不可见交互时画 +116
0x69D150 | 【L1】文本列表 Create：+8=a2 类型/+40 行缓冲尺寸 a3(new a3+1)/+44 行对象尺寸 a4
          （new a4）；行指针数组 memset 清零；失败→0x80000008
0x69D210 | 【L1】文本列表 Draw（GDI 路径）：+68 起始行遍历到 +68+72，
          每行按 +92 对齐模式(1 居中/2 右对齐)算 x 偏移；
          有字体(+56)走渲染器 sub_416430；否则 GDI TextOutA 8 方向描边
          （+88 阴影色/+84 高光色/+0 主色，+8/+9/+10 描边开关位）
0x69D4E0 | 【L1】文本列表 ClearAll：逐行 free(行对象+24 文本/+6 槽) + free 数组 + 字段复位
0x69D550 | 【L1】文本列表 Destroy：先 ClearAll 语义再释放 +48/+52 缓冲与全部字段（+4=0）
0x69D640 | 【L1】文本列表 ClearItems：行数组区间 free + 数组指针/容量清零（保留配置字段）
0x69D6C0 | 【L1】文本列表 DeleteItems(n)：-1 全清(69D640)；否则删前 n 行并 memmove 压缩数组
0x69D780 | 【L1】文本列表 Flush：+28 就绪时 渲染器 vt+72 Begin → vt+96 取 HDC →
          sub_69D210 逐行画 → vt+76 End
0x69D7D0 | 【L1】std::vector<T*>::insert 中段（扩容/搬移三路分支，0x3FFFFFFF 哨兵）
0x69D9B0 | 【L1】std::vector<T*>::push_back：容量足直接尾插，否则转 69D7D0 扩容插入
0x69DA00 | 【L1】文本列表 AddLine：new(0x20) 行对象（继承 +80 颜色/+60/61/62 标志/+84/+88，
          +12 行号 a3/+28 a4），strncpy 文本，push_back 入 +12 行 vector
0x69DB10 | 【L1】文本列表 AddExtra：超长(+40)拒绝打日志；按 CR/LF(13/10) 拆行，
          SJIS 双字节不拆，行宽经字体引擎 vt+24 适配截断，逐行 sub_69DA00 入列；返回行数
0x69DDB0 | 【L1】文本列表 AddLinef：vsprintf 到 +52 缓冲 → sub_69DB10 格式化追加行
0x69DDF0 | 【L1】标签(类型 2) ctor：+8=2，子槽(+26)=0
0x69DE00 | 【L1】标签 dtor：+104=0，+4=0，+8=2
0x69DE30 | 【L1】标签 Render：可见(+16)且 +104 图非空时 vt+152 绘制于 +20/+24
0x69DE60 | 【L1】CWSBuffer::Reset：+4..+20 五字段清零（vtable off_749D94 的构造尾巴）
0x69DE80 | 【L1】CWSBuffer::Allocate(Size)：重复分配→0x80000002；new(Size) 入 +8 基址，
          容量 +12=Size，memset 清零，+4=1；失败→0x80000000。ISocket::Init 调用两次。
0x69DEE0 | 【L1】CWSBuffer::Free：free(+8) 基址，五字段清零
0x69DF10 | 【L1】CWSBuffer::Write(buf,len)（入队）：+4 未就绪→0x80000001；
          写指针(+20)+len > 容量(+12) → 0x80000000（环满）；
          qmemcpy 到 基址+8+写偏移，+20 += len。上层: sub_698C80 包头入队。
0x69DF70 | 【L1】CWSBuffer::Read(len)（出队）：+4 未就绪→0；
          读(+16)+len > 写(+20) → 越界：fopen("SocketLog.log","a") 写
          "* Devector Error" 四行现场（读/len/写）后返回 0；
          正常：读指针 += len，读完(+16==+20)归零对，返回 基址+8+旧读偏移。
0x69E010 | 【L1】ijl15 JPEG 编码（截屏 JPG 路径）：ijlInit(69762A) 失败 → MessageBox "ijlInit..."；
          填 JPEG_CORE_PROPERTIES(维度/通道 3或4/quality=a6=100 调用点) →
          ijlWrite(第9号调用) 失败 → "ijlWrite..."；ijlFree 后返回压缩缓冲与 *a7=大小。
0x69E1A8 | 【L1】导入 thunk → __imp__FSOUND_SetDriver@4（FMOD 声音库）
0x69E1BA | 【L1】导入 thunk → __imp__FSOUND_SetVolume@8
0x69E1C6 | 【L1】导入 thunk → __imp__FSOUND_Stream_SetEndCallback@12
0x69E1D8 | 【L1】导入 thunk → __imp__FSOUND_Stream_Play@8
0x69E1EA | 【L1】导入 thunk → __imp__FSOUND_Stream_Open@16
0x69E1F6 | 【L1】导入 thunk → __imp_RtlUnwind（SEH 展开链）
0x69E1FC | 【L1】CRT _filelength：句柄表锁内 _lseek_lk(END) 取文件长，失败 errno=9(EBADF)
0x69E2D0 | 【L1】CRT _fileno：FILE->_file
0x69E2D8 | 【L1】CRT longjmp：FS:0 帧校验 → _global_unwind2/_local_unwind2 →
          1447244336=0x56433230 法线校验 → _NLG_Notify → 跳转
0x69E350 | 【L1】空桩 nullsub_14（CRT 对齐）
0x69E360 | 【L1】CRT _pow（反编译失败计划内替代 → math.h pow；610B 大函数）
0x69E5C2 | 【L1】CRT x87 TOS 整数判定（rint 等价，dbl_79BE70=2^63 缩放）
0x69E5E6 | 【L1】CRT x87 空 helper（odd）
0x69E5F0 | 【L1】CRT _CIacos：x87 寄存器版 acos（_checkTOS_withFB 前置）
0x69E604 | 【L1】CRT acos（cdecl 入口 → start_1 主体）
0x69E60D | 【L1】CRT acos/asin 主体（反编译失败计划内替代；链接器/CRT 桩）
0x69E6BB | 【L1】CRT _finite：指数位 != 0x7FF0 即有限
0x69E6D0 | 【L1】CRT _fpclass：IEEE754 分类（512/4/2/1/16/128/32/64/8/256 全谱）
0x69E76C | 【L1】CRT _ftol：x87→int64（原型 usercall st0）
0x69E7A0 | 【L1】CRT floor：SSE 快路径（MXCSR 0x1F80 检测）+ x87 慢路径（计划内替代 floor）
0x69E8D0 | 【L1】CRT _aullshr：uint64 逻辑右移（cl 计数）
0x69E8EF | 【L1】CRT wcslen
0x69E905 | 【L1】CRT _rt_probe_read4：4 字节安全读探测（SEH frame 探针）
0x69E950 | 【L1】CRT _CIpow_pentium4：x87 寄存器版 pow（P4 分派入口）
0x69E969 | 【L1】CRT _pow_pentium4：pow 主体（exp2(y*log2(x)) 管线，2.2KB 大函数）
0x69F5FE | 【L1】CRT pow 内部：expm1 分支（正 SP 偏移警告，Hex-Rays 计划内噪声）
0x69F641 | 【L1】CRT x87 isintTOS 判定
0x69F666 | 【L1】CRT x87 isintTOS 返回桩
0x69F675 | 【L1】CRT powhlp 调度入口（usepowhlp：2^x 快路径选择）
0x69F880 | 【L1】CRT 双参数错误处理入口（_startTwoArgErrorHandling）
0x69F897 | 【L1】CRT 单参数错误处理入口（_startOneArgErrorHandling）
0x69F8E0 | 【L1】CRT x87 2^TOS（_twoToTOS）
0x69F8F5 | 【L1】CRT x87 控制字加载（_load_CW）
0x69F90C | 【L1】CRT TOS→QNaN 转换（_convertTOStoQNaN）
0x69F925 | 【L1】CRT _fload_withFB：带 FB 状态的浮点加载
0x69F968 | 【L1】CRT _checkTOS_withFB：TOS 合法性检查
0x69F9B5 | 【L1】CRT 溢出退出桩（_check_overflow_exit）
0x69F9C9 | 【L1】CRT 范围退出桩（反编译失败计划内替代）
0x69FA6C | 【L1】CRT _d_inttype：double→整型分类
0x69FADA | 【L1】CRT _powhlp：pow 核心助手（参数归一/特殊值分流，1.1KB）
0x69FC3C | 【L1】CRT _floor_default：floor 通用路径
0x69FD10 | 【L1】CRT 运行时未知桩（Microsoft VisualC 2-14/net runtime 签名）
0x69FE27 | 【L1】CRT 运行时未知桩（同上）\
"""


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


def main():
    ref = Refiner(PATH)
    ref.set_header(HEADER.split("\n"))
    ref.build(parse_sections(SECTIONS), parse_annots(ANNOTS))
    assert ref.verify_lossless(), "lossless verification failed (剥除插入行 != L2 原文)"
    ref.save(PATH)
    n_va = Refiner.count_vas(PATH)
    ob, cb = Refiner.braces(PATH)
    print("OK socket_ime.cpp: sections=%d annots=%d va=%d braces=%d/%d" % (
        9, len(parse_annots(ANNOTS)), n_va, ob, cb))
    assert n_va == 269, n_va
    assert ob == cb, (ob, cb)


# ---------- 恢复模式：把已精修文件无损还原为 L2 原文（drop-set 剥除） ----------
ORIG_HEADER = [
    "// ============================================================",
    "// 【40JP逆向源代码工程】platform/socket_ime.cpp",
    "// 模块: ISocket/IME/NCEdit",
    "// 来源: work/dump_tail.txt  区块: 0x690000-0x6A0000",
    "// 层级: L2 转写层 —— 反编译输出的规范化重排(类型名清洗/VA标记/分模块),",
    "//        语义与 IDA Hex-Rays 逐字对应; 精修层(L1)见 src/nengine|cnrd 与 docs/10-15",
    "// 函数数: 269",
    "// ============================================================",
]
L2_TAIL_COMMENTS = (
    "// attributes: thunk",
    "// (decomp failed",
    "// [计划内]",
    "// 替代:",
    "// Microsoft VisualC 2-14/net runtime",
    "// positive sp value has been detected",
)

def recover():
    raw = open(PATH, encoding="utf-8", newline="").read()
    nl = chr(13) + chr(10) if chr(13) + chr(10) in raw else chr(10)
    lines = raw.split(nl)
    starts = [i for i, ln in enumerate(lines)
              if ln == BANNER and i + 1 < len(lines) and VA_RE.match(lines[i + 1])]
    s0 = starts[0]
    inc = next(i for i in range(s0) if lines[i].startswith("#include"))
    assert lines[inc - 1] == "", "header layout unexpected"
    drops = set()
    # 1) SECTION 横幅块（L2 原文正文无 "// ===" 行）：连续 // 注释块，尾行必为 "// ==="
    i = inc + 1
    while i < len(lines):
        if lines[i].startswith("// ==="):
            j = i + 1
            while j < len(lines) and not lines[j].startswith("// ==="):
                assert lines[j].startswith("// "), i
                j += 1
            assert j > i + 1 and j < len(lines), i
            drops.add(i - 1)            # 插入时前置的空行
            for k in range(i, j + 1):
                drops.add(k)
            i = j + 1
        else:
            i += 1
    # 2) 注释块：每个函数第 3 行横幅后的 "// 【L1】" 连续注释行
    for st in starts:
        j = st + 3
        if j < len(lines) and lines[j].startswith("// 【L1】"):
            while j < len(lines) and lines[j].startswith("// "):
                if any(lines[j].startswith(t) for t in L2_TAIL_COMMENTS):
                    break
                drops.add(j)
                j += 1
    # 3) 头部替换 + 正文剥除
    out = list(ORIG_HEADER) + ["", lines[inc], ""]
    for i in range(inc + 1, len(lines)):
        if i not in drops:
            out.append(lines[i])
    txt = nl.join(out)
    n_va = len(re.findall(r"(?m)^// 0x[0-9A-Fa-f]{6}", txt))
    assert n_va == 269, n_va
    assert "【L1】" not in txt
    body = txt[txt.find("#include"):]
    assert body.count("// ============================================================") == 0
    open(PATH, "w", encoding="utf-8", newline="").write(txt)
    print("recovered to L2 original: va=%d lines=%d" % (n_va, len(out)))

if __name__ == "__main__":
    import sys as _s
    if len(_s.argv) > 1 and _s.argv[1] == "recover":
        recover()
    else:
        main()
