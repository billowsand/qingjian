//! 给 qingjian_tsf.dll 嵌 VERSIONINFO（ProductName / ProductVersion）。
//! SignPath Foundation 免费代码签名的硬性要求：所有被签二进制必须带产品元数据，
//! artifact configuration 按 product-name / product-version 强制校验（docs/design/code-signing.md），
//! 且同一次构建里所有产物的版本值一致——CI 统一传 QINGJIAN_PRODUCT_VERSION（= 安装包版本），
//! 本地开发回落到 crate 自己的版本号。
//! 图标不走资源表（regsvr32 用的图标是 DLL 里 include_bytes 的数据段，见 installer README）。

fn main() {
    embed_version_info();
}

#[cfg(windows)]
fn embed_version_info() {
    println!("cargo:rerun-if-env-changed=QINGJIAN_PRODUCT_VERSION");
    let version = std::env::var("QINGJIAN_PRODUCT_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "Qingjian")
        .set("ProductVersion", &version)
        .set("FileVersion", &version)
        .set("FileDescription", "Qingjian TSF text service");
    if let Err(error) = res.compile() {
        println!("cargo:warning=嵌入 DLL 版本信息失败: {error}");
    }
}

#[cfg(not(windows))]
fn embed_version_info() {}
