use crate::{log_info, log_warn};
use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// 博客更新日志地址前缀。
const BLOG_POSTS_BASE: &str = "https://smtc2web.org/posts";
/// 单个博客页面允许读取的最大 HTML 大小。
const MAX_CHANGELOG_HTML_BYTES: usize = 512 * 1024;
/// 展示给前端的更新日志最大字符数，避免超长文章撑爆弹窗。
const MAX_CHANGELOG_CHARS: usize = 20_000;

/// 暂存 `check` 得到的待安装更新，供 `start_update` 使用。
#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

/// 返回给前端的检查结果
#[derive(Debug, Serialize, Clone)]
pub struct UpdateCheckResult {
    /// 是否有可用更新
    pub has_update: bool,
    /// 当前版本
    pub current_version: String,
    /// 最新版本
    pub latest_version: String,
    /// 更新说明
    pub notes: Option<String>,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 更新下载进度事件负载
#[derive(Debug, Serialize, Clone)]
pub struct UpdateProgress {
    /// 已下载字节数
    pub downloaded: u64,
    /// 总字节数（服务器未提供时为 null）
    pub total: Option<u64>,
    /// 百分比（0–100，总大小未知时为 null）
    pub percent: Option<f64>,
}

/// 手动检查更新。更新源、签名校验均由 Tauri updater 插件管理。
///
/// 发现新版本后，会从官方博客拉取对应版本的更新日志：
/// 中文（简体 / 繁体）使用中文文章，其余语言使用英文文章。
/// 博客不可用时回退到更新包自带的 `body`。
#[tauri::command]
pub async fn check_update(
    app: AppHandle,
    locale: Option<String>,
) -> Result<UpdateCheckResult, String> {
    let current = app.package_info().version.to_string();
    let updater = app.updater().map_err(|e| format!("更新器不可用: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            log_info!(
                "发现新版本: {} -> {}",
                update.current_version,
                update.version
            );

            let use_chinese = locale.as_deref().unwrap_or_default().starts_with("zh");
            let blog_notes = fetch_blog_changelog(&update.version, use_chinese).await;
            if blog_notes.is_none() {
                log_warn!(
                    "未能从博客获取 v{} 的更新日志，回退到更新包说明",
                    update.version
                );
            }

            let result = UpdateCheckResult {
                has_update: true,
                current_version: update.current_version.clone(),
                latest_version: update.version.clone(),
                notes: blog_notes.or_else(|| update.body.clone()),
                error: None,
            };
            *app.state::<PendingUpdate>().0.lock().unwrap() = Some(update);
            Ok(result)
        }
        Ok(None) => {
            *app.state::<PendingUpdate>().0.lock().unwrap() = None;
            Ok(UpdateCheckResult {
                has_update: false,
                current_version: current.clone(),
                latest_version: current,
                notes: None,
                error: None,
            })
        }
        Err(e) => {
            *app.state::<PendingUpdate>().0.lock().unwrap() = None;
            Ok(UpdateCheckResult {
                has_update: false,
                current_version: current,
                latest_version: String::new(),
                notes: None,
                error: Some(format!("检查更新失败: {}", e)),
            })
        }
    }
}

/// 下载并安装待处理的更新，过程中通过 `update-progress` 事件上报进度。
#[tauri::command]
pub async fn start_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .state::<PendingUpdate>()
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or("没有待安装的更新，请先检查更新")?;

    log_info!("开始下载并安装更新: {}", update.version);

    let downloaded = std::sync::Arc::new(AtomicU64::new(0));
    let downloaded_for_chunk = downloaded.clone();
    let app_for_progress = app.clone();

    update
        .download_and_install(
            move |chunk_length, content_length| {
                let current = downloaded_for_chunk
                    .fetch_add(chunk_length as u64, Ordering::Relaxed)
                    + chunk_length as u64;
                let percent = content_length
                    .filter(|total| *total > 0)
                    .map(|total| (current as f64 / total as f64 * 100.0).min(100.0));
                let _ = app_for_progress.emit(
                    "update-progress",
                    UpdateProgress {
                        downloaded: current,
                        total: content_length,
                        percent,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| format!("安装更新失败: {}", e))?;

    let _ = app.emit(
        "update-progress",
        UpdateProgress {
            downloaded: downloaded.load(Ordering::Relaxed),
            total: None,
            percent: Some(100.0),
        },
    );

    app.restart();
}

/* ------------------------- 博客更新日志 ------------------------- */

/// 将版本号转换为博客文章 slug（例如 `0.7.1` -> `v0.7.1`）。
///
/// 只接受版本号中常见的 ASCII 字符，避免把不可信内容拼进 URL 路径。
fn blog_slug(version: &str) -> Option<String> {
    let version = version.trim().trim_start_matches(['v', 'V']);
    if version.is_empty() || version.len() > 64 {
        return None;
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return None;
    }
    Some(format!("v{version}"))
}

/// 从博客抓取对应版本的更新日志并转换为可展示的纯文本。
///
/// 中文（简体 / 繁体）访问中文文章，其余语言访问 `.en` 英文文章。
/// 网络失败、文章不存在或页面结构变化时返回 `None`，由调用方回退到更新包说明。
async fn fetch_blog_changelog(version: &str, chinese: bool) -> Option<String> {
    let slug = blog_slug(version)?;
    let url = if chinese {
        format!("{BLOG_POSTS_BASE}/{slug}")
    } else {
        format!("{BLOG_POSTS_BASE}/{slug}.en")
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent(concat!("smtc2web/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => {
            log_warn!("请求博客更新日志失败 ({}): {}", url, e);
            return None;
        }
    };

    if !response.status().is_success() {
        log_warn!("博客更新日志不存在 ({}): HTTP {}", url, response.status());
        return None;
    }
    if response
        .content_length()
        .is_some_and(|len| len as usize > MAX_CHANGELOG_HTML_BYTES)
    {
        log_warn!("博客更新日志页面过大，已忽略: {}", url);
        return None;
    }

    let html = response.text().await.ok()?;
    if html.len() > MAX_CHANGELOG_HTML_BYTES {
        log_warn!("博客更新日志页面过大，已忽略: {}", url);
        return None;
    }

    let content = extract_main_content(&html)?;
    let mut notes = String::new();
    if let Some(title) = extract_post_title(&html) {
        notes.push_str(&title);
        notes.push_str("\n\n");
    }
    notes.push_str(&html_to_text(content));

    let notes = notes.trim().to_string();
    if notes.is_empty() {
        return None;
    }
    Some(truncate_changelog(notes))
}

/// 提取 VitePress 文章正文 `<main class="main">...</main>`。
fn extract_main_content(html: &str) -> Option<&str> {
    const MARKER: &str = "<main class=\"main\"";
    let start = html.find(MARKER)?;
    let content_start = html[start..].find('>')? + start + 1;
    let content_end = html[content_start..].find("</main>")? + content_start;
    Some(&html[content_start..content_end])
}

/// 提取文章标题 `<h1 class="post-title">...</h1>`。
fn extract_post_title(html: &str) -> Option<String> {
    const MARKER: &str = "class=\"post-title\"";
    let start = html.find(MARKER)?;
    let content_start = html[start..].find('>')? + start + 1;
    let content_end = html[content_start..].find("</h1>")? + content_start;
    let title = html_to_text(&html[content_start..content_end]);
    (!title.is_empty()).then_some(title)
}

/// 将受限 HTML 片段转换为纯文本，保留段落、标题与列表结构。
///
/// 返回值会经过实体解码与空白归一化，前端只需用 `white-space: pre-line`
/// 展示即可，不需要 `v-html`，避免渲染远程 HTML 带来的 XSS 风险。
fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        let Some(gt) = rest[lt..].find('>') else {
            out.push_str(&rest[lt..]);
            rest = "";
            break;
        };
        append_tag_markup(&rest[lt + 1..lt + gt], &mut out);
        rest = &rest[lt + gt + 1..];
    }
    out.push_str(rest);

    normalize_text(&decode_entities(&out))
}

fn append_tag_markup(tag: &str, out: &mut String) {
    let tag = tag.trim();
    if tag.is_empty() || tag.starts_with('!') || tag.starts_with('?') {
        return;
    }

    let closing = tag.starts_with('/');
    let name = tag
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_ascii_lowercase();

    match name.as_str() {
        "br" => out.push('\n'),
        "hr" => {
            ensure_blank_line(out);
            out.push_str("────────");
            out.push('\n');
        }
        "li" => {
            if closing {
                out.push('\n');
            } else {
                ensure_newline(out);
                out.push_str("• ");
            }
        }
        "p" => {
            if closing {
                out.push('\n');
            } else {
                ensure_blank_line(out);
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            if closing {
                out.push('\n');
            } else {
                ensure_blank_line(out);
                out.push('▍');
            }
        }
        "div" | "section" | "article" | "blockquote" | "ul" | "ol" | "table" | "thead"
        | "tbody" | "tr" | "pre" => {
            if closing {
                out.push('\n');
            } else {
                ensure_newline(out);
            }
        }
        // 表格单元格之间用空格分隔，保证纯文本可读。
        "td" | "th" if !closing => out.push_str("  "),
        _ => {}
    }
}

fn ensure_newline(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

fn ensure_blank_line(out: &mut String) {
    ensure_newline(out);
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
}

fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp..];
        if let Some(end) = after.find(';')
            && let Some(ch) = decode_entity(&after[1..end])
        {
            out.push(ch);
            rest = &after[end + 1..];
            continue;
        }
        out.push('&');
        rest = &after[1..];
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => {
            let code = if let Some(hex) = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
            {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                let decimal = entity.strip_prefix('#')?;
                decimal.parse::<u32>().ok()?
            };
            char::from_u32(code)
        }
    }
}

/// 按行裁剪空白：保留段落之间的单个空行，去掉零宽字符和多余缩进。
fn normalize_text(input: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    for raw in input.lines() {
        let cleaned: String = raw
            .chars()
            .filter(|c| !matches!(c, '\u{200B}' | '\u{FEFF}'))
            .collect();
        let line = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        if line.is_empty() {
            if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
                lines.push(String::new());
            }
        } else {
            lines.push(line);
        }
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn truncate_changelog(mut text: String) -> String {
    if text.chars().count() > MAX_CHANGELOG_CHARS {
        text = text.chars().take(MAX_CHANGELOG_CHARS).collect();
        text.push('…');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_update_default_is_empty() {
        let pending = PendingUpdate::default();
        assert!(pending.0.lock().unwrap().is_none());
    }

    #[test]
    fn blog_slug_accepts_versions_and_rejects_paths() {
        assert_eq!(blog_slug("0.7.1").as_deref(), Some("v0.7.1"));
        assert_eq!(blog_slug("v0.7.1").as_deref(), Some("v0.7.1"));
        assert_eq!(
            blog_slug("0.1.9-nightly1").as_deref(),
            Some("v0.1.9-nightly1")
        );
        assert_eq!(blog_slug("../secret"), None);
        assert_eq!(blog_slug("0.7.1?x=1"), None);
        assert_eq!(blog_slug(""), None);
    }

    #[test]
    fn extract_post_title_and_main_content() {
        let html = r#"<h1 class="post-title">v1.2.3 &amp; fixes</h1>
        <main class="main" data-v-1><p>Hello <strong>world</strong>.</p></main>"#;
        assert_eq!(extract_post_title(html).as_deref(), Some("v1.2.3 & fixes"));
        assert_eq!(
            extract_main_content(html),
            Some("<p>Hello <strong>world</strong>.</p>")
        );
    }

    #[test]
    fn html_to_text_keeps_structure_and_decodes_entities() {
        let html = r#"<p>First&nbsp;line &amp; more.</p><h2>Fix</h2><ul><li>A</li><li>B</li></ul><p>Use &lt;style&gt; and &#39;quotes&#39;.</p>"#;
        let text = html_to_text(html);

        assert!(text.contains("First line & more."));
        assert!(text.contains("▍Fix"));
        assert!(text.contains("• A"));
        assert!(text.contains("• B"));
        assert!(text.contains("Use <style> and 'quotes'."));
    }

    #[test]
    fn truncate_changelog_respects_char_limit() {
        let text = "a".repeat(MAX_CHANGELOG_CHARS + 10);
        let truncated = truncate_changelog(text);
        assert_eq!(truncated.chars().count(), MAX_CHANGELOG_CHARS + 1);
        assert!(truncated.ends_with('…'));
    }
}
