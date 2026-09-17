# 图标

`logo.svg` 是「字在」图标的矢量源文件。图形由宝盖的横势、输入光标与开放的候选框组成；钴蓝是品牌色，薄荷色只表示正在输入或本地生效。

`generate.py` 用同一组几何参数生成随仓库提交的两个运行时资源：

- `assets/icon/logo.png`：1024×1024 RGBA 主图；
- `apps/windows/tsf/resources/qingjian.ico`：16–256 px 的 Windows 多尺寸图标。

```powershell
python assets/icon/generate.py
```

脚本需要 Pillow。文件名暂时保留 `qingjian.ico`，让现有安装包和升级路径不因品牌切换失效；它的图形已经是「字在」。
