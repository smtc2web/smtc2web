use rust_embed::RustEmbed;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use warp::path::Tail;

#[derive(RustEmbed)]
#[folder = "frontend"]
pub struct DefaultTheme;

/// 当前活动主题的磁盘路径。
///
/// 空路径表示使用内嵌的默认主题。服务器启动后无需重启即可通过更新该路径
/// 切换到另一个主题，从而支持"切换主题后自动刷新网页"。
static ACTIVE_THEME_PATH: LazyLock<RwLock<PathBuf>> = LazyLock::new(|| RwLock::new(PathBuf::new()));

/// 更新当前活动主题路径。
pub fn set_active_theme_path(path: PathBuf) {
    if let Ok(mut guard) = ACTIVE_THEME_PATH.write() {
        *guard = path;
    }
}

/// 读取当前活动主题路径。
fn active_theme_path() -> PathBuf {
    ACTIVE_THEME_PATH
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// 注入到每个 HTML 页面中的热重载脚本。
///
/// 监听 `/__reload` SSE 事件，收到 `reload` 后刷新页面；同时保留断线重连
/// 兜底逻辑，避免主题切换瞬间服务器短暂不可用导致页面停留在错误状态。
const RELOAD_SCRIPT: &str = r#"<script>
(function () {
  if (window.__smtcReloadBound) return;
  window.__smtcReloadBound = true;
  var reloaded = false;
  function doReload() {
    if (reloaded) return;
    reloaded = true;
    location.reload();
  }
  try {
    var es = new EventSource('/__reload');
    var opened = false;
    function handleReload() { es.close(); doReload(); }
    es.onopen = function () { opened = true; };
    es.addEventListener('reload', handleReload);
    es.onmessage = handleReload;
    es.onerror = function () {
      // 端点从未连接成功时不做任何事，避免损坏的端点导致刷新死循环。
      if (!opened) return;
      es.close();
      // 服务器暂时不可用时，等待恢复后刷新（仅在曾经连接成功后启用）。
      var tries = 0;
      var timer = setInterval(function () {
        tries++;
        fetch('/api/now', { cache: 'no-store' }).then(function (r) {
          if (r.ok) { clearInterval(timer); doReload(); }
        }).catch(function () {
          if (tries > 40) { clearInterval(timer); doReload(); }
        });
      }, 250);
    };
  } catch (e) {}
})();
</script>"#;

const BODY_CLOSE_TAG: &str = "</body>";

/// 将热重载脚本注入到 HTML 的 `</body>` 之前。
fn inject_reload_script(html: &str) -> String {
    // 使用 ASCII 小写化，保证字节长度不变，`pos` 一定是合法的字符边界，
    // 同时兼容 `</BODY>` 这类大小写写法。
    let lower = html.to_ascii_lowercase();
    if let Some(pos) = lower.rfind(BODY_CLOSE_TAG) {
        let mut s = html.to_string();
        s.insert_str(pos, RELOAD_SCRIPT);
        s
    } else {
        format!("{}{}", html, RELOAD_SCRIPT)
    }
}

/// 提供主题静态文件。
///
/// 优先从当前活动主题目录读取，找不到时回退到内嵌的默认主题。
/// HTML 响应会被注入热重载脚本，以便在切换主题后自动刷新页面。
pub async fn serve_theme_file(tail: Tail) -> Result<impl warp::Reply, warp::Rejection> {
    let path = tail.as_str();
    let path = if path.is_empty() { "index.html" } else { path };

    let theme_path = active_theme_path();
    let has_custom_theme =
        !theme_path.to_string_lossy().is_empty() && theme_path.components().next().is_some();

    // 从自定义主题目录读取（如果存在且通过目录遍历校验）
    if has_custom_theme {
        let custom_path = theme_path.join(path);
        if let (Ok(base_dir), Ok(resolved_path)) = (
            std::fs::canonicalize(&theme_path),
            std::fs::canonicalize(&custom_path),
        ) && resolved_path.starts_with(&base_dir)
            && let Ok(content) = std::fs::read(&resolved_path)
        {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return Ok(make_response(content, mime.to_string()));
        }
    }

    // 回退到内嵌的默认主题
    match DefaultTheme::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Ok(make_response(content.data.to_vec(), mime.to_string()))
        }
        None => Err(warp::reject::not_found()),
    }
}

fn make_response(content: Vec<u8>, mime: String) -> impl warp::Reply {
    let is_html = mime.starts_with("text/html");
    let body = if is_html {
        inject_reload_script(&String::from_utf8_lossy(&content)).into_bytes()
    } else {
        content
    };
    // HTML 不做缓存：切换主题后刷新即可拿到新主题的页面。
    let cache_control = if is_html { "no-store" } else { "no-cache" };
    warp::reply::with_header(
        warp::reply::with_header(body, "content-type", mime),
        "cache-control",
        cache_control,
    )
}
