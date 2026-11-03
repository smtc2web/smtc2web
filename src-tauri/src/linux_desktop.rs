//! Linux 桌面集成。
//!
//! Wayland 合成器（如 KWin）不读取窗口自带的图标（GDK 3 在 Wayland 下
//! 也不会设置窗口图标），而是按窗口 app_id 查找同名的 `.desktop` 文件，
//! 再用其中的 `Icon=` 解析任务栏图标。未通过 deb/rpm 安装时（开发运行、
//! AppImage），系统里没有这个入口，任务栏只能显示合成器的默认图标。
//!
//! 这里为这些场景写入一份用户级 `.desktop`（`NoDisplay`，不会出现在应用
//! 菜单）和 hicolor 图标；一旦检测到系统已安装同名入口，就清理自己写入的
//! 入口，避免遮蔽软件包安装的版本。

use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = "X-Smtc2web-SelfInstalled=true";
const ICON: &[u8] = include_bytes!("../icons/icon.png");

/// 确保当前运行方式（开发运行 / AppImage）有可供合成器解析的桌面入口。
pub fn ensure_desktop_entry() {
    // 由打包方负责提供入口，沙箱内写入用户目录对宿主桌面无效。
    if std::env::var_os("FLATPAK_ID").is_some() || std::env::var_os("SNAP").is_some() {
        return;
    }

    let Some(data_home) = dirs::data_dir() else {
        return;
    };
    let app_id = app_id();
    let user_entry = data_home
        .join("applications")
        .join(format!("{app_id}.desktop"));

    let ours = fs::read_to_string(&user_entry)
        .map(|content| content.contains(MARKER))
        .unwrap_or(false);

    if installed_entry_exists(&app_id, &data_home) {
        if ours {
            let _ = fs::remove_file(&user_entry);
        }
        return;
    }

    let Some(exec) = exec_path() else {
        return;
    };

    write_icon(&data_home, &app_id);

    let desktop = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=smtc2web\n\
         Comment=An software can let OBS show playing media.\n\
         Exec={}\n\
         Icon={app_id}\n\
         Terminal=false\n\
         NoDisplay=true\n\
         Categories=AudioVideo;Utility;\n\
         {MARKER}\n",
        quote_exec(&exec)
    );

    if let Some(parent) = user_entry.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&user_entry, desktop) {
        crate::log_warn!("写入桌面入口失败: {}", e);
    }
}

/// 窗口 app_id：GDK 在 Wayland 下取进程名（可执行文件名）。
fn app_id() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "smtc2web".to_string())
}

/// AppImage 用原始镜像路径（挂载点会变化），其余情况用当前可执行文件。
fn exec_path() -> Option<PathBuf> {
    std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| std::env::current_exe().ok())
}

/// 按 Desktop Entry 规范引用 Exec 路径。
fn quote_exec(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let needs_quote = raw
        .chars()
        .any(|c| matches!(c, ' ' | '"' | '\'' | '\\' | '$' | '`'));
    if !needs_quote {
        return raw.into_owned();
    }
    let escaped = raw
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$");
    format!("\"{escaped}\"")
}

/// 是否已存在非本程序写入的同名入口（系统安装、AppImageLauncher 等）。
fn installed_entry_exists(app_id: &str, data_home: &Path) -> bool {
    let file_name = format!("{app_id}.desktop");
    let mut dirs = vec![data_home.join("applications")];

    let data_dirs = std::env::var_os("XDG_DATA_DIRS")
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs.extend(std::env::split_paths(&data_dirs).map(|dir| dir.join("applications")));

    dirs.iter().any(|dir| {
        fs::read_to_string(dir.join(&file_name))
            .map(|content| !content.contains(MARKER))
            .unwrap_or(false)
    })
}

/// 安装 hicolor 图标，供 `Icon=<app_id>` 解析。
fn write_icon(data_home: &Path, app_id: &str) {
    let icon_path = data_home
        .join("icons/hicolor/512x512/apps")
        .join(format!("{app_id}.png"));

    if fs::read(&icon_path).map(|existing| existing == ICON).unwrap_or(false) {
        return;
    }

    if let Some(parent) = icon_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&icon_path, ICON) {
        crate::log_warn!("写入桌面图标失败: {}", e);
    }
}
