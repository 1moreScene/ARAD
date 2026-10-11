# custom-dnf —— DNF 同人客户端

使用本机官方资源的纯 Rust 客户端，无第三方运行时依赖。

2026-10-11 已重新取证并修正普通移动、奔跑、跳跃与转向。权威说明见
[02_移动与跳跃取证.md](docs/02_移动与跳跃取证.md)，包含 PE 地址、实测轨迹、公式、状态门和实现边界。

- 左右双击命令窗口 **500ms**；上下双击不能启动奔跑。
- **反向退出奔跑**。输入层双击候选与有效奔跑状态分别处理；按住一侧点反向键不建立持续奔跑。
- 普通跳跃：**准备100ms → 飞行 → 落地50ms**；飞行x继承起跳档位，y参数**整数除以2**。
- 鬼剑士的三个跳跃帧界是 **1 / 8 / 15**；顶点和落地由物理门选择。
- 走/跑动画按移速属性播放（基础850为85%），站立/普通跳跃100%。
- 内部保留浮点位置，输出坐标四舍五入，停步保留位置小数。

这些修正已进入代码与回归测试。官方状态请求/回执的瞬态、逐像素碰撞和其他职业专属跳跃尚未完整接入；不能据此声称整个官方客户端100%复刻。

常用Debug、Release与`target/movement-verified/release/dnf-app.exe`均随本轮更新。
窗口默认屏蔽游戏线程输入法并钉住焦点，**F10解除/恢复钉住**；失焦/最小化仅停止接收输入，世界逻辑和声音继续运行。
官方逻辑画布640×480；原版在本机125%桌面实际显示800×600。重制默认匹配这个显示大小，`--scale 1`可恢复物理640×480。见[窗口输入与分辨率](docs/03_窗口输入与分辨率.md)。
X已接入三段普攻，飞行中X进行单次空中攻击；见[攻击实现](docs/04_普通攻击与跳跃攻击准备.md)。默认装备官方1级损坏铁剑。
移动／奔跑脚步、普攻／空攻语音、短剑挥动和命中、起跳／落地已实际发声，支持异步重叠播放；`--volume 60`调音量、`--mute`静音。见[音效实现](docs/05_音效资源与接入准备.md)。
如果旧的 `target/release/dnf-app.exe` 窗口仍在运行，它不会自动加载新代码；关闭旧窗口后运行已验证文件：

```powershell
cd custom-dnf
.\target\movement-verified\release\dnf-app.exe
```

默认场景已有原版哥布林测试靶：奔跑中X突刺，Z上挑（MP6、冷却2秒），A鬼斩（MP20、冷却8秒），F5重置哥布林并补MP。鬼斩与上挑默认可从三段普通攻击强制施放，保存打断前姿势的白色残影；原版残影消退时长仍待确认。技能等级、MP、冷却、图标与附加伤害读自官方.skl，见[技能数据与强制取消](docs/11_鬼斩技能数据与强制取消.md)。

哥布林身体与装备已按官方.mob/.equ动作表绘制；地面受击375ms、双方独立100ms卡肉和倒地弹跳/起身改为原版取证逻辑。见[受击取证](docs/07_哥布林动作与受击反馈取证.md)。
**M/I打开角色属性／装备窗口，F6展开调试**，鼠标或Tab选值、输入数字后Enter应用，支持加减和恢复默认。移速/攻速/跳跃参数、MP和测试物攻可实际调整；属性派生和完整伤害公式仍待接通。
UI底图、职业插画、铁剑图标与伤害数字均加载官方素材；伤害字采用逐位精灵和500ms缩放/上浮/淡出时间线。见[UI说明与验证](docs/08_角色信息UI与伤害数字.md)。`--panel`启动时显示面板；面板打开暂停战斗，Esc先关闭面板。
显示已按原版交叉验证改为双线性采样，并修正16位素材的颜色位扩展；`--scale 1`仍为640×480原样输出。见[渲染取证](docs/09_色位与采样交叉验证.md)。
已接入DirectInput 256 DIK采样与统一CNCommandChecker：方向优先级、双击、按住/释放、四触发模式、命令优先级、动作权限与技能键序解析；**F7查看按键记录/状态**。14个技能模板已注册，未实装技能保持禁用。全部21默认键位已有PE证据，Buff默认修正为Space。见[按键管理器与失焦运行](docs/10_按键管理器与失焦继续运行.md)。

**K打开技能窗**，拖拽图标到A/S/D/F/G/H快捷栏（默认A鬼斩、S上挑），拖回技能窗移除；布局保存到data/skillbar.txt。快捷键字形使用Base.img原图，冷却保留原矩形遮罩与结束闪光，中心加一位小数秒数，悬停显示原版技能说明。HP/MP当前/最大值仅在胶囊悬停时显示，扣减保留原500ms白闪与每秒65px尾段回落。

施放失败会触发原版300ms显示颤动，并在左上临时通告列显示原因，最长8行/10秒。MP/冷却/物品提示读自ARAD.exe，HP费用是未来技能的公共扩展接口，鬼斩/上挑不额外消耗HP/无色。取消残影按反馈延长为600ms（本地标定）。右上显示真实FPS，渲染无上限，逻辑独立60Hz。详见[HUD与失败反馈](docs/12_HUD快捷栏与施放失败反馈.md)。

## 1. 怎么跑

```powershell
cd custom-dnf

# ① 看一眼官方数据与键位表（含每个键位的取证等级 + 房间/角色分层装载诊断）
cargo run --release -- --info

# ② 无头确定性回放：输出 PNG 到 out/（11 秒时间线，覆盖站立/步行/奔跑/跳跃/反向/纵深移动）
cargo run --release -- --headless

# ③ 打开窗口实际玩（640×480逻辑画布，显示随DPI）
cargo run --release

# ④ 换房间 / 换装 / 换回放脚本
cargo run --release -- --map map/lorien/110.map
cargo run --release -- --outfit data/outfit.txt --keymap data/keymap.txt
cargo run --release -- --headless --script jump --png-every 30 --out out/jump

# ⑤ 三个行为回归演示（可逐帧看 --trace）
#    mashtap：按住右 + 每 4 帧点按左 ⇒ 全程 WALK，绝不进入奔跑（官方语义）
#    turn   ：奔跑中反向 ⇒ 退回 WALK
#    airturn：站立起跳 → 空中先右后左 ⇒ 落地点可调（官方正常跳跃继承地面档位）
cargo run --release -- --headless --frames 120 --script mashtap --trace
cargo run --release -- --headless --frames 90  --script turn --trace
cargo run --release -- --headless --frames 45  --script airturn --trace
```

窗口内操作（官方默认键位）：

| 操作 | 键 |
|---|---|
| 移动（8 向） | `↑` `↓` `←` `→` |
| 起步冲刺（奔跑） | 双击左/右（500ms；单击只走143档） |
| 同轴反向键 | 后按优先：按住 ← 再按 → ⇒ 向右；松开 → ⇒ 回到 ←（官方语义） |
| 普通攻击 | `X`（再次按X可按帧窗口续第二、第三击；空中X攻击） |
| 跳跃 | `C` |
| 上挑 | `Z`（基础1级技能，MP6、冷却2秒，默认强制） |
| 鬼斩 | `A`（技能快捷栏1，基础1级技能，MP20、冷却8秒，默认强制） |
| 技能窗口与快捷栏 | `K`；拖放图标绑定六格，默认A/S/D/F/G/H，S也可上挑 |
| 跑动攻击 | 奔跑中 `X`（突刺） |
| 重置测试靶 | `F5`（哥布林HP／位置／计数及角色MP） |
| 属性／装备窗口 | `M` / `I`；打开时暂停战斗 |
| 面板调试设置 | `F6`；鼠标/Tab选值，Enter应用，支持加减与恢复默认 |
| 按键管理器调试 | `F7`；显示按下时间、按住时长、变化标记和命令状态 |
| 技能快捷栏 1-6 | `A` `S` `D` `F` `G` `H` |
| 道具快捷栏 1-6 | `1` `2` `3` `4` `5` `6` |
| 退出 | `Esc`（先关闭已打开的面板，再次按退出） |
| 解除／恢复焦点钉住 | `F10` |

客户端目录默认自动探测（`DNF_CLIENT_DIR` 环境变量可覆盖）：

```powershell
cargo run --release -- --info --client "D:\path\to\客户端目录"
```

## 2. 工程结构

```
custom-dnf/
├── crates/
│   ├── dnf-core/    逻辑核心（无依赖）：键位表 / 输入 / 移动 / 跳跃 / 动画 / 角色状态机 / 定步时钟
│   ├── dnf-data/    官方资源读取（无依赖）：Script.pvf(v1) / .chr / .ani
│   │                + 真实素材层：NPK v1 / IMG / TIL / MAP / EQU
│   ├── dnf-render/  软件渲染（无依赖）：640×480 RGBA 帧缓冲 / alpha 混合 / PNG 导出 / 5×7 点阵字体
│   └── dnf-app/     可执行：Win32 窗口（GDI 呈现 + GetAsyncKeyState 轮询）+ 无头回放
│                    资源门面 assets.rs / 分层角色 character.rs / 真实房间 room.rs
├── data/keymap.txt  官方默认键位（可直接改，程序启动时加载并覆盖内建值）
├── data/outfit.txt  时装（槽位 → 官方 [variation] 两个编号；改它即可换装）
├── docs/00_设计说明.md
├── docs/01_键位取证.md
└── out/             无头回放导出的 PNG（可目视验证）
```

## 3. 真实素材（官方格式要点）

渲染**只吃官方文件**，没有任何自制或替代贴图：

| 资源 | 格式 | 本工程用法 |
|---|---|---|
| `ImagePacks2/*.NPK` | 16B `NeoplePack_Bill\0` 头 + 条目表 `{off,u32 size,256B 名}` | `NpkSet` 一次扫描 216 个包，建立 5810 条 `sprite/...` 索引，按名随机读 |
| `*.img` | 32B 头（`Neople Image File\0`）+ 每帧 `u32 tag`；`0x11` = 别名帧（`+u32 目标帧`），否则 `sub,w,h,size32,x,y,canvas_w,canvas_h` + `w*h*2` 字节 **16 位 A1R5G5B5（未压缩）** | 解析器要求**恰好消费到文件尾**，否则报错（不猜格式） |
| `map/<区>/*.map` | 文本：`[tile]` / `[far·middle·near sight scroll]` / `[background animation]` / `[pathgate pos]` / `[animation]` | 房间布局、视差率、门坐标、物件层次与坐标 |
| `map/<区>/tile/*.til` | `[IMAGE]` + `[pass type]` 14×30 通行网格（16px 格） | 每条 `.til` = 224px 宽横条；网格 = 画布 224×480 |
| `equipment/.../avatar/<槽>/<槽>_<字母>/<动作>.ani` | 与角色动作同名，`[IMAGE]` 里是**路径模板** `.../sm_coat%02d%02da.img` | 用装备的 `[variation]` 填模板；**层值取 `.equ` 的 `[LAYER]`**（不是字母顺序）——字母只决定用哪个部件目录，同槽的兄弟部件一起画（见 `dnf-data/src/avatar.rs`） |

层次顺序（官方 `[closeback]`/`[bottom]`/`[normal]`/`[close]`）：

```
远景(视差 56%) → 中景(视差 90%) → 瓦片条(世界锁定)
  → closeback → bottom → normal(按 y 排序) → 角色 → close(前景，会遮住角色)
```

> **绘制偏移约定**（踩过坑，见 `docs/90` 会话 15）：`blit_frame(fb, f, x, y)` 的 `(x, y)`
> **就是像素左上角**；`帧 (f.x, f.y)` 这个「画布内位置」由调用方决定是否叠加 ——
> 角色与物件的偏移里已经含了 `[IMAGE POS] + (f.x, f.y)`，**只有瓦片条需要额外叠**（它的
> `[IMAGE POS]` 就是画布原点，坐标在网格里）。多叠一次会让角色飞走、少叠一次会让瓦片跑到屏幕顶上。

## 4. 移动与跳跃

| 档位 | x直/斜 | y直/斜 |
|---|---|---|
| 步行 | 143 / 119 | 114 / 95 |
| 奔跑 | 286 / 238 | 136 / 114 |
| 步行起跳的飞行 | 143 / 119 | 57 / 47 |
| 奔跑起跳的飞行 | 286 / 238 | 68 / 57 |

速度乘运行时move speed/1000，斜向不作cos45°归一化。
初速为官方495430整数公式，基础鬼剑士约604.5495px/s，属性加权重力1425px/s²。
准备与落地阶段锁定平面位移和转向；普通飞行允许调整落点和朝向，落地回自由态。
`--trace`输出准备/飞行/落地阶段和双击候选，详见[取证说明](docs/02_移动与跳跃取证.md)。

## 5. 资源与证据

PVF、NPK、地图、时装图层和键位资源说明见[设计说明](docs/00_设计说明.md)与[键位取证](docs/01_键位取证.md)。
移动相关证据以[02_移动与跳跃取证.md](docs/02_移动与跳跃取证.md)为准；旧文档中的250ms、反向仍跑、空中y全速和统一100%速率已经撤销。
运行账号有装备/属性修正：本次官方采样移速842，不能用PVF裸基础850宣称两者位移完全一致。

## 6. 排查手册

| 现象 | 原因 / 处理 |
|---|---|
| 窗口打开但**全黑** | 早期版本的 `WM_PAINT` 没用 `BeginPaint/EndPaint` 收尾 ⇒ 无效区域永不清除 ⇒ Windows 无限重发 `WM_PAINT` ⇒ 消息泵被占死、逻辑一帧都没推进。已修复（`crates/dnf-app/src/win32.rs` 的 `wnd_proc`），并在显示窗口前先渲染好第一帧。 |
| 窗口比 640×480 大、画面发糊 | 原版有Windows DPI兼容缩放。现在默认按DPI显式放大；本机125%显示800×600。需要物理原始像素时使用 `--scale 1`。 |
| 画面只有角色、背景或瓦片整层不见了 | 先跑 `--info` 看**房间装载诊断**：每一层背景/瓦片/物件都会打印解析出的路径、帧号、尺寸、画布偏移与失败原因。历史坑：`.ani` 的 `[IMAGE]` 帧号若从整行取数字，路径里的数字（`Map\01far1.img`）会被当成帧号 ⇒ 帧号越界 ⇒ 整层消失（已在 `docs/90` 会话 15 修复并加回归测试）。 |
| 角色飞走 / 瓦片跑到屏幕顶部 | 绘制偏移被叠加了两次或漏叠（见上文「绘制偏移约定」）。 |
| **改了代码但手感/行为没变** | 你运行的可能不是刚构建的那份 exe：`cargo run`（不带 `--release`）跑的是 `target/debug/dnf-app.exe`，`cargo run --release` 跑的是 `target/release/dnf-app.exe`——**两套是独立的**。改完代码请用 `cargo run --release`，或先 `cargo build && cargo build --release` 把两套都刷新（本项目会话 18 就踩过一次：旧的 debug 版停在几小时前，表现还是老逻辑）。 |
| 物件位置离谱（如 y 值等于它的动画名） | `.map` 物件的坐标在**第 4 个反引号之后**，别从整行取数字（`smallTree0.ani` 里的 `0` 不是坐标，已修复）。 |
| 角色/物件的图层错位（例如头发跑到身体后面、上衣不见） | 官方绘制顺序 = **`.equ` 的 `[LAYER]` 升序**（身体层 0，负数层画在身体之前）。`--info` 会打印每层的官方 `[LAYER]` 与来源图集，逐层核对即可。历史坑：曾把「部件字母 a/b」当成前后层（`a` = 身体之后）⇒ 头发被身体挡住、上衣被藏起来（会话 16 修复）。 |
| **左朝向**看起来"不支持"（部件错位/整层差一列） | 官方镜像 = 动画速率取负，等价于**以角色脚点为轴翻转画布**：翻转时层左边缘 = `角色屏幕x − (偏移 + 精灵宽) + 1`（未翻转 = `+ 偏移`）。只把精灵原地翻面而不改 x ⇒ 非对称部件（头发/护肩/护臂）错位；漏掉 `+1` ⇒ 整层差一列。回归测试：`game.rs::左右朝向互为镜像_站立与行走`（逐像素比对，站立与行走两种姿态）。 |
| 想确认「窗口上到底有没有画面」 | `cargo run --release -- --shot-after 90 --shot-out out/window.png` —— 跑 90 帧后**实拍窗口客户区**存 PNG 并退出（不是离屏渲染，是屏幕上的真实内容）。 |
| 想确认 GDI 呈现链本身是否正常 | `cargo run --release -- --gdi-selftest` —— 在内存 DC + DIB 段上跑同一条 `StretchDIBits` 调用并**读回像素**断言非黑。 |

## 7. 验证

```powershell
cargo test --workspace     # 164 项测试：逻辑与回归 + 官方PVF/NPK/EXE真值校验
cargo clippy --all-targets  # 零告警
cargo fmt --all -- --check  # 格式干净

# 窗口可视化验证（两步实证，不靠"应该没问题"）：
cargo run --release -- --gdi-selftest                       # 呈现链：目标位图非黑像素数 + 左上角像素
cargo run --release -- --shot-after 90 --shot-out out/window.png  # 窗口实拍
```

`crates/dnf-data/tests/official_client.rs` 与各模块的「真值」测试会直接打开本机的
`Script.pvf` / `ImagePacks2`，把解析结果与人工读出的官方真值逐项比对（找不到客户端时自动跳过）。

## 8. 下一步（不在第一步范围内）

1. 正式敌人AI与完整属性／伤害公式、特殊反馈分支；测试哥布林已接入脚本动作、真实命中盒、独立卡肉和基础受击／倒地时序。
2. 房间切换：`.map` 的 `[pathgate pos]` 已经解析出来了，接通「走到门口 → 换房间」。
3. 物件动画：现在取物件的第 1 帧静态绘制，`[DELAY]` 已在 `.ani` 里，可直接接 `AnimPlayer`。
4. 扩展音效：其他地表／环境／BGM与OGG；现有脚步、语音、挥剑／命中和起落已播放。`.map` 的 `[sound]`（洛兰104 = `G_LORIEN` / `RM_FOREST`）待接入。
5. 官方「键位设置」界面：把 `data/keymap.txt` 的覆盖机制接到 UI 上。
6. 属性派生、完整背包/装备更换及原版NeopleFontFile；当前已有原素材角色窗口和运行时调试。

音效验证：`cargo run --release -- --audio-selftest`；全链检查：仓库根目录运行`python tools/verify_custom_audio.py`。
无头模式默认静音，添加`--audio-out out/attack.wav`可以导出确定性音轨。

UI验证：`python tools/verify_custom_ui.py`；受击验证：`python tools/verify_custom_feedback.py`；采样验证：`python tools/verify_custom_rendering.py`。
按键/后台运行验证：`python -X utf8 tools/verify_custom_key_manager.py`；`--show-keys`启动时显示记录，`--info`查看14个命令模板。
HUD/拖放验证：`python -X utf8 tools/verify_custom_hud.py`；胶囊/施放失败/FPS验证：`python -X utf8 tools/verify_custom_hud_feedback.py`；技能取消验证：`python -X utf8 tools/verify_custom_skills.py`。
