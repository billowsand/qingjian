# 字在设置（egui）

Windows 安装包缺省使用的设置程序。它读写 `%APPDATA%\Qingjian\config.toml`，改一项即原地保存，Server 在一秒内热加载。
WinUI 3 的旧实现仍放在 `apps/windows/settings`，只供 `build.ps1 -WinUiSettings` 对比。

## 运行

```bash
cargo run -p qingjian-windows-settings-egui --release
```

缺省是 GUI 子系统。要看首帧耗时与 wgpu 适配器，用：

```bash
cargo run -p qingjian-windows-settings-egui --release --features console
```

`QINGJIAN_SPIKE_SOFTWARE=1` 可强制使用 WARP 软件适配器，验证无独显 / RDP 回退。

## 界面约定

- 窗口 700×560，左侧 168 pt 导航；页面使用 38 pt 紧凑设置行。
- 关闭系统标题栏，由 `title_bar.rs` 绘制拖动区与最小化、最大化、关闭按钮；它与设置正文使用同一色系。
- 奶油、字在蓝、紫藤拿铁、森林四套配色各有浅色与深色；明暗固定跟随 Windows。
- 「候选窗口」页的主题卡是实时 egui 绘制，不是预览图片。卡内文字、设置正文、候选窗与状态条都使用 `[general] font`。
- Segoe Fluent Icons / Segoe MDL2 Assets 只装进 `zizai-icons` 字族。私用区图标必须显式用 `fonts::icon_font`，不能用 `FontId::proportional`。
- 配色角色直接来自 `qingjian_render::Palette`；设置窗口、候选窗口与悬浮状态条读取同一个 `[general] theme`。
- 说明不常驻：停在标签或 `ⓘ` 上显示。自绘开关与导航项要补 `widget_info`，右侧控件要用 `labelled_by` 关联标签。

## 打包

```powershell
powershell -ExecutionPolicy Bypass -File apps/windows/installer/build.ps1
```

安装器把 `qingjian-settings-egui.exe` 按正式名字 `qingjian-settings.exe` 安装，不携带 Windows App Runtime。
决定过程与实测数字见 `docs/notes/egui-settings-spike.md`。
