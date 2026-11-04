export interface Theme {
  name: string
  folder_name: string
  author: string
  version: string
  /** 默认主题为 data URI；普通主题为磁盘路径，通过 Tauri asset 协议加载。 */
  screenshot_path: string
  is_default?: boolean
}

export interface GitThemeInfo {
  repo_url: string
  branch: string
  folder_name: string
}
