# 设置界面 egui spike

只在分支 `egui-settings-spike` 上，**不随包发布、不接受依赖**。正式设置程序仍是 `apps/windows/settings`（Windows Reactor / WinUI 3）。

## 为什么有这个

WinUI 3 的设置程序要自包含部署 Windows App Runtime：安装目录多 56.1 MB / 118 个文件，安装包里多约 13.5 MB（LZMA2 压缩后），
还得靠 `build.rs` 里的 `/DELAYLOAD` hack 才能在 Windows 10 上启动（见 `docs/notes/windows-win10.md`）。
这个 spike 用 egui + wgpu 复刻「通用」页，量四件事决定值不值得换：

1. **体积**：release exe 大小（对比 WinUI 版 4.94 MB + 56.1 MB 运行时）。
2. **启动**：进程起来到第一帧画完的墙钟时间（程序自己 `println!`）。
3. **无障碍**：accesskit 在 UIA 树里给不给得出开关 / 下拉的名字与状态（读屏能不能念）。
4. **无 GPU 回退**：没有独显 / RDP 会话里 wgpu 能不能退到 WARP 软件适配器（程序自己打印选中的适配器）。

## 跑

```bash
cargo run -p qingjian-windows-settings-egui --release
```

控制台会打印首帧耗时与 wgpu 适配器。读写的是同一个 `%APPDATA%\Qingjian\config.toml`，
可以和正式设置程序开着对比（两边都是改一项就原地写回、再重读）。

强制走软件适配器（模拟没有可用 GPU 的机器）：

```bash
WGPU_ADAPTER_NAME="Microsoft Basic Render Driver" cargo run -p qingjian-windows-settings-egui --release
```

## 实现上与 WinUI 版的差别

- 中文字体自己装（`src/fonts.rs`）：走渲染器已有的 `system_fonts::family_files` 找系统字体文件，不打包字体。
- 配色取候选窗渲染器的 `Palette`（`src/theme.rs`），设置界面与候选窗同一套色。
- 左侧导航、开关是自己拼的（`src/nav.rs`、`src/widgets/toggle.rs`）——WinUI 那边是 `NavigationView`、`ToggleSwitch` 两个现成控件。
- 图标用 Segoe Fluent Icons 的私用区码点，**码点尚未逐个核对**，显示成别的图标属于已知现象。

结论与实测数字记在 `docs/notes/egui-settings-spike.md`。
