use crate::log_info;
use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

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
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<UpdateCheckResult, String> {
    let current = app.package_info().version.to_string();
    let updater = app.updater().map_err(|e| format!("更新器不可用: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => {
            log_info!("发现新版本: {} -> {}", update.current_version, update.version);
            let result = UpdateCheckResult {
                has_update: true,
                current_version: update.current_version.clone(),
                latest_version: update.version.clone(),
                notes: update.body.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_update_default_is_empty() {
        let pending = PendingUpdate::default();
        assert!(pending.0.lock().unwrap().is_none());
    }
}
