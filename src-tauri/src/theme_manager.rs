use crate::theme::DefaultTheme;
use crate::{log_debug, log_error, log_info, log_warn};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use zip::ZipArchive;

/// 单个主题文件的最大解压大小（32 MiB）。
const MAX_THEME_FILE_SIZE: u64 = 32 * 1024 * 1024;
/// 整个主题包的最大解压大小（128 MiB），用于抵御 Zip 炸弹。
const MAX_THEME_TOTAL_SIZE: u64 = 128 * 1024 * 1024;
/// 主题包允许的最大条目数。
const MAX_THEME_ENTRIES: usize = 4096;
/// 主题文件夹名 / 主题名的最大长度。
const MAX_THEME_NAME_LEN: usize = 100;
/// 主题截图的最大读取大小（16 MiB）。
const MAX_SCREENSHOT_SIZE: u64 = 16 * 1024 * 1024;

/// 校验主题文件夹名：必须是单层目录名，且不能包含路径分隔符、盘符、
/// 控制字符或 Windows 保留设备名，避免 `join` 后逃逸出主题目录。
fn validate_theme_folder(name: &str) -> io::Result<()> {
    let invalid = name.is_empty()
        || name.len() > MAX_THEME_NAME_LEN
        || name == "."
        || name == ".."
        || name == "default"
        || name.ends_with([' ', '.'])
        || name.chars().any(|c| {
            c.is_control() || !(c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        });

    if invalid || is_windows_reserved_name(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("非法的主题名称: '{}'", name),
        ));
    }

    Ok(())
}

/// Windows 保留设备名（CON、PRN、AUX、NUL、COM1-9、LPT1-9），
/// 即使带扩展名也不允许作为文件名。
fn is_windows_reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    let stem = stem.trim_end_matches([' ', '.']).to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

/// 校验主题来源 URL：仅允许 https，拒绝凭据、`ext::` 等本地命令协议，
/// 防止 `git clone` 的参数注入与命令执行。
fn validate_repo_url(repo_url: &str) -> io::Result<Url> {
    let parsed = Url::parse(repo_url.trim()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "无效的仓库地址，只支持 https:// 链接",
        )
    })?;

    if parsed.scheme() != "https" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "只允许使用 https:// 仓库地址",
        ));
    }
    if parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "仓库地址不能包含凭据或缺少主机名",
        ));
    }

    Ok(parsed)
}

/// 校验 Git 分支名，避免以 `-` 开头的参数注入以及 `..` 路径穿越。
fn validate_git_branch(branch: &str) -> io::Result<()> {
    if branch.is_empty() {
        return Ok(());
    }

    let invalid = branch.len() > 200
        || branch.starts_with('-')
        || branch.starts_with('/')
        || branch.ends_with('/')
        || branch.contains("..")
        || branch.chars().any(|c| {
            c.is_control() || !(c.is_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'))
        });

    if invalid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("非法的分支名: '{}'", branch),
        ));
    }
    Ok(())
}

/// 校验压缩包内的单个路径组件，拒绝 ADS（`:`）、保留设备名以及
/// Windows 会静默截断的尾随空格 / 点。
fn is_safe_path_component(component: &std::ffi::OsStr) -> bool {
    let Some(component) = component.to_str() else {
        return false;
    };
    !component.is_empty()
        && component != "."
        && component != ".."
        && !component.ends_with([' ', '.'])
        && !component.contains([':', '\\', '/', '\0'])
        && !is_windows_reserved_name(component)
}

/// 将压缩包内路径转换为相对于主题根目录的安全路径。
///
/// 只接受普通文件/目录组件；`..`、绝对路径、盘符前缀、UNC 路径都会被拒绝。
fn safe_relative_path(normalized: &str, root: &str) -> Option<PathBuf> {
    let rest = normalized.strip_prefix(root)?.strip_prefix('/')?;
    if rest.is_empty() {
        return Some(PathBuf::new());
    }

    let mut out = PathBuf::new();
    for component in Path::new(rest).components() {
        match component {
            Component::Normal(name) if is_safe_path_component(name) => out.push(name),
            _ => return None,
        }
    }
    Some(out)
}

/// 生成同目录下的唯一临时目录名。
fn unique_staging_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!(".staging-{}-{}", std::process::id(), nanos)
}

/// 创建隐藏控制台窗口的 git 命令
fn git_command() -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("git");
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("git")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub folder_name: String,
    pub author: String,
    pub version: String,
    pub screenshot_path: String,
    pub is_default: bool,
}

pub struct ThemeManager;

impl ThemeManager {
    /// 获取主题目录路径
    pub fn get_themes_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("smtc2web");
        path.push("theme");
        path
    }

    /// 确保主题目录存在
    pub fn ensure_themes_dir() -> io::Result<()> {
        let themes_dir = Self::get_themes_dir();
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir)?;
        }
        Ok(())
    }

    /// 获取默认主题信息
    fn get_default_theme_info() -> ThemeInfo {
        // 从嵌入的资源中读取 theme.toml
        let theme_toml_content = DefaultTheme::get("theme.toml")
            .map(|c| String::from_utf8_lossy(&c.data).to_string())
            .unwrap_or_else(|| {
                // 如果读取失败，使用硬编码的默认值
                r#"[smtc2web.theme]
name = "默认主题"
version = "0.1.1"
author = "AkarinLiu"
screenshot = "screenshot.png"
"#
                .to_string()
            });

        // 解析 TOML
        let config: toml::Value =
            toml::from_str(&theme_toml_content).expect("Failed to parse embedded theme.toml");

        // 尝试从 [smtc2web.theme] 节读取配置
        let theme_section = config.get("smtc2web").and_then(|s| s.get("theme"));
        let theme_config = theme_section.unwrap_or(&config);

        let name = theme_config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("默认主题")
            .to_string();
        let author = theme_config
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("AkarinLiu")
            .to_string();
        let version = theme_config
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.1")
            .to_string();
        let screenshot = theme_config
            .get("screenshot")
            .and_then(|v| v.as_str())
            .unwrap_or("screenshot.png")
            .to_string();

        log_debug!("默认主题截图配置: '{}'", screenshot);

        // 从嵌入的资源中读取截图并转换为 Base64
        let screenshot_path = if let Some(screenshot_data) = DefaultTheme::get(&screenshot) {
            log_debug!(
                "默认主题截图文件找到，大小: {} 字节",
                screenshot_data.data.len()
            );
            // 根据文件扩展名判断 MIME 类型
            let mime = if screenshot.ends_with(".png") {
                "image/png"
            } else if screenshot.ends_with(".jpg") || screenshot.ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "image/png"
            };
            let result = format!(
                "data:{};base64,{}",
                mime,
                STANDARD.encode(&screenshot_data.data)
            );
            log_debug!("默认主题 Base64 编码完成，长度: {}", result.len());
            result
        } else {
            log_warn!("默认主题截图文件 '{}' 未找到", screenshot);
            // 如果找不到截图，返回空字符串
            String::new()
        };

        ThemeInfo {
            name,
            folder_name: "default".to_string(),
            author,
            version,
            screenshot_path,
            is_default: true,
        }
    }

    /// 获取主题配置文件路径
    fn get_theme_config_path(theme_folder: &str) -> PathBuf {
        let mut path = Self::get_themes_dir();
        path.push(theme_folder);
        path.push("theme.toml");
        path
    }

    /// 解析主题配置
    fn parse_theme_config(theme_folder: &str) -> Option<ThemeInfo> {
        if validate_theme_folder(theme_folder).is_err() {
            log_warn!("跳过非法主题目录: {}", theme_folder);
            return None;
        }

        let config_path = Self::get_theme_config_path(theme_folder);

        if !config_path.exists() {
            log_debug!("  配置文件不存在: {:?}", config_path);
            return None;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                log_error!("  读取配置文件失败: {}", e);
                return None;
            }
        };

        let config: toml::Value = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                log_error!("  解析 TOML 失败: {}", e);
                return None;
            }
        };

        // 尝试从 [smtc2web.theme] 节读取配置
        let theme_section = config.get("smtc2web").and_then(|s| s.get("theme"));

        // 如果找不到嵌套节，尝试直接从顶层读取（向后兼容）
        let theme_config = theme_section.unwrap_or(&config);

        let name = theme_config.get("name")?.as_str()?.to_string();
        let author = theme_config.get("author")?.as_str()?.to_string();
        let version = theme_config.get("version")?.as_str()?.to_string();

        // 截图字段是可选的，使用 unwrap_or_default 处理缺失的情况
        let screenshot = theme_config
            .get("screenshot")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        log_debug!("  主题 '{}' 截图配置: '{}'", name, screenshot);

        // 验证截图路径：禁止网络 URL，只允许本地相对路径
        let screenshot_path = if screenshot.is_empty() {
            log_debug!("  主题 '{}' 没有配置截图", name);
            String::new()
        } else if screenshot.starts_with("http://") || screenshot.starts_with("https://") {
            log_warn!("  主题 '{}' 的截图使用了网络 URL，已忽略", name);
            String::new()
        } else {
            // 只允许读取主题目录内的本地图片，防止恶意 theme.toml 通过
            // `screenshot = "../../../secret"` 或绝对路径读取任意文件。
            let theme_dir = Self::get_themes_dir().join(theme_folder);
            let lower = screenshot.to_ascii_lowercase();
            let mime = if lower.ends_with(".png") {
                Some("image/png")
            } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                Some("image/jpeg")
            } else if lower.ends_with(".webp") {
                Some("image/webp")
            } else if lower.ends_with(".gif") {
                Some("image/gif")
            } else {
                None
            };

            let resolved = fs::canonicalize(&theme_dir).ok().and_then(|base| {
                let path = fs::canonicalize(theme_dir.join(&screenshot)).ok()?;
                Some((base, path))
            });

            match (mime, resolved) {
                (Some(mime), Some((base, path))) if path.starts_with(&base) && path.is_file() => {
                    match fs::metadata(&path) {
                        Ok(meta) if meta.len() > MAX_SCREENSHOT_SIZE => {
                            log_warn!(
                                "  主题 '{}' 的截图文件过大（{} 字节），已忽略",
                                name,
                                meta.len()
                            );
                            String::new()
                        }
                        Ok(_) => match fs::read(&path) {
                            Ok(data) if (data.len() as u64) <= MAX_SCREENSHOT_SIZE => {
                                format!("data:{};base64,{}", mime, STANDARD.encode(&data))
                            }
                            Ok(_) => {
                                log_warn!("主题 '{}' 的截图文件过大，已忽略", name);
                                String::new()
                            }
                            Err(e) => {
                                log_warn!("读取主题 '{}' 的截图文件失败: {}", name, e);
                                String::new()
                            }
                        },
                        Err(e) => {
                            log_warn!("读取主题 '{}' 的截图文件信息失败: {}", name, e);
                            String::new()
                        }
                    }
                }
                _ => {
                    log_warn!(
                        "  主题 '{}' 的截图路径无效或越出主题目录，已忽略: '{}'",
                        name,
                        screenshot
                    );
                    String::new()
                }
            }
        };

        Some(ThemeInfo {
            name,
            folder_name: theme_folder.to_string(),
            author,
            version,
            screenshot_path,
            is_default: false,
        })
    }

    /// 扫描所有主题
    pub fn scan_themes() -> io::Result<Vec<ThemeInfo>> {
        Self::ensure_themes_dir()?;

        let themes_dir = Self::get_themes_dir();
        let mut themes = Vec::new();

        // 首先添加默认主题
        let default_theme = Self::get_default_theme_info();
        themes.push(default_theme);

        log_info!("扫描主题目录: {:?}", themes_dir);

        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && let Some(folder_name) = path.file_name()
                {
                    let folder_name = folder_name.to_string_lossy().to_string();

                    // 跳过 default 文件夹（默认主题使用内置的）
                    if folder_name == "default" {
                        log_debug!("发现文件夹: {} (跳过，使用内置默认主题)", folder_name);
                        continue;
                    }

                    log_debug!("发现文件夹: {}", folder_name);

                    let config_path = Self::get_theme_config_path(&folder_name);
                    log_debug!("  配置文件路径: {:?}", config_path);
                    log_debug!("  配置文件存在: {}", config_path.exists());

                    if let Some(theme_info) = Self::parse_theme_config(&folder_name) {
                        log_info!("  成功解析主题: {}", theme_info.name);
                        themes.push(theme_info);
                    } else {
                        log_debug!("  解析主题失败，跳过");
                    }
                }
            }
        }

        log_info!("扫描完成，共找到 {} 个主题", themes.len());
        Ok(themes)
    }

    /// 解压主题压缩包
    pub fn extract_theme(zip_path: &Path) -> io::Result<String> {
        let theme_name = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "无效的文件名"))?
            .to_string();

        validate_theme_folder(&theme_name)?;
        Self::extract_theme_as(zip_path, &theme_name)
    }

    /// 以指定名称解压主题压缩包。
    ///
    /// 解压前会完整校验所有条目路径，解压到临时目录后再原子替换，
    /// 避免路径穿越、符号链接、Zip 炸弹以及失败时留下半成品。
    pub fn extract_theme_as(zip_path: &Path, theme_name: &str) -> io::Result<String> {
        validate_theme_folder(theme_name)?;
        Self::ensure_themes_dir()?;

        let file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        if archive.len() > MAX_THEME_ENTRIES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("此主题无效：压缩包条目过多（上限 {}）", MAX_THEME_ENTRIES),
            ));
        }

        // 验证 1：所有条目必须位于同一个根文件夹下，且路径安全。
        let mut root_folder: Option<String> = None;
        let mut seen_paths = std::collections::HashSet::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            let normalized = entry.name()?.replace('\\', "/");
            let Some((root, _)) = normalized.split_once('/') else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：压缩包内必须有且只有一个根文件夹！",
                ));
            };

            if !is_safe_path_component(std::ffi::OsStr::new(root)) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：压缩包根目录名称不安全！",
                ));
            }
            let Some(relative_path) = safe_relative_path(&normalized, root) else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：压缩包包含不安全的路径！",
                ));
            };
            if !relative_path.as_os_str().is_empty() && !seen_paths.insert(relative_path) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：压缩包包含重复的路径！",
                ));
            }

            match &root_folder {
                None => root_folder = Some(root.to_string()),
                Some(existing) if existing == root => {}
                Some(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "此主题无效：压缩包内只能有一个根文件夹！",
                    ));
                }
            }
        }

        let root_folder = root_folder.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "此主题无效：压缩包内必须有且只有一个根文件夹！",
            )
        })?;

        // 验证 2：根文件夹内必须包含 theme.toml（限制读取大小）。
        let expected_path = format!("{}/theme.toml", root_folder);
        let mut theme_toml_index = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.name()?.replace('\\', "/") == expected_path {
                theme_toml_index = Some(i);
                break;
            }
        }

        let theme_toml_index = theme_toml_index.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "此主题无效：根文件夹内缺少 theme.toml 文件！",
            )
        })?;

        // 验证 3：theme.toml 必须包含 [smtc2web.theme] 配置节。
        {
            let mut file = archive.by_index(theme_toml_index)?;
            if file.size() > 1024 * 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：theme.toml 过大！",
                ));
            }
            let mut content = String::new();
            file.by_ref()
                .take(1024 * 1024 + 1)
                .read_to_string(&mut content)?;
            if content.len() > 1024 * 1024 || !content.contains("[smtc2web.theme]") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "此主题无效：theme.toml 缺少 [smtc2web.theme] 配置节！",
                ));
            }
        }

        // 解压到临时目录；全部成功后再替换正式目录。
        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(theme_name);
        let staging_dir = themes_dir.join(unique_staging_name());
        // 使用 create_dir 而非 create_dir_all：若同名临时目录已存在则直接失败，
        // 避免复用他人预先创建的目录。
        fs::create_dir(&staging_dir)?;

        let extract_result = (|| -> io::Result<()> {
            let file = fs::File::open(zip_path)?;
            let mut archive = ZipArchive::new(file)?;
            let mut total_written: u64 = 0;

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let normalized = entry.name()?.replace('\\', "/");
                let Some(relative_path) = safe_relative_path(&normalized, &root_folder) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "此主题无效：压缩包包含不安全的路径！",
                    ));
                };
                if relative_path.as_os_str().is_empty() {
                    continue;
                }

                let out_path = staging_dir.join(&relative_path);

                if entry.is_dir() {
                    fs::create_dir_all(&out_path)?;
                    continue;
                }

                // 拒绝符号链接，避免解压后通过链接读写主题目录之外的文件。
                if entry
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "此主题无效：压缩包内不允许符号链接！",
                    ));
                }

                if entry.size() > MAX_THEME_FILE_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "此主题无效：压缩包内文件过大！",
                    ));
                }

                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&out_path)?;
                let copied = io::copy(
                    &mut entry.by_ref().take(MAX_THEME_FILE_SIZE + 1),
                    &mut outfile,
                )?;
                if copied > MAX_THEME_FILE_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "此主题无效：压缩包内文件过大！",
                    ));
                }

                total_written = total_written.checked_add(copied).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "此主题无效：解压大小溢出！")
                })?;
                if total_written > MAX_THEME_TOTAL_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "此主题无效：解压后总大小超出限制！",
                    ));
                }
            }
            Ok(())
        })();

        if let Err(e) = extract_result {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(e);
        }

        // 原子替换旧主题（同文件系统内 rename）。
        if theme_dir.exists() {
            fs::remove_dir_all(&theme_dir)?;
        }
        if let Err(e) = fs::rename(&staging_dir, &theme_dir) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(e);
        }

        Ok(theme_name.to_string())
    }

    /// 删除主题
    pub fn delete_theme(theme_folder: &str) -> io::Result<()> {
        validate_theme_folder(theme_folder)?;

        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(theme_folder);

        if theme_dir.exists() {
            fs::remove_dir_all(&theme_dir)?;
        }

        Ok(())
    }

    /// 获取主题的服务路径
    pub fn get_theme_server_path(theme_folder: &str) -> PathBuf {
        // 默认主题使用空路径（表示使用内置主题）；非法名称同样回退到内置主题。
        if theme_folder == "default" || validate_theme_folder(theme_folder).is_err() {
            return PathBuf::new();
        }
        Self::get_themes_dir().join(theme_folder)
    }

    /// 从 Git 仓库安装主题
    pub fn install_from_git(repo_url: &str, branch: &str) -> io::Result<String> {
        Self::ensure_themes_dir()?;
        let parsed = validate_repo_url(repo_url)?;
        validate_git_branch(branch)?;

        // 从 URL 路径推导文件夹名：取最后一段，去掉 .git 后缀。
        let last_segment = parsed
            .path_segments()
            .and_then(|mut segments| segments.rfind(|s| !s.is_empty()))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "无法从仓库地址推导主题名称")
            })?;
        let folder_name = last_segment
            .strip_suffix(".git")
            .unwrap_or(last_segment)
            .to_string();

        validate_theme_folder(&folder_name)?;

        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(&folder_name);

        // 如果目录已存在，先删除
        if theme_dir.exists() {
            fs::remove_dir_all(&theme_dir)?;
        }

        // 构建 git clone 命令
        let mut cmd = git_command();
        cmd.arg("clone")
            .arg("--single-branch")
            .arg("--depth")
            .arg("1");

        if !branch.is_empty() {
            cmd.arg("--branch").arg(branch);
        }

        // `--` 终止选项解析，避免仓库地址被当作 git 参数；
        // 同时通过 GIT_ALLOW_PROTOCOL 禁止 ext:: / file:// 等本地协议。
        cmd.arg("--")
            .arg(repo_url)
            .arg(&theme_dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "https");

        log_info!(
            "正在从 Git 克隆主题: {} (分支: {}) -> {:?}",
            parsed,
            if branch.is_empty() { "默认" } else { branch },
            theme_dir
        );

        let output = cmd.output().map_err(|e| {
            io::Error::other(format!("执行 git clone 失败，请确保已安装 Git: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 克隆失败时清理可能创建的目录
            let _ = fs::remove_dir_all(&theme_dir);
            return Err(io::Error::other(format!(
                "git clone 失败: {}",
                stderr.trim()
            )));
        }

        // 验证 theme.toml 存在
        let config_path = theme_dir.join("theme.toml");
        if !config_path.exists() {
            let _ = fs::remove_dir_all(&theme_dir);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "此主题无效：仓库根目录缺少 theme.toml 文件！",
            ));
        }

        // 验证 theme.toml 包含 [smtc2web.theme]
        let content = fs::read_to_string(&config_path)?;
        if !content.contains("[smtc2web.theme]") {
            let _ = fs::remove_dir_all(&theme_dir);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "此主题无效：theme.toml 缺少 [smtc2web.theme] 配置节！",
            ));
        }

        log_info!("Git 主题安装成功: {}", folder_name);
        Ok(folder_name)
    }

    /// 更新 Git 主题（git pull）
    pub fn update_git_theme(folder_name: &str) -> io::Result<()> {
        validate_theme_folder(folder_name)?;
        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(folder_name);

        if !theme_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("主题目录不存在: {}", folder_name),
            ));
        }

        log_info!("正在更新 Git 主题: {}", folder_name);

        let output = git_command()
            .arg("-c")
            .arg("protocol.ext.allow=never")
            .arg("-C")
            .arg(&theme_dir)
            .arg("pull")
            .arg("--ff-only")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "https")
            .output()
            .map_err(|e| io::Error::other(format!("执行 git pull 失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::other(format!(
                "git pull 失败: {}",
                stderr.trim()
            )));
        }

        log_info!("Git 主题更新成功: {}", folder_name);
        Ok(())
    }

    /// 检查 Git 主题是否有远程更新
    pub fn check_git_update(folder_name: &str) -> io::Result<bool> {
        validate_theme_folder(folder_name)?;
        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(folder_name);

        if !theme_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("主题目录不存在: {}", folder_name),
            ));
        }

        // git fetch origin
        let fetch_output = git_command()
            .arg("-c")
            .arg("protocol.ext.allow=never")
            .arg("-C")
            .arg(&theme_dir)
            .arg("fetch")
            .arg("origin")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "https")
            .output()
            .map_err(|e| io::Error::other(format!("执行 git fetch 失败: {}", e)))?;

        if !fetch_output.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_output.stderr);
            return Err(io::Error::other(format!(
                "git fetch 失败: {}",
                stderr.trim()
            )));
        }

        // 获取本地 HEAD
        let local_output = git_command()
            .arg("-C")
            .arg(&theme_dir)
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .map_err(|e| io::Error::other(e.to_string()))?;

        // 获取远程 HEAD
        let remote_output = git_command()
            .arg("-C")
            .arg(&theme_dir)
            .arg("rev-parse")
            .arg("@{u}")
            .output()
            .map_err(|e| io::Error::other(e.to_string()))?;

        if !local_output.status.success() || !remote_output.status.success() {
            return Ok(false);
        }

        let local_hash = String::from_utf8_lossy(&local_output.stdout)
            .trim()
            .to_string();
        let remote_hash = String::from_utf8_lossy(&remote_output.stdout)
            .trim()
            .to_string();

        Ok(local_hash != remote_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_theme_names() {
        for name in [
            "",
            ".",
            "..",
            "default",
            "../escape",
            "a/b",
            "a\\b",
            "C:evil",
            "con",
            "COM1",
            "theme.",
            "theme ",
            "bad\u{0}name",
        ] {
            assert!(
                validate_theme_folder(name).is_err(),
                "should reject unsafe theme name: {:?}",
                name
            );
        }
    }

    #[test]
    fn accepts_normal_theme_names() {
        for name in ["my-theme", "\u{4e3b}\u{9898}_01", "theme.v2", "Some Theme"] {
            assert!(
                validate_theme_folder(name).is_ok(),
                "should accept normal theme name: {:?}",
                name
            );
        }
    }

    #[test]
    fn safe_relative_path_blocks_traversal() {
        for path in [
            "root/../evil.txt",
            "root/../../evil.txt",
            "root//evil.txt",
            "root/C:/evil.txt",
            "root/.\u{0}evil",
        ] {
            assert!(
                safe_relative_path(path, "root").is_none(),
                "should reject escaping path: {:?}",
                path
            );
        }
    }

    #[test]
    fn safe_relative_path_keeps_files_under_root() {
        assert_eq!(
            safe_relative_path("root/assets/a.png", "root"),
            Some(PathBuf::from("assets/a.png"))
        );
        assert_eq!(
            safe_relative_path("root/theme.toml", "root"),
            Some(PathBuf::from("theme.toml"))
        );
    }

    #[test]
    fn repo_url_requires_https() {
        assert!(validate_repo_url("https://github.com/user/theme.git").is_ok());
        for url in [
            "http://github.com/user/theme.git",
            "ext::sh -c whoami",
            "file:///etc/passwd",
            "ssh://git@github.com/user/theme.git",
            "https://user:pass@github.com/user/theme.git",
            "-oProxyCommand=calc",
        ] {
            assert!(
                validate_repo_url(url).is_err(),
                "should reject repo URL: {}",
                url
            );
        }
    }

    #[test]
    fn git_branch_rejects_argument_injection() {
        assert!(validate_git_branch("main").is_ok());
        assert!(validate_git_branch("feature/security-fix").is_ok());
        for branch in ["-u", "--upload-pack=calc", "a..b", "/etc", "main/"] {
            assert!(
                validate_git_branch(branch).is_err(),
                "should reject branch: {}",
                branch
            );
        }
    }
}
