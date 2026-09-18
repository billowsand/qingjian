# 设置界面 egui spike

只在分支 `egui-settings-spike` 上，**不随包发布、不接受依赖**。正式设置程序仍是 `apps/windows/settings`（Windows Reactor / WinUI 3）。

## 为什么有这个

WinUI 3 的设置程序要自包含部署 Windows App Runtime：安装目录多 56.1 MB / 118 个文件，安装包里多约 13.5 MB（LZMA2 压缩后），
还得靠 `build.rs` 里的 `/DELAYLOAD` hack 才能在 Windows 10 上启动（见 `docs/notes/windows-win10.md`）。
这个 spike 用 egui + wgpu 复刻整个设置界面（五页齐），量四件事决定值不值得换：

1. **体积**：release exe 大小（对比 WinUI 版 5.08 MB + 56.1 MB 运行时）。
2. **启动**：进程起来到第一帧画完的墙钟时间（程序自己 `println!`）。
3. **无障碍**：accesskit 在 UIA 树里给不给得出开关 / 下拉的名字与状态（读屏能不能念）。
4. **无 GPU 回退**：没有独显 / RDP 会话里 wgpu 能不能退到 WARP 软件适配器（程序自己打印选中的适配器）。

## 跑

```bash
cargo run -p qingjian-windows-settings-egui --release
```

控制台会打印首帧耗时与 wgpu 适配器。读写的是同一个 `%APPDATA%\Qingjian\config.toml`，
可以和正式设置程序开着对比（两边都是改一项就原地写回、再重读）。

缺省是 GUI 子系统（双击不弹黑窗），所以那两行 `println!` 要 `--features console` 才看得到：

```bash
cargo run -p qingjian-windows-settings-egui --release --features console
```

强制走软件适配器（模拟没有可用 GPU 的机器）：

```bash
QINGJIAN_SPIKE_SOFTWARE=1 cargo run -p qingjian-windows-settings-egui --release --features console
```

## 打一个能装的包

```bash
powershell -ExecutionPolicy Bypass -File apps/windows/installer/build.ps1 -EguiSettings
```

egui 版按正式名字 `qingjian-settings.exe` 装进 `{app}`，不装那 118 项 Windows App Runtime；
成品是 `target\installer\Zizai-<版本>-egui-Setup.exe`，与正式包不同名。同一提交实测 76.9 → 66.2 MiB。
装上后 Server 的齿轮、开始菜单打开的都是它。五页都能用，但仍是 spike：出问题回去装正式包。

## 实现上与 WinUI 版的差别

- **一页一张列表卡**：页内没有小节标题（分节在左侧导航里），行与行之间只有一条淡分隔线；窗口 700×560。
- **说明不常驻**：每项一行，说明挂成悬停提示（停在标签或它后面的 ⓘ 上），一行 38 px，一屏放 9 项（WinUI 版 4 项）。
- **图标由 painter 按字形外框居中画**，不跟字体基线走——否则私用区图标一定比汉字高一线。
- 字体自己装（`src/fonts.rs`）：跟随候选窗口的字体选择；选「系统字体」时两者共同使用 Segoe UI / Arial，并以 Microsoft YaHei、Yu Gothic 回退，不打包字体。
- 配色取候选窗渲染器的 `Palette`（`src/theme.rs`），设置界面与候选窗同一套色。
- 明暗自己读注册表跟随系统（`AppsUseLightTheme`，与 Server 判断候选窗深浅同一个键）：
  egui 的 `ThemePreference::System` 在 Windows 上收不到 winit 的主题变化，不能用。
- 左侧导航、开关是自己拼的（`src/nav.rs`、`src/widgets/toggle.rs`）——WinUI 那边是 `NavigationView`、`ToggleSwitch` 两个现成控件。
- 图标用 Segoe Fluent Icons 的私用区码点，**码点尚未逐个核对**，显示成别的图标属于已知现象。

结论与实测数字记在 `docs/notes/egui-settings-spike.md`。
