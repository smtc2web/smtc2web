export interface AppConfig {
  server_port: number;
  address: string;
  current_theme: string;
  locale: string;
  process_filter: string;
  /** 是否启用自动检查更新 */
  auto_check_update: boolean;
  /** 启动时最小化至系统托盘 */
  minimize_to_tray: boolean;
  /** 轻量模式：关闭窗口时销毁界面，仅保留后台进程 */
  lightweight_mode: boolean;
  /** 主题 overlay 使用的字体 */
  font_family: string;
}
