# 设置程序换 egui 的 spike（2026-09-18）

分支 `egui-settings-spike`，代码在 `apps/windows/settings-egui`（不随包发布）。**只是量数字，还没有换的决定。**

## 起因

WinUI 3（`windows-reactor`）的设置程序要**自包含部署** Windows App Runtime：本机实测安装目录里多 **56.1 MB / 118 项**，
按 Inno 的 LZMA2 压完在安装包里占 **13.5 MB**（当前包 82 MB 的 16%）。这套运行时还带来两笔账：

- `settings/build.rs` 的 `/DELAYLOAD` + 自包含清单 hack，是 Windows 10 上能启动的唯一办法（[windows-win10.md](windows-win10.md)）；
  上游 windows-rs 的修复（PR #4873）至今关闭未合并。
- `installer/settings-runtime.txt` 118 行清单 + `build.ps1` 挑文件 + `qingjian.iss` 装目录，升级 `windows-reactor-setup` 时要人工核对。

[design/rendering.md](../design/rendering.md) 定过「设置程序继续用各平台原生 UI，不自绘」，理由是「控件面需要完整控件库，自绘等于写一个 toolkit」。
egui 是现成 toolkit，不落在那条理由里；`design/candidate-ui.md` 排除 egui 的理由（框架要拥有窗口和事件循环，与非激活浮层打架）只针对候选窗，
设置程序就是个普通窗口。所以这条路值得量一次。

## 做了什么

用 egui 0.33 + eframe（wgpu 后端 + accesskit）复刻设置界面。第一轮只做「通用」页量数字；第二轮补齐五页、改紧凑布局、测深色。
文案与取值照抄 WinUI 版；左侧导航、开关、候选窗预览自己画；配色直接取候选窗渲染器的 `Palette`；
中文字体走 `qingjian_render::system_fonts::family_files` 从系统装，不打包字体。读写的是同一个 `config.toml`，改一项就原地写回再重读。

## 数字（本机：Windows 11 26200，RTX 4090 Laptop，release 构建）

| 项 | WinUI 3（现在） | egui spike |
|---|---|---|
| exe | 5.08 MB | **11.86 MB**（五页齐） |
| 跟着装的运行时 | 56.1 MB / 118 项 | 0 |
| 安装目录合计 | 61.2 MB | **11.9 MB** |
| 安装包（同一提交各打一个） | 76.9 MiB | **66.2 MiB（-10.7 MiB / -13.9%）** |
| 窗口出现（连跑三次） | 195 / 108 / 94 ms | 252 / 156 / 157 ms |
| 进程起到首帧画完（程序自报） | 未量 | 570 ms（冷）/ 295 ms（热） |
| 稳定后 WorkingSet | 143 MB | **236 MB**（走 WARP 时 154 MB） |
| 全部代码 | 1817 行 | 2025 行（含字体 / 配色 / GPU 后端这 200 行框架白送的东西） |
| 一屏能放几项设置 | 4 项 | **9 项**（一行 34 px，WinUI 版两行文字 76 px） |

净账：**安装包小 10.7 MiB（-13.9%），安装目录小 50 MB，少装 118 个文件**；exe 自己大一倍。
真机试装用 `powershell -File apps\windows\installer\build.ps1 -EguiSettings`：egui 版设置程序按正式名字装进去，
Server 的齿轮、开始菜单、安装前 taskkill 都照常，成品是 `Zizai-<版本>-egui-Setup.exe`（与正式包不同名，不互相覆盖）。

## 四个问题的答案

1. **体积**：过。省下的就是上面那 56 MB / 13.5 MB，外加 `build.rs` 的 hack、118 行运行时清单、`windows-reactor` 这条依赖一起消失。
2. **启动**：**没赢，反而略慢**（热启动 156 ms vs 94 ms），内存高出约 90 MB（字形图集 + wgpu/D3D12 的驱动占用；走 WARP 时降到 154 MB）。
   换 egui 不能拿「更快更轻」当理由。
3. **无障碍**：能做到可用，但**每个控件都要手工补**。egui 默认：`ComboBox` / 自绘开关在 UIA 树里 Name 是空的，读屏只念得出「复选框 已选中」。
   补两样之后树就全了——`Response::labelled_by` 把控件系到左边的标签，自绘控件在 `widget_info` 里自报名字（导航项还要重报一次，
   否则图标那个私用区码点会被一起念）。补完的树：`ComboBox name=[双拼]`、`CheckBox name=[本地整句模型] toggle=On`，
   而且从 UIA 调 `TogglePattern.Toggle()` 能真的拨动开关并写进 `config.toml`——读屏用户操作得了。
4. **无 GPU 回退**：过。`QINGJIAN_SPIKE_SOFTWARE=1` 强挑 CPU 适配器，wgpu 落到
   `Microsoft Basic Render Driver / Cpu / Dx12`（D3D12 的 WARP），画面与独显下**逐像素一致**，首帧 251 ms。
   注意：eframe 的 `wgpu` feature 不替你选后端，不在自己的 `Cargo.toml` 里开 `wgpu/dx12` 会在运行时 panic
   「No wgpu backend feature…」——这也是不选 glow（OpenGL 3.3）的原因，RDP 会话与没装驱动的虚拟机上只有 1.1。

## 顺带发现

- 中文字体不用打包：`system_fonts::family_files("Microsoft YaHei")` 拿到 `msyh.ttc`，`egui::FontData` 有 `index` 字段能指 ttc 里的 face。
  代价是整份字体读进内存（雅黑 19 MB），也是上面内存差的一部分。
- 配色可以和候选窗同源（`Palette::light()/dark()`）——WinUI 版只能跟系统的 `ThemeBrush` 走，品牌色对不上，
  现在 `controls/mod.rs` 里那些「与候选窗口的 `Theme::corner_radius` 一致」的注释就是在手工对齐这件事。
- 图标可以继续用 Segoe Fluent Icons（私用区码点），显示正常；spike 里的码点是猜的，正式做要逐个核对。
- `NumberBox`（带上下箭头）在 egui 里只有 `DragValue`，观感差一截，要自己拼加减按钮。

## 第二轮：补齐五页、紧凑布局、深色（2026-09-18）

- **五页齐了**：候选窗口（含浅 / 深两套候选窗与状态条预览，自己画）、词库（读 `.qj` 的名称 / 条数 / 许可证，勾选、移除、导入）、
  高级（打开配置 / 数据 / 日志目录、打包日志、三个开关、清空输入日志）、关于（品牌、构建号、输入统计表、许可证与署名）。
  `files.rs` 从 WinUI 版原样移过来（`notepad` / `explorer` / `Compress-Archive` 那套）。
- **说明不再常驻**：每项一行，说明挂成整行的悬停提示，标签后一个淡色 ⓘ 表示「有说明」。
  一行从 76 px 降到 34 px，一屏从 4 项变 9 项。改的是 `widgets::field` 一个函数，页面代码一行没动——
  immediate mode 下「布局密度」是个能一处改全局的参数，这点比 WinUI 那套声明式舒服。
- **深色模式**：`ThemePreference::System` **在 Windows 上不灵**——系统切成深色后 winit 没把主题变化报过来，窗口一直停在浅色。
  改成自己读 `HKCU\...\Themes\Personalize\AppsUseLightTheme`（Server 判断候选窗深浅用的同一个键），每帧比一次、
  变了就换 Visuals + `ViewportCommand::SetTheme` 同步系统画的标题栏；egui 只在有输入时重绘，所以还要
  `request_repaint_after(800ms)` 定期醒来。这么改之后浅 ↔ 深来回切都即时跟随，两套配色都对。
- 顺手修的两处：状态条预览的盲文点阵 `⠿` 雅黑与 Segoe Fluent 都没有（显示成方框），改自绘六个点；
  随包词库标「· 随包」而不列许可证，与 WinUI 版一致。

## 没测

真实 Windows 10 机器、NVDA 实听、下拉展开后弹出层的无障碍树、DPI 中途切换、多显示器、
文本输入框与 IME（五页里都没有文本框；将来做自定义短语要单独验证——用自家输入法在自家设置里打字）。

## 眼下的看法

换的理由是**体积与依赖链**（56 MB 运行时、Win10 hack、`windows-reactor` 这条上游不动的依赖），不是性能，也不是省代码。
代价是无障碍要逐个控件补、观感不再是 Fluent（没有 Mica、系统强调色与控件动效）、多 188 行平台胶水，
以及要改掉 `design/rendering.md` 里「设置程序不自绘」那条决定。

还没比的备选：Slint（有 software renderer，不依赖 GPU，GPL 授权与本项目兼容）、纯 Win32 + windows-rs（exe 1–2 MB，原生无障碍，但暗色与布局要手写）。
