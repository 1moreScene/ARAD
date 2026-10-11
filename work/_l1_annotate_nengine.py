# -*- coding: utf-8 -*-
"""
L1 精修注入器 —— src/nengine/ 四文件（0x40 日志转储 / 0x41 图像包 / 0x65 zlib浏览器 / 0x66 声音脚本尾）
只加注释（头替换/分区横幅/标记行尾注/【解读】块），不改任何代码行。
机制与 work/_l1_annotate_rtti_core.py（rtti_core 精修先例）一致。
"""
import re, io, sys
sys.stdout.reconfigure(encoding='utf-8')

BASE = r'E:\DOF LX\【40JP逆向源代码工程】'
DASH = '// ------------------------------------------------------------'
BAR  = '// ============================================================'

# =====================================================================
# 各文件配置
# =====================================================================

# ---------------------------------------------------------------- log_dump
LOG_SECTIONS = [
 (0x401000, '基础设施：nengine::IObject 构造/析构桩 + MSVC std::string(CString) 模板实例族'),
 (0x401B20, '程序入口 WinMain 与全局初始化链（崩溃回调注册 / 随机播种 / 子系统聚合初始化）'),
 (0x401F80, '★★ 崩溃报告链：旧转储清理 → 命名 → dmp 占位 → 7z 打包 DNF_DUMP.zip → base64 → NIS::Http 上报'),
 (0x402A50, '字符串参数包 / CString 数组 / vsprintf 格式化打包工具'),
 (0x4031E0, '数学库与引擎子系统初始化（sin/cos/tan 表、rand 播种、随机数工具）'),
 (0x4038D0, '日志核心：NiLog 对象（QPC 计时日志 / 横幅 / 5000 行轮转 / OutputDebugString）'),
 (0x4042E0, 'Win32 窗口框架：主窗口过程 / 消息循环 / 单实例检测 / 全屏分辨率切换'),
 (0x4049C0, '插值与通用辅助（lerp 源头 / 字符串安全拷贝 / rdtsc 高精度时钟族）'),
 (0x405C20, '窗口类注册与创建 / GDI 图标位图工具'),
 (0x405E00, '引擎子系统事件分发框架（list/vector 容器事件路由，0x405E00-0x406C50）'),
 (0x406C50, 'NiMemory.log / NiProfile.log 日志对象（Gamebryo 风格内存/性能日志）'),
 (0x406D90, 'IME 输入法处理族（Composition/Candidate/NotifyIME，日志直证）'),
 (0x407AE0, '子系统注册容器族（树/链表基础设施，为 0x41 区等上层共用）'),
 (0x408B90, 'GDI 位图工具 + NEOPLE_WEB 内嵌浏览器窗口抓屏（BitBlt 采样）'),
 (0x409AE0, 'D3D8 应用初始化：设备/模式枚举（qsort）、D3DAPPERR 错误分类、REF 设备回退警告'),
 (0x40A800, 'DirectSound8 声音子系统（DirectSoundCreate8 / 协作级 / 缓冲管理）'),
 (0x40B140, 'DirectInput8 输入子系统（DirectInput8Create / 键盘设备容器）'),
 (0x40B990, '文本测量（GetTextExtentPoint32）/ GDI 文本渲染 / 容器工具'),
 (0x40C4C0, 'STL map/set 红黑树模板实例 + 字体位图缓存（invalid map/set iterator 诊断）'),
 (0x40D020, '调试控制台（_DEBUG：AllocConsole / MS UI Gothic / DIB 行缓冲 / FreeConsole）'),
 (0x40D660, '大小写归一文件查找 + 文件名红黑树（NPK 内索引的磁盘侧支撑）'),
 (0x40E9E0, '★★ NPK 资源包加载器（NeoplePack_Bill 魔数 / *.npk 目录扫描 / 包内文件定位）'),
 (0x40F2F0, '矩形/绘制辅助与杂项（SetRect 族、sprite 小工具）'),
]

LOG_NOTES = {}
LOG_NOTES['0x401000'] = [
 'nengine::IObject 构造桩：写 IObject vftable + this+4 清零（引用计数/标志头）。',
 '全引擎对象的最基础构造，0x41/0x65 区大量对象的第一步。',
]
LOG_NOTES['0x401110'] = [
 '★★ 崩溃回调本体（WinMain 中 RegisterExceptionCallback(0x401110) 注册，参数=异常信息串）:',
 '  ① sub_404020()  冲刷日志 fflush；',
 '  ② sub_403A90()  NiLog/QPC 日志对象收尾；',
 '  ③ dword_79C0C0（主管理器）非 0 时 sub_402080(管理器+113) 取角色名 → 设为报告名；',
 '  ④ sub_4020B0(Format) 写 dmp 占位文件（异常信息串为内容）；',
 '  ⑤ sub_402440(&unk_79C0C8) 7z 打包 + HTTP 上报（上下文对象 = WinMain 初始化的 79C0C8）。',
 '完整链见文件头“崩溃报告流程图”。',
]
LOG_NOTES['0x401B20'] = [
 '★ WinMain 入口:',
 '  sub_402170(0, hInstance, off_77F284[0], off_77F288[0]) —— 崩溃上报上下文初始化',
 '    （off_77F284/288 数据对 = 上报目标串【推断：URL/主机名】）；',
 '  RegisterExceptionCallback(sub_401110) —— 崩溃报告链入口；',
 '  timeGetTime 播种 srand + malloc(rand()%100000) 哨兵分配；',
 '  初始化链: sub_403340（数学表+声音+日志对象）→ sub_401090（NIS::Http 等）→',
 '    sub_403400 → sub_4037C0（主管理器 dword_79C0A0/C0C0）；',
 '  sub_4048F0 = 单实例检测（FindWindowA 命中 → SetForegroundWindow 唤醒旧实例）；',
 '  sub_404360 = 主消息循环；退出时 sub_4010E0 清理。',
]
LOG_NOTES['0x401DA0'] = ['base64 编码器（127 行，标准字母表；由 0x4020F0 计算输出缓冲长度后调用）。']
LOG_NOTES['0x401F80'] = [
 '旧转储清理：FindFirstFileA 循环 DeleteFileA（跳过 "."/".." 与目录位 0x10）。',
 '0x402170 对 "*.dmp" / "*.trc" / "*.zip" 各调一次 —— 启动时清空上次残留。',
]
LOG_NOTES['0x402030'] = [
 '转储文件名三元组生成:',
 '  this+386 ← "%s_%s.dmp"；this+646 ← "%s_%s.trc"；this+906 ← "DNF_DUMP.zip"；',
 '  前缀 = 角色名(this+75) + "_" + %08x timeGetTime 实例号(this+24)。',
]
LOG_NOTES['0x402080'] = ['设置崩溃报告角色名（strncpy 0x32=50 字节 → this+75）并重生成文件名三元组。崩溃回调 ③ 调用。']
LOG_NOTES['0x4020B0'] = ['写 dmp 占位文件（fopen(this+386,"w") + fprintf + fclose），内容 = 异常回调入参串。崩溃回调 ④ 调用。']
LOG_NOTES['0x4020F0'] = [
 'base64 输出长度计算: len = 4n/3 + n/3；再按 76 列折行补齐（+4-(len%76%4)）',
 '并加 CRLF×2/76 行与尾 2 字节 → sub_401DA0(数据, n, 输出, &len, 0)。',
]
LOG_NOTES['0x402170'] = [
 '★ 崩溃上报上下文初始化（WinMain 首个调用）:',
 '  this+0/+4 ← a2/a3（回调登记）；上报目标串 a5 → this+126；',
 '  timeGetTime → "%08x" 实例号 → this+24；角色名占位 "_NONE" → this+75；',
 '  sub_402030 生成文件名三元组；随后 sub_401F80 清理旧 *.dmp/*.trc/*.zip。',
]
LOG_NOTES['0x402440'] = [
 '★★ 崩溃打包上报本体（崩溃报告链第 5 步）:',
 '  ① fopen("DNF.trc","rb") → 读全量 → 复制为 %s_%s.trc（this+646）；',
 '  ② sprintf(Buffer, "a -tzip %s %s %s", zip, dmp, trc) →',
 '     ShellExecuteExA("7z", "open", ".\\", fMask=320 SEE_MASK_NOCLOSEPROCESS) ——',
 '     ★ 调用系统 7-Zip 命令行把 dmp+trc 打包成 DNF_DUMP.zip；',
 '  ③ GetExitCodeProcess 轮询 259(STILL_ACTIVE) + Sleep(0x3E8=1000ms) 等待打包完成；',
 '  ④ 读 zip 全量 → base64 编码（0x4020F0/0x401DA0）→ 拼 " content=" 前缀 →',
 '     sub_4028C0 HTTP POST 上报（上报目标 = this+126）。',
]
LOG_NOTES['0x402780'] = [
 'WinInet 会话构造: InternetOpenA("NIS::Http", 0/PRECONFIG, 0,0,0)。',
 '"NIS::Http" = 上报代理标识（NIS = Neople Internet Service【推断】）。',
]
LOG_NOTES['0x4028C0'] = [
 '★ HTTP POST 上报执行器:',
 '  拆分 server/object（sub_4027F0 memchr/memchr 定位）→ InternetConnectA →',
 '  HttpOpenRequestA("POST") → 头 "Content-Type: application/x-www-form-urlencoded" →',
 '  HttpSendRequestA（body = base64 后的 DNF_DUMP.zip）→ InternetCloseHandle。',
]
LOG_NOTES['0x4027F0'] = ['URL 子串定位（memchr 首字符 + memcmp 全串推进）—— HTTP server/object 分隔查找。']
LOG_NOTES['0x403D60'] = [
 'NiLog 文件写入器（核心）:',
 '  行计数(+71) 满 5000 行（-4999 < 0 / OFSUB 5000）时 fclose+fopen 重开 —— 日志轮转；',
 '  横幅 "/**...**\\n * %s - Log File Opened\\n *     %02d/%02d/%04d - %2d:%02d:%02d"；',
 '  byte_79C7AC == 1 时同步镜像到控制台（sub_406CD0）；',
 '  入参超过 512 字节截断到 0x1FC+3（防溢出）。',
]
LOG_NOTES['0x404050'] = ['日志关闭横幅（Log File Closed + GetLocalTime + 5 空行）→ fclose。']
LOG_NOTES['0x404130'] = ['日志文件打开（fopen + strcpy 文件名 + Log File Opened 横幅）。']
LOG_NOTES['0x404360'] = ['主消息循环: ShowWindow/UpdateWindow/SetFocus → PeekMessage(帧驱动)+GetMessage 双泵 → TranslateMessage/DispatchMessage；每帧 sub_4052D0(timeGetTime 帧计时)。']
LOG_NOTES['0x404620'] = [
 '主窗口过程 WndProc: DefWindowProcA 兜底分发；sub_406D90/sub_407900 = IME 主处理；',
 'PostQuitMessage 退出；GetCurrentProcess+SetPriorityClass —— 状态切换时调优先级【推断：挂起降级】。',
]
LOG_NOTES['0x4048F0'] = ['单实例检测: FindWindowA(类名) 命中 → SetForegroundWindow 唤醒旧实例 + sub_402AE0 通知后退出。']
LOG_NOTES['0x4049C0'] = ['区间线性插值 lerp（与 src/nengine/camera_fx.cpp 引用的 0x4049C0 同址 —— lerp_f 源头）。']
LOG_NOTES['0x404E70'] = ['rdtsc 高精度时钟封装（__rdtsc + HIDWORD 拼装 + Sleep 校准循环）—— NiProfile 计时底层。']
LOG_NOTES['0x405950'] = ['三角函数表构建（sin/cos/tan 查表初始化，172 行）—— sub_403340 初始化链一环。']
LOG_NOTES['0x405C20'] = ['窗口类注册+主窗口创建: LoadIconA/RegisterClassExA/SetRect/AdjustWindowRect/LoadMenuA/CreateWindowExA/SetWindowTextA。']
LOG_NOTES['0x4090D0'] = [
 '★ NEOPLE_WEB 窗口抓屏: GetWindowTextA 比对 "NEOPLE_WEB"（0x652D30 内嵌浏览器窗口）→',
 '  GetWindowRect/IntersectRect 裁剪 → GetWindowDC/CreateCompatibleDC/BitBlt 拷贝浏览器画面 →',
 '  GDI 位图供纹理上传（sub_408CF0）。内嵌网页显示到游戏画面的通路。',
]
LOG_NOTES['0x409AE0'] = [
 'D3D 应用设备/模式枚举（387 行）: qsort(sub_409870) 排序显示模式表；',
 'D3DAPPERR_NOHARDWAREDEVICE/NOWINDOWEDHAL/NOHALTHISMODE 等错误分类 →',
 'MSGWARN_SWITCHEDTOREF 警告（退回 REF 软件渲染时的提示路径）。',
]
LOG_NOTES['0x40A130'] = ['Direct3DCreate8 初始化（158 行）: 设备枚举 + 模式选择 + SetWindowPos 应用。']
LOG_NOTES['0x40AA20'] = ['DirectSoundCreate8 + SetCooperativeLevel —— 声音子系统构造（缓冲对象 sub_40A930 族）。']
LOG_NOTES['0x40B140'] = ['DirectInput8Create —— 输入子系统构造（设备容器 0x40B210/0x40B990）。']
LOG_NOTES['0x40BE20'] = [
 '_DEBUG 控制台单行输出: _access 探测控制台状态 + GetStdHandle/WriteConsoleA。',
 '全引擎调试打印的最终出口（0x655A90/0x657120 等都转发到这里）。',
]
LOG_NOTES['0x40C4C0'] = ['std::map 红黑树模板实例（228 行）: "invalid map/set<T> iterator" 抛出 + 插入/重平衡 —— 调试控制台文本表的容器。']
LOG_NOTES['0x40C810'] = ['std::map 红黑树插入（114 行）: "map/set<T> too long" + sub_40C430 结点分配 —— MSVC _Tree::_Insert。']
LOG_NOTES['0x40D510'] = [
 '★ 调试控制台初始化（_DEBUG）: AllocConsole + "\\n<Start Console Log>...\\n\\n" +',
 '  sub_40BE20 输出 + CreateFontA("MS UI Gothic") + sub_40BF80 CreateDIBSection 行缓冲。',
]
LOG_NOTES['0x40E360'] = ['包内文件打开（磁盘态）: sub_40DD60 查找 + fopen。被 gfx_texture 0x418CD0（IMG 打开）复用。']
LOG_NOTES['0x40E3E0'] = ['包内文件打开（NPK 态）: sub_40DED0 红黑树定位 + fopen(NPK) + _fileno/_filelength —— 返回包流 FILE*。']
LOG_NOTES['0x40E9E0'] = [
 '★ NPK 包打开: 读包头 strncmp("NeoplePack_Bill") 魔数校验 → 包索引构建',
 '  （sub_40E700 红黑树，文件名 tolower 归一 sub_40D660）。Neople 自有封包格式。',
]
LOG_NOTES['0x40EF70'] = [
 '★ *.npk 扫描装载: GetCurrentDirectoryA/SetCurrentDirectoryA + _findfirst/_findnext',
 '  遍历 "*.npk" → 逐包 sub_40E9E0 打开 → 挂入包管理器（0x40EEF0 链表）。',
]
LOG_NOTES['0x40DD60'] = ['NPK 内文件查找（磁盘名→索引）: tolower 归一 + sub_40DC70 红黑树匹配 + fseek 定位。']
LOG_NOTES['0x40DED0'] = ['NPK 内文件查找（包态，返回文件长度）: 同上 + fopen 包流。']
LOG_NOTES['0x406C60'] = ['NiMemory.log 日志对象构造（Gamebryo 风格内存分配跟踪日志，基类 sub_403990）。']
LOG_NOTES['0x406CA0'] = ['NiProfile.log 日志对象构造（性能采样日志）。']
LOG_NOTES['0x401830'] = ['std::length_error("vector<T> too long") 抛出器 —— MSVC vector::reserve 模板实例的溢出路径。']
LOG_NOTES['0x4067C0'] = ['std::length_error("list<T> too long") 抛出器（list 模板实例）。']
LOG_NOTES['0x406ED0'] = ['std::length_error("deque<T> too long") 抛出器（deque 模板实例）。']

# ---------------------------------------------------------------- gfx_texture
GFX_SECTIONS = [
 (0x410000, 'CND3DBaseTexture 绘制族：NiProfile 包裹 + IntersectRect 裁剪 + 浮点变换的块绘制（0x410030/0x410140/0x410300/0x4105A0 变体）'),
 (0x410750, '纹理↔ImagePackEx 绑定（SetImagePackEx）+ 块传输族'),
 (0x4111E0, '屏幕矩形/视口辅助'),
 (0x4116A0, '★ IMG 图像块解码族（0x4116A0/0x411B30/0x412150/0x412700 —— 均调 0x652E00 zlib uncompress：IMG 内嵌图像数据为 zlib 压缩流）'),
 (0x412AE0, '图像属性 getter/setter（宽高/矩形/标志，HIWORD/LOWORD 打包）'),
 (0x413B80, '图像对象构造/克隆（SetRect + 容器分配）'),
 (0x413EE0, '窗口尺寸调整族（GetWindowLong/GetSystemMetrics/AdjustWindowRectEx/SetWindowPos）'),
 (0x414200, '引擎日志格式化转发（vsprintf → 0x404B30 缓冲）'),
 (0x4143A0, '对象销毁/容器壳'),
 (0x414570, '声音对象/WAV 缓冲族（0x4149E0 mmio WAV→DirectSound 装载）'),
 (0x414D60, 'COM/宽字符工具（CoCreateInstance / MultiByteToWideChar）'),
 (0x415020, '键盘输入族（GetAsyncKeyState 轮询）'),
 (0x415B10, '输入映射 vector 管理（按键状态容器）'),
 (0x4164C0, 'string/vector 容器工具'),
 (0x416890, '字形/文本对象容器（大函数，字体缓存管理）'),
 (0x416CD0, 'GDI 资源清理（DeleteObject 族）'),
 (0x416F40, '★ NeopleFontFile 自有字体文件校验（fopen + 魔数 "NeopleFontFile"）'),
 (0x4176E0, '★ Arial Narrow GDI 位图字体构建（CreateFontA + CreateDIBSection + TextOutA 栅格化缓存）'),
 (0x4185B0, 'GDI 文本渲染三件套（DIB/TextOut/宽度表，0x4185B0/0x4186C0/0x418AA0）'),
 (0x418CB0, '★★ CNImagePackEx：IMG 打开/魔数校验/延迟加载/别名/销毁 + 路径拼装'),
 (0x4199A0, 'mmio WAV 读取族（mmioOpenA/Descend/Read/Ascend → 声音缓冲）'),
 (0x419D40, '路径拼装（"%s%c%s%c%s"）/ 图像项容器 / 时间节流'),
 (0x41A7C0, 'STL 容器模板实例族（vector/list/deque 诊断与操作）'),
 (0x41C0C0, '应用状态/模块管理（场景查询、退出、事件分发、消息发送）'),
 (0x41CC80, '★★ onChangeModule 模块切换总控（UDP STOP/START 广播、LADDER_DOWN、STOP SOUND、MAP_APPEAR）'),
 (0x41D5A0, '模块切换辅助（map 容器/观察者清理/资源卸载链）'),
 (0x41E4F0, '服务器连接 UI 状态（"Connect to Server..."/"Connection Failed"）'),
 (0x41F2C0, '★ 界面 IMG 预载清单（Sprite/Interface/*.img 八件）+ DllMain'),
 (0x41F930, '引擎聚合初始化/事件路由/杂项'),
]

GFX_NOTES = {}
GFX_NOTES['0x410000'] = ['纹理设备丢失复位: this+44 就绪标志检查 → 设备 vt+68(0) 复位 → 标志清 0；失败返回 0x80000005（E_FAIL 族）。']
GFX_NOTES['0x410030'] = [
 '纹理块绘制入口（userpurge）: sub_404F40(a1,4) = NiProfile 计时作用域开启 →',
 '  源矩形平移（a3+a2[7]）→ IntersectRect 裁剪 → 浮点坐标变换 → sub_404F80 计时结束。',
]
GFX_NOTES['0x410750'] = [
 'SetImagePackEx: 把 CNImagePackEx(+索引) 绑到 CND3DBaseTexture（this+32/33）；',
 '重复绑定报 "CND3DBaseTexture : ImagePackEx already exists"（0x80000005）。',
 '0x419550（PackEx::Load）对每个新图像调用 —— 纹理回指图像包的通路。',
]
GFX_NOTES['0x4116A0'] = [
 '★ IMG 图像块解码（202 行）: 读块头（sub_66BD5A fread）→',
 '  sub_652E00 = zlib uncompress（★ IMG 内嵌图像数据是 zlib 压缩流，zlib_inflate.cpp 0x65 区）→',
 '  解码结果建图像对象（new + sub_652E00 失败路径 j_j__free）。',
]
GFX_NOTES['0x411B30'] = ['★ 同族解码变体（279 行，同样调 0x652E00 uncompress）—— 不同位深/格式分支【推断】。']
GFX_NOTES['0x412150'] = ['★ 同族解码变体（232 行，调 0x652E00 uncompress）。']
GFX_NOTES['0x412700'] = ['★ 同族解码变体（282 行，调 0x652E00 uncompress + BYTE2 分支）。']
GFX_NOTES['0x413EE0'] = ['主窗口尺寸调整（客户区→窗口矩形）: GetWindowLongA 风格 + GetSystemMetrics + AdjustWindowRectEx + SetWindowPos。0x414090 为带边框变体。']
GFX_NOTES['0x414200'] = ['引擎格式化日志转发: vsprintf → sub_404B30（安全拷贝）→ sub_404E30 —— off_77F3FC 日志指针的落地实现之一【推断】。']
GFX_NOTES['0x4149E0'] = [
 'WAV→声音对象装载: sub_4199A0(mmioOpenA/Descend/Read 全量) + sub_419B40(Seek/Descend fmt 块)',
 ' + sub_419B80(mmioGetInfo/Advance 数据流) → sub_414850/0x414940 建缓冲 → 挂声音管理器。',
]
GFX_NOTES['0x414D60'] = ['CoCreateInstance —— COM 对象创建（0x414D40/0x414E00 伴随）【推断：声音/滤镜类组件】。']
GFX_NOTES['0x416F40'] = [
 'NeopleFontFile 校验（写向）: fopen + wsprintfA 生成字体文件名 → 写魔数 "NeopleFontFile" → fclose。',
 'Neople 自有字体容器格式（与 0x417030 读校验成对）。',
]
GFX_NOTES['0x417030'] = ['NeopleFontFile 读校验: fopen + sub_66BD5A 读头 + strcmp("NeopleFontFile") + sub_402AE0 登记文件名。']
GFX_NOTES['0x4176E0'] = [
 '★ Arial Narrow 位图字体构建器（253 行）: strcmp 选族 → CreateFontA → GetDC/CreateCompatibleDC',
 ' → CreateDIBSection 栅格缓存 → SetTextColor/SetBkMode + TextOutA 逐字符绘制 →',
 ' GetTextExtentPoint32A 宽度表。"Arial Narrow" 为默认 UI 字体。',
]
GFX_NOTES['0x417B60'] = ['Arial Narrow 字体族管理（199 行）: sub_402AE0 登记 + new 字体对象 + 复用 0x4176E0 构建器。']
GFX_NOTES['0x418CD0'] = [
 '★ IMG 文件打开+魔数校验:',
 '  this+100 == 0（包态）→ sub_40E3E0 从 NPK 包流打开；否则 fopen(磁盘态)；',
 '  读 0x20(32) 字节头 → strcmp "Neople Image File" —— IMG 格式魔数；',
 '  this+21(84) 非 0 = 已加载守卫；成功返回 FILE*（由 0x418E20/0x419550 续用）。',
]
GFX_NOTES['0x418DD0'] = ['按索引取图像对象: 12B 目录项（this+13 起 12*i）[+4] = 图像对象指针；越界/未建 → sub_419F00（全局占位图像）。']
GFX_NOTES['0x418E20'] = [
 '★ IMG 延迟加载驱动: 遍历 12B 目录矢量（this+52 起，12 字节/项）→',
 '  对未加载项（图像+4 == 0）且 this+88 == 1（允许流式）: fseek(FILE*, entry.offset) →',
 '  图像对象 vt+108(FILE*) 从包流装载；dword_77F488 控制逐个/跳过策略（v14 = a2+1 或 0x418CB0 计数）。',
]
GFX_NOTES['0x419070'] = ['图像目录矢量中段插入（MSVC vector 模板实例，12B 元素: 偏移/对象/标志；0x357913941 = 12B 元素容量上限）。']
GFX_NOTES['0x419320'] = ['CNImagePackEx::Destroy: off_77F3FC 日志（函数名自证字符串）→ sub_418FD0 释放目录/图像 → 名字串复位 → IObject vftable 复位。']
GFX_NOTES['0x419480'] = [
 'CNImagePackEx 构造: IObject 头 → off_6C2A64 vftable → 名字串(this+24, 容量15) →',
 '  目录矢量(this+52..60 三指针) → 标志组: +88 加载模式、+92/96 设备/管理器、',
 '  +84 独立文件标志(this+21)、+22(this+88?) 就绪位、this+100 磁盘态标志；+96(24) = a2 类型。',
]
GFX_NOTES['0x419550'] = [
 '★★ CNImagePackEx::Load —— IMG 装载主流程（132 行，函数名直证）:',
 '  ① sub_419D40 拼路径（"%s%c%s%c%s"，ImagePacks2 包根由 0x420190 决策）+ _access 探测；',
 '  ② sub_418CD0 打开 IMG（32B 魔数 "Neople Image File" 校验）；',
 '  ③ this+88 == 1 时逐目录项处理: 读 4B 记录头 ——',
 '     == 0x11(17) → 别名条目: 再读 4B 源索引，图像指针共享已建项，flag(+8)=1；',
 '     否则 → 设备 vt+8 CreateImage(ID) 建图 + sub_410750 SetImagePackEx 绑定 +',
 '            entry.offset = ftell（数据起点）→ 图像 vt+116(FILE*) 自描述装载',
 '            （返回消费字节数 → fseek(v15+offset) 定位下一记录）；',
 '  ④ 失败报 "CreateImage(%d) failed" / "SetImagePackEx(%d) failed" / "open_file failed %s"。',
 '  【推断】IMG 结构 = 32B 魔数 + 顺序记录流（4B 图像ID + 自定长 zlib 压缩块，见文件头）。',
]
GFX_NOTES['0x4198A0'] = ['Load 外壳: sub_419D40 拼完整路径 + sub_4015C0 登记 → 转 0x419550。']
GFX_NOTES['0x419D40'] = ['路径拼装器 "%s%c%s%c%s" / "%c%s%c%s"（根/分隔符/文件名/分隔符/扩展名，strncpy 截断保护）。']
GFX_NOTES['0x4199A0'] = ['mmio WAV 装载: mmioOpenA → mmioDescend(RIFF/WAVE) → mmioRead 全量 → GlobalAlloc 拷出 → mmioAscend/Close。']
GFX_NOTES['0x41A510'] = ['timeGetTime 缓存（+0x41A490 容器）—— 图像/资源延迟加载的时间节流。']
GFX_NOTES['0x41A630'] = ['随机源初始化: time 播种 srand + rand 工具（全引擎随机基元之一）。']
GFX_NOTES['0x41CC80'] = [
 '★★ onChangeModule 模块切换总控（252 行，函数名直证于 CALL 表）:',
 '  按目标模块 switch: MODULE_TYPE_PVP / WARROOM / MAIN_GAME||SELECT_DUNGEON →',
 '    "@@UDP STOP - onChangeModule(), MODULE_TYPE_XXX" 广播（语音/UDP 停止）；',
 '  SELECT_DUNGEON → "@@UDP START - onChangeModule(), MODULE_TYPE_SELECT_DUNGEON : seatIndex %d"；',
 '  case 4 → sub_432EA0("LADDER_DOWN")（天梯离场提示/音效）；',
 '  另见 "STOP SOUND" / "MAP_APPEAR" 关键串 —— 场景声音停播与地图出场事件。',
]
GFX_NOTES['0x41E4F0'] = [
 '服务器连接 UI 状态机: "Connect to Server..." → j__atol 端口解析 + sub_4231B0 连接发起 +',
 '  sub_698F90 超时时钟；失败路径 "Connection Failed"（off_77F3FC 日志）。',
]
GFX_NOTES['0x41F2C0'] = [
 '★ 界面 IMG 预载清单（new + sub_41F190/0x6990D0/0x6992C0/0x699100 挂管理器）:',
 '  Sprite/Interface/{SelectCharacter, CreateCharacter, CharacterIllust, Base,',
 '  ItemInventoryWindow, AvatarInventoryWindow, ItemTradeWindow, EquipmentShopWindow}.img',
 '  —— 角色选择/创建/仓库/交易/商店八大界面包。',
]
GFX_NOTES['0x41F930'] = ['DllMain 桩（DLL_PROCESS_ATTACH 无附加处理）。']
GFX_NOTES['0x41FAA0'] = [
 '引擎事件路由聚合构造（83 行）: new 事件总线 + 注册 8 个回调',
 '  （sub_41DE30/41EBC0/41F790/41E2A0/41C700/41EA50/41EDD0/41EFF0 —— 模块生命周期钩子）。',
]
GFX_NOTES['0x418120'] = ['std::out_of_range("invalid vector<T> subscript") 抛出器（MSVC vector::operator[] 检查版）。']
GFX_NOTES['0x418190'] = ['std::length_error("vector<T> too long") 抛出器。']

# ---------------------------------------------------------------- zlib_inflate
ZLIB_SECTIONS = [
 (0x650030, 'Neople 游戏演出代码：技能/效果对象状态机（虚调用密集，与 0x42-0x46 战斗区衔接）'),
 (0x650C30, '粒子特效加载（Common/CommonEffect/Particle/*.ptl）'),
 (0x6510A0, '剑魂震波音效族（QUAKE_WAVE_READY_LOOP / QUAKE_WAVE_LOOP / SM_GHOSTSTEP 幽灵步克隆）'),
 (0x651B60, '状态解除/斗气/瞄准要害演出（RemoveActiveStatus.ptl / Provocationed.ani / AimedVitalPoint.ani）'),
 (0x6525A0, '窗口/客户端小工具（GetClientRect 等）'),
 (0x6527F0, '★★ 内嵌 OLE 浏览器（NEOPLE_WEB: OleCreate WebBrowser 控件 / Navigate / NAVIGATION START/END 调试标记）'),
 (0x652DE0, '★★★ zlib 1.2.3 静态链接（uncompress / inflateInit2_ / inflate / inflate_table / inflate_fast(MMX) / crc32 / zalloc/zfree；版本串 "1.2.3"）'),
 (0x655A90, 'Neople 调试打印（enduranceTime/waitTime/elapsedTime 计时调试 → 0x40BE20 控制台）'),
 (0x6560D0, '观察者/容器族（vector 工具 + 对象分配，供上层效果系统）'),
 (0x656810, 'MSVC ConcRT 运行时桩（ContextBase::SweepUnstructured ×3 —— 编译器注入，非游戏逻辑）'),
 (0x656840, '定时器/调度容器（GetTickCount / timeGetTime 节流链表）'),
 (0x656D70, '链表/网络对象工具（通用双向链表清空 0x6569D0 全引擎复用）'),
 (0x657120, '★ UDP 中继客户端（startup/connected to Relayserver/Disconnected —— socket/htons/inet_addr）'),
]

ZLIB_NOTES = {}
ZLIB_NOTES['0x652E00'] = [
 '★ uncompress()（zlib 1.2.3）: inflateInit_(version="1.2.3", sizeof(z_stream)=56) →',
 '  inflate(4=Z_FINISH) → inflateEnd；Z_OK → 输出总长；Z_BUF_ERROR(-3)/Z_DATA_ERROR 透传。',
 '  ★ 被 gfx_texture.cpp 0x4116A0/0x411B30/0x412150/0x412700 调用 —— IMG 图像块解压入口。',
]
ZLIB_NOTES['0x652EB0'] = ['inflateReset(): 状态复位（窗口 0x8000、window 指针 +332、计数清零）。zlib 标准。']
ZLIB_NOTES['0x652F10'] = [
 'inflateInit2_(): 版本校验（version 首字节必须为 0x31 即字符1 + stream_size==56）→ zcalloc 分配 9520B 状态',
 ' → windowBits>>4+1 建窗口。zlib 标准（windowBits 语义见 zlib 源 inftrees/inflate）。',
]
ZLIB_NOTES['0x653000'] = ['inflate 内部子过程（78 行，间接调用密集）【推断：fixed tables/边界分支】。zlib 标准体。']
ZLIB_NOTES['0x653120'] = [
 '★★★ inflate() 主状态机（1477 行，zlib 1.2.3 直证）:',
 '  错误串 "incorrect header check" / "unknown compression method" / "invalid window size" /',
 '  "unknown header flags set" / "header crc mismatch" / "invalid block type" /',
 '  "invalid stored block lengths" / "invalid distance code" / "invalid distance too far back" /',
 '  "invalid literal/length code"。0x65 区 zlib 群的核心；IMG 解码（0x41 区）的底层。',
]
ZLIB_NOTES['0x654750'] = ['inflateEnd(): 状态非空 → zcfree（先释放 state+52 附属窗口再释放状态本体）。zlib 标准。']
ZLIB_NOTES['0x6547A0'] = ['zcalloc(): malloc(items*size) —— zlib 分配器挂钩（0x652F10 挂到 stream+8/+9）。']
ZLIB_NOTES['0x6547C0'] = ['zcfree(): free(block) —— zlib 释放挂钩。']
ZLIB_NOTES['0x6548D8'] = [
 '★ inflate_fast()（555 行，MMX 优化版）: __m64/_m_psrlq/_m_psllqi/_m_por/_mm_cvtsi32_si64',
 '  等 SIMD 内建加速的解码快路径。zlib 1.2.3 官方无 SIMD —— 发布版/Neople 定制补丁【推断】。',
 '  错误串 "invalid distance code"/"invalid distance too far back"/"invalid literal/length code"。',
]
ZLIB_NOTES['0x654F70'] = ['inflate_table(): Huffman 解码表构建（286 行）—— zlib 标准函数。']
ZLIB_NOTES['0x655460'] = ['crc32(): zlib 标准循环校验（表驱动）。']
ZLIB_NOTES['0x6556B0'] = ['crc32 内部（字节序分派/表构建 usercall）。zlib 标准体。']
ZLIB_NOTES['0x655990'] = ['crc32 包装（转 sub_6556B0）。']
ZLIB_NOTES['0x652D30'] = [
 '★ 内嵌浏览器宿主窗口注册: OleInitialize → RegisterClassExA("Browser Example") →',
 '  CreateWindowExA（标题 "NEOPLE_WEB"，与 log_dump.cpp 0x4090D0 抓屏目标对应）→ EnableWindow。',
 '  "Browser Example" 为 Neople 集成 IE 控件时的工程残留命名。',
]
ZLIB_NOTES['0x6528C0'] = [
 'OleCreate 内嵌 WebBrowser 控件: GlobalAlloc 站点内存 → OleCreate(IOleObject) →',
 '  OleSetContainedObject → SetWindowLong 挂站点。0x652880 = 控件分离（还原 WndProc + GlobalFree）。',
]
ZLIB_NOTES['0x652A70'] = [
 'Navigate: MultiByteToWideChar + SysAllocString(BSTR URL) → Variant 打包 → 浏览器控件 Navigate',
 ' → VariantClear。内嵌浏览器的 URL 跳转入口。',
]
ZLIB_NOTES['0x652B50'] = ['浏览器宿主 WndProc: OleUninitialize / GetParent / SetFocus / EnableWindow / DefWindowProcA 分发。']
ZLIB_NOTES['0x6527F0'] = [
 '导航调试标记: "************** NAVIGATION START **************" / "...NAVIGATION END..."',
 ' → OutputDebugStringA + PostMessageA。（docs/04 把本区归为“寻路/导航”实为此导航调试，非寻路算法。）',
]
ZLIB_NOTES['0x650C30'] = ['加载 Common/CommonEffect/Particle/AttackPowerUp.ptl（攻击力上升粒子，new + sub_428F30 粒子对象）。']
ZLIB_NOTES['0x6519E0'] = [
 '加载 Character/Swordman/Effect/Particle/DOTArea{Quake,Rock,Dust}.ptl 三连 ——',
 '剑魂（Swordman）范围 DOT 状态的震波/落石/尘土粒子。',
]
ZLIB_NOTES['0x6510A0'] = ['"QUAKE_WAVE_READY_LOOP" 音效状态查询（返回就绪循环标志）。']
ZLIB_NOTES['0x6511E0'] = ['"QUAKE_WAVE_READY_LOOP" 设置/清除（sub_64A570 音对象转发）。']
ZLIB_NOTES['0x651760'] = [
 '"QUAKE_WAVE_LOOP" 震波循环主逻辑（154 行）: sub_64A790 音对象 + sub_699000 时钟 +',
 ' sub_4922F0 效果查询 + sub_651510（120 行状态构建）。',
]
ZLIB_NOTES['0x651260'] = ['"SM_GHOSTSTEP" 幽灵步（剑魂被动）状态克隆: __RTDynamicCast 跨层次转型 + sub_64A360 复制。']
ZLIB_NOTES['0x6520B0'] = ['应用 Character/Fighter/Effect/Animation/Provocationed.ani（斗气爆发）—— 格斗家演出切换（__RTDynamicCast + 动画接口族）。']
ZLIB_NOTES['0x652380'] = ['应用 Character/Fighter/Effect/Animation/AimedVitalPoint.ani（瞄准要害）。']
ZLIB_NOTES['0x651F70'] = ['加载 Common/CommonEffect/Particle/RemoveActiveStatus.ptl（状态解除特效）。']
ZLIB_NOTES['0x655A90'] = [
 '计时调试打印: "-------------%d---------------" 与',
 ' "%d enduranceTime_=>%f waitTime_=>%f elapsedTime=>%f" → sub_40BE20（_DEBUG 控制台）。',
]
ZLIB_NOTES['0x657120'] = [
 'UDP 中继客户端启动: "\\nstartup[%s] :%d" + "\\ninitialize(getIP(),getPort())"',
 '（ninitialize/getIP/getPort —— RTII 网络接口族）。',
]
ZLIB_NOTES['0x6571C0'] = ['"\\nDisconnected": closesocket 断连通知。']
ZLIB_NOTES['0x6572C0'] = [
 'UDP socket 建立: socket() 失败 → "\\nCould not create a UDP socket : %d"；成功 →',
 ' "\\nconnected to Relayserver"（htons/inet_addr 连中继服）。与 0x41CC80 的 @@UDP STOP/START 呼应。',
]
ZLIB_NOTES['0x656810'] = ['MSVC ConcRT 运行时桩（Concurrency::details::ContextBase::SweepUnstructured 实例 1）—— 编译器注入。']
ZLIB_NOTES['0x656820'] = ['同上（实例 2）。']
ZLIB_NOTES['0x656830'] = ['同上（实例 3）。']
ZLIB_NOTES['0x6569D0'] = [
 '通用双向链表清空器: free 全部节点 + 哨兵自指 + 计数清零。',
 '全引擎复用（0x40/0x41 区大量析构链调用）。',
]

# ---------------------------------------------------------------- sound_script
SND_SECTIONS = [
 (0x6680B0, '0x66 区尾段：GetMessageBuffer 2 的幂缓冲池 / flex 风格分块扫描 / WSAStartup / CRT 支撑（operator new、std::_Xran/_Xlen、_Init_locks）'),
]
SND_NOTES = {}
SND_NOTES['0x6680B0'] = [
 '★ GetMessageBuffer 2 的幂级缓冲池分配器（异常串自证 "GetMessageBuffer"）:',
 '  请求 n 字节 → 升档到 16/32/64/.../0x80000（2 的幂 ×15 档，二分判断链）；',
 '  每档 operator new(档位 + 0x14) —— 对象头 0x14(20B):',
 '    [+0] vftable（0x8000 档用 0x741FD8，其余 0x743ED8）',
 '    [+4] 数据指针 = this+10（即 0x14 偏移处缓冲）',
 '    [+8] 档位容量；[+12..] 游标/状态清零；小档（16/32）逐 DWORD 清零展开；',
 '  n > 0x80000 → 以字符串 "GetMessageBuffer" 抛异常（拒绝超额）。',
 '  消费者 = 0x668340 分块扫描循环。',
]
SND_NOTES['0x668320'] = ['缓冲对象析构（vftable → off_743ED0 + a2&1 → free）。与 0x6680B0 配对。']
SND_NOTES['0x668340'] = [
 '分块扫描/refill 驱动循环（flex 风格，userpurge）:',
 '  vt+4 取余量 → 循环: vt+8 取当前指针 → 读 [+2] 处 2B 块长 →',
 '  余量 < 块长 → break；块长 0 → vt+16(余量) 提交返回；',
 '  否则 vt+12 保存回溯点 → sub_6680B0(块长) 申请缓冲 → qmemcpy 拷入 → vt+16 提交 →',
 '  缓冲对象 vt+0(a2) 交给处理器（yylex 式逐块消费）→ vt+4 再取余量继续。',
 '  【推断】.str/. PAK 词法驱动的缓冲 refill 骨架（flex yy_create_buffer 同构）。',
]
SND_NOTES['0x668410'] = ['WSAStartup(2.0) 惰性初始化（dword_9D21E0 保证仅一次；失败返回 0）。0x66 区网络/上报侧依赖。']
SND_NOTES['0x668444'] = ['operator new（CRT）: malloc 循环 + _callnewh(new_handler) + std::_Nomemory。']
SND_NOTES['0x668468'] = ['std::_String_base::_Xran → 抛 std::out_of_range("invalid string position")。']
SND_NOTES['0x6684A8'] = ['std::_String_base::_Xlen → 抛 std::length_error("string too long")。']
SND_NOTES['0x6684E8'] = ['operator new thunk（j_ 前缀跳转壳 → 0x668444）。']
SND_NOTES['0x6684ED'] = [
 'std::_Init_locks 构造: InterlockedIncrement(&Addend) == 0（首次）→',
 '  对 stru_9D21E8..9D2248 锁数组逐个 unknown_libname_14 初始化 —— CRT 流锁自举。',
]

# =====================================================================
# 注入引擎
# =====================================================================
MARK = re.compile(r'^// (0x[0-9A-F]{6}) (\S+)(.*)$')

def extract_strings(lines, i0, i1):
    body = '\n'.join(lines[i0:i1])
    strs = re.findall(r'"([^"\n]{3,90})"', body)
    seen, out = set(), []
    for s in strs:
        if s not in seen:
            seen.add(s); out.append(s)
    return out

def inject(src_path, newheader, sections, notes, tag=None):
    raw = io.open(src_path, encoding='utf-8').read()
    if '\ufeff' in raw:
        raw = raw.lstrip('\ufeff')
    if '【解读】' in raw:
        sys.exit('ABORT: %s 已含 L1 注释（注入非幂等）' % src_path)
    lines = raw.split('\n')

    # 预提取每函数字符串证据（按原始行号）
    marks = []  # (idx, va, name)
    for i, l in enumerate(lines):
        m = MARK.match(l)
        if m:
            marks.append((i, m.group(1), m.group(2)))
    strmap = {}
    for k, (i, va, name) in enumerate(marks):
        end = marks[k + 1][0] if k + 1 < len(marks) else len(lines)
        strmap[va] = extract_strings(lines, i, end)

    out = []
    n_notes = 0
    n_str = 0
    seen_secs = set()

    # 头替换：到 #include 为止的旧头
    hdr_end = 0
    for i, l in enumerate(lines):
        if l.startswith('#include'):
            hdr_end = i
            break
    out.extend(newheader.replace('@BAR@', BAR).replace('%%', '%').strip('\n').split('\n'))
    out.append('')
    out.append('#include "../../common/arad_types.h"')
    out.append(BAR)
    out.append('// SECTION 索引（按 VA 升序）')
    out.append(BAR)
    for a, t in sections:
        out.append('//   0x%06X — %s' % (a, t))
    out.append(BAR)
    # 跳过原 #include 后的第一个空行，由本表后的一个空行顶替（保持非注释行集不变）
    j = hdr_end + 1
    if j < len(lines) and lines[j].strip() == '':
        j += 1
    out.append('')
    out.extend(lines[j:])

    lines2 = out
    res = []
    i = 0
    while i < len(lines2):
        l = lines2[i]
        m = None
        if l.startswith('// ---') and i + 2 < len(lines2):
            mm = MARK.match(lines2[i + 1])
            if mm and lines2[i + 2].startswith('// ---'):
                m = mm
        if m:
            va, name = m.group(1), m.group(2)
            v = int(va, 16)
            # 分区横幅
            secva = None
            for a, t in sections:
                if a <= v:
                    secva = (a, t)
            if secva and secva[0] not in seen_secs:
                seen_secs.add(secva[0])
                res.append(BAR)
                res.append('// SECTION 0x%06X — %s' % secva)
                res.append(BAR)
            note = notes.get(va)
            strs = strmap.get(va, [])
            tail = ''
            if tag and tag.get(va):
                tail += ' [%s]' % tag[va]
            if not note and strs:
                show = []
                for s in strs:
                    if len(show) >= 3:
                        break
                    show.append('"%s"' % (s if len(s) <= 60 else s[:57] + '...'))
                if show:
                    tail += ' [串: %s]' % ' | '.join(show)
                    n_str += 1
            if note:
                n_notes += 1
            res.append(DASH)
            res.append('// %s %s%s' % (va, name, tail))
            if note:
                res.append(DASH)
                res.append('// 【解读】')
                for nl in note:
                    res.append('//   ' + nl)
            res.append(DASH)
            i += 3
            continue
        res.append(l)
        i += 1

    text = '\n'.join(res)
    text = text.replace('@@N_NOTES@@', str(n_notes)).replace('@@N_STR@@', str(n_str)).replace('@@N_SEC@@', str(len(seen_secs)))
    io.open(src_path, 'w', encoding='utf-8', newline='\r\n').write(text)
    print('%s: notes=%d str-annotated=%d sections=%d' % (src_path.split('\\')[-1], n_notes, n_str, len(seen_secs)))

# =====================================================================
# 文件头
# =====================================================================
HDR_LOG = '''
@BAR@
// 【40JP逆向源代码工程】src/nengine/log_dump.cpp
// 模块: 引擎核心 0x40 区 —— 程序入口 / 日志 / 崩溃报告链 / NPK 包加载 / Win32-D3D8-DSound-DInput 框架
// 来源: work/dump_engine.txt  区块: 0x400000-0x410000
// 层级: L2 转写层（Hex-Rays 逐字对应）+【L1 精修层】
//
// 【L1 精修内容】（注释级，函数体与 L2 逐字一致，零改动）:
//   ① 文件头重写 + 23 个 SECTION 分区横幅（@@N_SEC@@ 段生效）+ SECTION 索引表
//   ② @@N_NOTES@@ 个关键函数【解读】注释块（WinMain/崩溃链/NPK/日志核心/D3D8/IME/控制台）
//   ③ @@N_STR@@ 个函数标记行追加 [串: "..."] 字符串证据尾注（逐函数覆盖）
//
// ★★ 崩溃报告完整流程（本区直证，全链在 0x401F80-0x4028C0）:
//   [运行时] WinMain(0x401B20)
//     ├→ sub_402170(0, hInstance, off_77F284[0], off_77F288[0])   崩溃上下文初始化:
//     │    · 上报目标串 → ctx+126（unk_79C0C8 对象，off_77F284/288=上报地址对【推断:URL】）
//     │    · timeGetTime → "%%08x" 实例号 ctx+24；角色名占位 "_NONE" ctx+75
//     │    · sub_402030 生成文件名: "%%s_%%s.dmp"(ctx+386) / "%%s_%%s.trc"(ctx+646)
//     │      / "DNF_DUMP.zip"(ctx+906)，前缀=角色名_实例号
//     │    · sub_401F80 清理旧 *.dmp / *.trc / *.zip
//     └→ RegisterExceptionCallback(sub_401110)                     注册崩溃回调
//   [崩溃时] sub_401110(异常信息串)
//     ① sub_404020()  冲刷日志 fflush
//     ② sub_403A90()  NiLog 收尾
//     ③ 主管理器(dword_79C0C0)+113 角色名 → sub_402080 覆写报告名（50B 上限）
//     ④ sub_4020B0(异常串) 写 dmp 占位文件
//     ⑤ sub_402440 ★ 打包上报:
//         a. DNF.trc 全量复制为 %%s_%%s.trc
//         b. sprintf "a -tzip %%s %%s %%s" → ShellExecuteExA("7z","open",".\\\\")
//            —— 调系统 7-Zip 命令行打包 dmp+trc → DNF_DUMP.zip
//         c. GetExitCodeProcess 轮询 259(STILL_ACTIVE) + Sleep(1000ms)
//         d. zip 全量 → base64（0x4020F0 长度 + 0x401DA0 编码，76 列折行）
//         e. 拼 " content=" → sub_4028C0: InternetConnectA + HttpOpenRequestA("POST")
//            + "Content-Type: application/x-www-form-urlencoded" + HttpSendRequestA
//            （会话标识 InternetOpenA("NIS::Http")，0x402780）
//
// 其它要点:
//   - 0x401B20 WinMain 的 malloc(rand()%%100000) 为启动哨兵分配；srand(timeGetTime) 播种。
//   - 日志体系: NiLog（0x403D60 写入器，5000 行轮转 + Log File Opened/Closed 横幅 +
//     "%%02d/%%02d/%%04d - %%2d:%%02d:%%02d" 时间戳）、NiMemory.log（0x406C60）、
//     NiProfile.log（0x406CA0）、_DEBUG 调试控制台（0x40D510 AllocConsole + MS UI Gothic）。
//   - NPK 封包: "NeoplePack_Bill" 魔数（0x40E9E0）+ "*.npk" 目录扫描（0x40EF70）+
//     文件名 tolower 红黑树索引（0x40D660-0x40E860）；包内文件打开 0x40E3E0 被
//     gfx_texture.cpp 0x418CD0（IMG）复用。
//   - off_77F3FC = 全局日志函数指针（引擎级格式化打印）；sub_404F40/404F80 = NiProfile
//     计时作用域对（rdtsc 底层 0x404E70）。
//   - off_77F284/288: 崩溃上报地址数据对【推断】。
//
// 函数数: 442（L1 覆盖 442：解读 @@N_NOTES@@ / 串证据 @@N_STR@@ / 分区 @@N_SEC@@）
@BAR@
'''

HDR_GFX = '''
@BAR@
// 【40JP逆向源代码工程】src/nengine/gfx_texture.cpp
// 模块: 引擎核心 0x41 区 —— D3D 纹理绘制 / CNImagePackEx 图像包(IMG) / NeopleFontFile+Arial Narrow 字体 / WAV / 模块切换
// 来源: work/dump_engine.txt  区块: 0x410000-0x420000
// 层级: L2 转写层（Hex-Rays 逐字对应）+【L1 精修层】
//
// 【L1 精修内容】（注释级，函数体与 L2 逐字一致，零改动）:
//   ① 文件头重写 + 29 个 SECTION 分区横幅（@@N_SEC@@ 段生效）+ SECTION 索引表
//   ② @@N_NOTES@@ 个关键函数【解读】注释块（CNImagePackEx::Load/Destroy、IMG 解码族、
//      字体构建、onChangeModule、界面 IMG 预载等）
//   ③ @@N_STR@@ 个函数标记行追加 [串: "..."] 字符串证据尾注（逐函数覆盖）
//
// ★★ IMG（Neople Image File）格式推断（0x418CD0/0x418E20/0x419550 直证）:
//   [文件头] 0x20(32) 字节魔数 = "Neople Image File"（strcmp 校验，不足补齐到 32B）
//   [记录流] 顺序记录，每条:
//     · 4B 记录头 = 图像 ID（0x419550 读出后交设备 vt+8 CreateImage）
//       └ 特例 ID == 0x11(17): 别名条目 —— 再读 4B 源索引，与已建图像共享指针（flag=1）
//     · 图像数据块 = zlib 压缩流（★ 解码族 0x4116A0/0x411B30/0x412150/0x412700
//       均调 zlib_inflate.cpp 0x652E00 = uncompress），块长自描述:
//       图像对象 vt+116(FILE*) 装载并返回消费字节数 → fseek 定位下一记录
//   [运行时目录] 12 字节/项矢量（this+52..60 三指针）: [数据偏移 4B | 图像对象 4B | 别名标志 1B]
//     —— 由 Load 构建；0x418E20 据此做延迟加载（fseek(offset) + 图像 vt+108 装载）
//   [包根决策] 0x420190（0x42 区）: _access("_ImagePacks") 开发目录 → 缺省 "ImagePacks2/"；
//     路径拼装 "%s%%c%s%%c%s"（0x419D40）
//   [自证字符串] CND3DTexture32/16::CalcuatePersistSize（二进制串证据，本区纹理持久尺寸计算）
//
// 其它要点:
//   - CNImagePackEx::Load（0x419550）/Destroy（0x419320）函数名自证；CreateImage 失败/
//     SetImagePackEx 失败/open_file failed 三类错误日志（off_77F3FC）。
//   - SetImagePackEx（0x410750）把图像包绑到 CND3DBaseTexture（+32/33），重复绑定报错。
//   - 字体: NeopleFontFile 自有字体容器（0x416F40/0x417030 魔数校验）+ Arial Narrow
//     GDI 栅格化（0x4176E0/0x417B60 CreateFontA+CreateDIBSection+TextOutA）。
//   - onChangeModule（0x41CC80）: MODULE_TYPE_PVP/WARROOM/MAIN_GAME/SELECT_DUNGEON 切换时
//     "@@UDP STOP/START" 广播 + LADDER_DOWN + STOP SOUND + MAP_APPEAR（seatIndex 直证）。
//   - 界面 IMG 预载八件套（0x41F2C0）: Sprite/Interface/{SelectCharacter,CreateCharacter,
//     CharacterIllust,Base,ItemInventoryWindow,AvatarInventoryWindow,ItemTradeWindow,
//     EquipmentShopWindow}.img。
//
// 函数数: 370（L1 覆盖 370：解读 @@N_NOTES@@ / 串证据 @@N_STR@@ / 分区 @@N_SEC@@）
@BAR@
'''

HDR_ZLIB = '''
@BAR@
// 【40JP逆向源代码工程】src/nengine/zlib_inflate.cpp
// 模块: 0x65 区 —— Neople 演出代码 + zlib 1.2.3 静态链接 + 内嵌 OLE 浏览器 + UDP 中继客户端
// 来源: work/dump_tail_a.txt 系  区块: 0x650000-0x660000
// 层级: L2 转写层（Hex-Rays 逐字对应）+【L1 精修层】
//
// 【L1 精修内容】（注释级，函数体与 L2 逐字一致，零改动）:
//   ① 文件头重写 + 13 个 SECTION 分区横幅（@@N_SEC@@ 段生效）+ SECTION 索引表
//   ② @@N_NOTES@@ 个关键函数【解读】注释块（inflate 全家、OLE 浏览器、QUAKE_WAVE、UDP 中继）
//   ③ @@N_STR@@ 个函数标记行追加 [串: "..."] 字符串证据尾注（逐函数覆盖）
//
// ★ Neople 自研 vs 第三方 对照（本区判定）:
//   [zlib 1.2.3 标准 —— 版本串 "1.2.3"（0x652E00），0x652DE0-0x655A40]
//     0x652E00 uncompress / 0x652EB0 inflateReset / 0x652F10 inflateInit2_ /
//     0x653120 inflate(1477 行状态机) / 0x6548D8 inflate_fast（★MMX 定制版，官方 1.2.3 无 SIMD）
//     / 0x654F70 inflate_table / 0x655460+0x6556B0 crc32 / 0x654750 inflateEnd /
//     0x6547A0+0x6547C0 zcalloc/zcfree
//     → 上游消费者: gfx_texture.cpp 0x4116A0 族（IMG 图像块解压）
//   [Neople 自研]
//     0x650030-0x6527C0 演出代码（.ptl 粒子加载 / QUAKE_WAVE / SM_GHOSTSTEP / .ani 应用 /
//       效果对象状态机）、0x655A90 计时调试、0x6560D0-0x656D70 容器与调度、
//       0x657120-0x6572C0 UDP 中继客户端
//   [内嵌 OLE 浏览器 —— 0x6527F0-0x652DE0]
//     "Browser Example" 宿主窗口类（Neople 工程残留命名）+ "NEOPLE_WEB" 标题 +
//     OleCreate WebBrowser 控件 + Navigate + "NAVIGATION START/END" 调试标记
//     （docs/04 所记“寻路”实为此网页导航调试，非寻路算法；flex yy_create_buffer
//      残留在 src/platform/missing_tail.cpp，不在本文件）
//   [编译器注入 —— 非游戏逻辑]
//     0x656810/0x656820/0x656830 = MSVC ConcRT ContextBase::SweepUnstructured ×3
//
// 函数数: 153（L1 覆盖 153：解读 @@N_NOTES@@ / 串证据 @@N_STR@@ / 分区 @@N_SEC@@）
@BAR@
'''

HDR_SND = '''
@BAR@
// 【40JP逆向源代码工程】src/nengine/sound_script.cpp
// 模块: 0x66 区尾段（0x668000-0x670000）—— GetMessageBuffer 缓冲池 / flex 风格分块扫描 / CRT 支撑
// 来源: work/dump_tail.txt 系  区块: 0x660000-0x670000（本文件仅含尾段 9 函数）
// 层级: L2 转写层（Hex-Rays 逐字对应）+【L1 精修层】
//
// 【L1 精修内容】（注释级，函数体与 L2 逐字一致，零改动）:
//   ① 文件头重写（含下方 .str 标签全表）+ 1 个 SECTION 横幅
//   ② 9/9 函数全部带【解读】注释块（本文件函数少，逐函数全量精修）
//
// ★★★ 声音脚本 .str 格式 —— 标签全表（本区关键产出）
//   .str 为 XML 属性式脚本（解析器在本文件之外的 0x66 前段，位于
//   src/platform/missing_tail.cpp）:
//     · 0x663910 = 属性 getter（节点+13 属性表查名，命中返回 [+8] 串，否则 szPassword 空串）
//     · 0x663BB0 = 节点序列化器（"<tag attr=…>…</tag>" 输出 —— XML 格式直证）
//     · 数值经 VARIANT（vt=8 BSTR → sub_660F50 取 int / sub_660FF0 取 float）
//   标签 → 解析函数 → 存储槽（全表由 work/dump_missing_0/1/2.txt 直证）:
//     ┌──────────────────────┬────────────┬──────────────────────────────────┐
//     │ 标签                  │ 解析函数    │ 语义 / 存储                      │
//     ├──────────────────────┼────────────┼──────────────────────────────────┤
//     │ PRIORITY             │ 0x6611E0   │ 播放优先级 → int this[1]          │
//     │ VOLUME_ADJUST        │ 0x6611E0   │ 音量增幅 %%×100 → int this[2]      │
//     │ VOLUME_ADJUST_RANGE  │ 0x6611E0   │ 音量随机幅度 %%×100 → int this[3]  │
//     │ LOOP_TIMES           │ 0x6611E0   │ 循环次数 → int this[6]            │
//     │ LOOP_DELAY           │ 0x6611E0   │ 循环间隔 ×1000(ms) → this[4]      │
//     │ LOOP_DELAY_RANGE     │ 0x6611E0   │ 间隔随机量 ×1000(ms) → this[5]    │
//     │ FILE                 │ 0x661A20   │ 声音文件路径 → CString this[9]    │
//     │                      │ 0x662270   │ （第二处 FILE 型节点）            │
//     │ MINDIST              │ 0x661A20   │ 3D 最小距离 → float this+64       │
//     │ MAXDIST              │ 0x661A20   │ 3D 最大距离 → float this+68       │
//     │ CASCADENUM           │ 0x661A20   │ 级联声道数 → int this[18]         │
//     │ CASCADETAG           │ 0x661A20   │ 级联标签 → CString                │
//     │ NPCNO                │ 0x661CE0   │ NPC 编号过滤 → int                │
//     │ X / Y / Z            │ 0x661CE0   │ 触发坐标 → int ×3                 │
//     │ XRANGE/YRANGE/ZRANGE │ 0x661CE0   │ 触发范围半径 → int ×3             │
//     │ IN / OUT             │ 0x661F50   │ BGM 淡入/淡出时长                 │
//     │ LOOP                 │ 0x661F50   │ BGM 循环开关                      │
//     │ CROSSFADE_TO_OUT_TIME│ 0x661F50   │ 交叉淡出到下一曲的时长            │
//     │ PROB                 │ 0x662A40   │ 触发概率                          │
//     │ DELAY / DELAY_RANGE  │ 0x662CA0   │ 播放延迟 / 延迟随机量             │
//     └──────────────────────┴────────────┴──────────────────────────────────┘
//   解析链: 0x661A20(3D 声音源) 首行调 0x6611E0(基础属性) —— 层叠式属性继承。
//   背景定案（docs/04）: 0x66 = 声音脚本/PAK/XML 区，声音后端 FMOD 3.x；
//   演出侧消费点: 0x41CC80 onChangeModule（@@UDP STOP/START）与主帧循环声音延迟队列
//   （docs/13 §研究 P32 调用点直证 sub_65FAF0）。
//   ※ 本文件 9 函数为 0x66 区尾段的 CRT/网络/缓冲支撑（GetMessageBuffer 缓冲池、
//     WSAStartup、operator new、std::_Xran/_Xlen、_Init_locks），非 .str 解析本体。
//
// 函数数: 9（L1 覆盖 9：解读 9 / 分区 1）
@BAR@
'''

# =====================================================================
# 执行
# =====================================================================
inject(BASE + r'\src\nengine\log_dump.cpp',     HDR_LOG,  LOG_SECTIONS,  LOG_NOTES)
inject(BASE + r'\src\nengine\gfx_texture.cpp',  HDR_GFX,  GFX_SECTIONS,  GFX_NOTES)
inject(BASE + r'\src\nengine\zlib_inflate.cpp', HDR_ZLIB, ZLIB_SECTIONS, ZLIB_NOTES)
inject(BASE + r'\src\nengine\sound_script.cpp', HDR_SND,  SND_SECTIONS,  SND_NOTES)
