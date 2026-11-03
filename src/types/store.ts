/** 主题商店 API 类型定义（与 smtc2web-themes 服务端接口对齐） */

export interface ThemeAuthor {
  login: string;
  avatar_url: string;
}

export interface ThemeSummary {
  slug: string;
  name: string;
  description: string;
  tags: string[];
  author: ThemeAuthor;
  version: string;
  version_count: number;
  downloads: number;
  repo_url: string;
  source_repo: string | null;
  published_via: string;
  screenshot_url: string;
  download_url: string;
  page_url: string;
  created_at: string;
  updated_at: string;
}

export interface ThemeVersion {
  version: string;
  size: number;
  downloads: number;
  has_screenshot: boolean;
  created_at: string;
  download_url: string;
}

export interface ThemeDetail {
  theme: ThemeSummary;
  versions: ThemeVersion[];
}

export interface ThemeList {
  themes: ThemeSummary[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface TagCount {
  tag: string;
  count: number;
}

export interface ThemeListParams {
  q?: string;
  tag?: string;
  sort?: string;
  page?: number;
}
