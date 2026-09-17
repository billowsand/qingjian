//! 打开文件 / 目录、定位随包资源、把日志打包到桌面。
//! 与 WinUI 版 `settings/src/panel/controls/files.rs` 同一套做法，spike 里失败只 `eprintln!`。

use std::path::{Path, PathBuf};

/// 随包资源（相对随包根，如 `data/generated/dicts`），定位逻辑与 Server 共用。
pub(crate) fn repo_resource(rel: &str) -> Option<PathBuf> {
    qingjian_platform::resources::bundled_resource(rel)
}

pub(crate) fn open_in_editor(path: &Path) {
    if let Err(error) = std::process::Command::new("notepad").arg(path).spawn() {
        eprintln!("打开 {} 失败: {error}", path.display());
    }
}

/// 资源管理器打开目录或网址。
pub(crate) fn open_with_explorer(target: &str) {
    if let Err(error) = std::process::Command::new("explorer").arg(target).spawn() {
        eprintln!("打开 {target} 失败: {error}");
    }
}

/// 三个进程共用的日志目录，没有就建出来（Server 没跑过时它还不存在）。
pub(crate) fn log_dir() -> Option<PathBuf> {
    let dir = qingjian_platform::dirs::log_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// 把整个日志目录加 `config.toml` 打成 `qingjian-logs-<日期>.zip` 放到桌面，再在资源管理器里选中它。
/// 压缩交给 PowerShell 的 `Compress-Archive`，不为此拉一个压缩库；桌面路径也让它取（OneDrive 会把桌面挪走）。
pub(crate) fn export_logs() {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let Some(logs) = log_dir() else {
        return;
    };
    let mut sources = vec![format!("'{}\\*'", logs.display())];
    if let Some(config) = qingjian_platform::dirs::config_path().filter(|path| path.is_file()) {
        sources.push(format!("'{}'", config.display()));
    }
    let zip_name = format!(
        "qingjian-logs-{}.zip",
        jiff::Zoned::now().strftime("%Y-%m-%d")
    );
    let script = format!(
        "$zip = Join-Path ([Environment]::GetFolderPath('Desktop')) '{zip_name}'\n\
         Compress-Archive -Path {} -DestinationPath $zip -Force\n\
         explorer.exe \"/select,`\"$zip`\"\"\n",
        sources.join(",")
    );
    let script_path = std::env::temp_dir().join("qingjian-export-logs.ps1");
    if let Err(error) = std::fs::write(&script_path, script) {
        eprintln!("写导出脚本失败: {error}");
        return;
    }
    let spawned = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script_path)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    if let Err(error) = spawned {
        eprintln!("导出日志失败: {error}");
    }
}
