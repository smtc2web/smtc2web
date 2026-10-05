// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    // GUI 模式：立即分离控制台（仅 Windows）
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Console::FreeConsole;
        let _ = FreeConsole();
    }

    smtc2web_lib::run();
}
