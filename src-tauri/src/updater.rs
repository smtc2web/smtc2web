use crate::log_info;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
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

/// 下载并安装待处理的更新。
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
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("安装更新失败: {}", e))?;

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
