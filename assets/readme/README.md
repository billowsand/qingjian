# README 配图

仓库根 `README.md` 用的候选窗 / 状态条截图，不是设计稿也不是真机截图，而是由 `crates/qingjian-render` 的预览程序按主题出图：

```bash
cargo run --release -p qingjian-render --example preview -- --out target/render-preview
```

脚本按「场景 × 浅 / 深色」一次画全套（`cloud-`、`corrected-`、`nihao-`、`probe-`、`status-`），
文件名与场景名一致，`--scale 2` 的默认值让图是 2 倍图，README 里按一半宽度引用。

字体取本机系统字族，所以换机器重跑可能与这里提交的图有细微差别；改渲染器或主题时重跑一遍并把变化的图一起提交。
版权随项目（GPL-3.0-or-later），图形本身没有任何第三方素材。
