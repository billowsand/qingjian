use std::path::PathBuf;

use thiserror::Error;

/// 配置文件的读写错误。两个 toml 解析错误装箱装：它们自带一大块内部状态，不装箱 `ConfigError` 就是 128 字节，
/// 让每个返回它的函数被 clippy 的 `result_large_err` 拦下（Windows 上尤其），而错误本身只在出错时构造一次。
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write config {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("cannot edit config {path} in place: {source}")]
    Edit {
        path: PathBuf,
        #[source]
        source: Box<toml_edit::TomlError>,
    },
}
