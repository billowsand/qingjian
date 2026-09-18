//! 在 Windows 上编时把字在图标嵌进 exe（任务栏 / 开始菜单 / 搜索里显示的就是它），与正式设置程序一致；
//! 再抓一下 git 构建标识（分支@短哈希 (日期)，工作区有改动加 +）给「关于」页显示。

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    if let Some(build) = git_build() {
        println!("cargo:rustc-env=QINGJIAN_BUILD={build}");
    }
    embed_icon();
}

fn git_build() -> Option<String> {
    let branch = git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let mut hash = git(&["rev-parse", "--short", "HEAD"])?;
    if git(&["status", "--porcelain"]).is_some() {
        hash.push('+');
    }
    let date = git(&[
        "show",
        "-s",
        "--format=%cd",
        "--date=format:%Y-%m-%d",
        "HEAD",
    ])?;
    Some(format!("{branch}@{hash} ({date})"))
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

/// 图标资源要 `rc.exe`（MSVC）编，只在 Windows 宿主上做；失败只警告，别让编译挂掉。
/// 同时嵌 VERSIONINFO 元数据：SignPath 签名按 product-name/product-version 校验
/// （docs/design/code-signing.md），版本统一用 QINGJIAN_PRODUCT_VERSION（CI 传安装包版本）。
#[cfg(windows)]
fn embed_icon() {
    const ICON: &str = "../tsf/resources/qingjian.ico";
    println!("cargo:rerun-if-changed={ICON}");
    println!("cargo:rerun-if-env-changed=QINGJIAN_PRODUCT_VERSION");
    let version = std::env::var("QINGJIAN_PRODUCT_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
    let mut res = winresource::WindowsResource::new();
    res.set_icon(ICON)
        .set("ProductName", "Qingjian")
        .set("ProductVersion", &version)
        .set("FileVersion", &version)
        .set("FileDescription", "Qingjian settings");
    if let Err(error) = res.compile() {
        println!("cargo:warning=嵌入设置程序图标失败: {error}");
    }
}

#[cfg(not(windows))]
fn embed_icon() {}
