use serde::Deserialize;
use std::sync::Mutex;

/// 托盘菜单翻译结构
#[derive(Debug, Deserialize, Clone)]
pub struct TrayTranslations {
    pub show_window: String,
    pub open_web: String,
    pub check_update: String,
    pub quit: String,
}

/// 语言包结构
#[derive(Debug, Deserialize, Clone)]
pub struct Locale {
    pub tray: TrayTranslations,
}

/// 当前语言状态
pub static CURRENT_LOCALE: std::sync::LazyLock<Mutex<String>> =
    std::sync::LazyLock::new(|| Mutex::new("zh-CN".to_string()));

/// 内嵌语言包：编译期打包进二进制，避免安装后找不到 locales 目录
/// 导致托盘菜单语言无法跟随切换。
fn embedded_locale(locale: &str) -> Option<&'static str> {
    Some(match locale {
        "zh-CN" => include_str!("../locales/zh-CN.toml"),
        "zh-TW" => include_str!("../locales/zh-TW.toml"),
        "en" => include_str!("../locales/en.toml"),
        "de-DE" => include_str!("../locales/de-DE.toml"),
        "fr-FR" => include_str!("../locales/fr-FR.toml"),
        "it-IT" => include_str!("../locales/it-IT.toml"),
        "ja-JP" => include_str!("../locales/ja-JP.toml"),
        "ko-KR" => include_str!("../locales/ko-KR.toml"),
        "ko-KP" => include_str!("../locales/ko-KP.toml"),
        "nl-NL" => include_str!("../locales/nl-NL.toml"),
        "ru-RU" => include_str!("../locales/ru-RU.toml"),
        _ => return None,
    })
}

/// 加载指定语言的翻译
pub fn load_locale(locale: &str) -> Option<Locale> {
    toml::from_str(embedded_locale(locale)?).ok()
}

/// 设置当前语言
pub fn set_locale(locale: &str) -> Result<(), String> {
    // 验证语言是否支持
    if !is_locale_supported(locale) {
        return Err(format!("Unsupported locale: {}", locale));
    }

    // 加载语言包（验证能否加载）
    load_locale(locale).ok_or_else(|| format!("Failed to load locale: {}", locale))?;

    // 更新当前语言
    let mut current = CURRENT_LOCALE.lock().unwrap();
    *current = locale.to_string();

    Ok(())
}

/// 获取当前语言
pub fn get_current_locale() -> String {
    CURRENT_LOCALE.lock().unwrap().clone()
}

/// 获取当前语言包
pub fn get_current_locale_data() -> Option<Locale> {
    let locale = get_current_locale();
    load_locale(&locale)
}

/// 检查语言是否支持
pub fn is_locale_supported(locale: &str) -> bool {
    matches!(
        locale,
        "zh-CN"
            | "zh-TW"
            | "en"
            | "de-DE"
            | "fr-FR"
            | "it-IT"
            | "ja-JP"
            | "ko-KR"
            | "ko-KP"
            | "nl-NL"
            | "ru-RU"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每个受支持的语言都必须能解析出非空的托盘翻译，否则托盘会退回中文兜底。
    #[test]
    fn all_supported_locales_load_tray_translations() {
        for code in [
            "zh-CN", "zh-TW", "en", "de-DE", "fr-FR", "it-IT", "ja-JP", "ko-KR", "ko-KP", "nl-NL",
            "ru-RU",
        ] {
            assert!(is_locale_supported(code), "{code} should be supported");
            let data = load_locale(code).unwrap_or_else(|| panic!("{code} failed to load"));
            assert!(!data.tray.quit.trim().is_empty(), "{code} tray.quit is empty");
        }
    }

    #[test]
    fn unsupported_locale_is_rejected() {
        assert!(load_locale("xx-XX").is_none());
        assert!(set_locale("xx-XX").is_err());
    }
}
