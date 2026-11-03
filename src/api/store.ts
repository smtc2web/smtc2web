/**
 * 主题商店 API 客户端
 *
 * 与远程 smtc2web-themes 服务端通信，浏览/搜索/下载主题。
 * 下载操作通过 Tauri Rust 后端执行，避免 WebView CSP 限制。
 */
import type {
  ThemeList,
  ThemeDetail,
  TagCount,
  ThemeListParams,
} from "@/types/store";
import { hasTauri, tauriInvoke } from "@/utils";

/** 主题商店 API 根地址，可根据部署环境修改 */
const API_BASE = "https://themes.smtc2web.org";

class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
    this.name = "StoreApiError";
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const url = `${API_BASE}${path}`;
  let response: Response;
  try {
    response = await fetch(url, {
      ...init,
      headers: {
        accept: "application/json",
        ...init?.headers,
      },
    });
  } catch (e) {
    throw new ApiError(0, `网络请求失败：${e instanceof Error ? e.message : String(e)}`);
  }
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new ApiError(response.status, `请求失败 (${response.status})：${text || response.statusText}`);
  }
  return response.json() as Promise<T>;
}

export const storeApi = {
  /** 获取主题列表（支持搜索、标签、排序、分页） */
  listThemes(params: ThemeListParams = {}): Promise<ThemeList> {
    const search = new URLSearchParams();
    if (params.q) search.set("q", params.q);
    if (params.tag) search.set("tag", params.tag);
    if (params.sort && params.sort !== "newest") search.set("sort", params.sort);
    if (params.page && params.page > 1) search.set("page", String(params.page));
    const qs = search.toString();
    return request<ThemeList>(`/api/themes${qs ? `?${qs}` : ""}`);
  },

  /** 获取主题详情（含版本列表） */
  getTheme(slug: string): Promise<ThemeDetail> {
    return request<ThemeDetail>(`/api/themes/${encodeURIComponent(slug)}`);
  },

  /** 获取所有标签及计数 */
  listTags(): Promise<{ tags: TagCount[] }> {
    return request<{ tags: TagCount[] }>("/api/tags");
  },

  /**
   * 从商店下载并安装主题。
   * 通过 Tauri Rust 后端执行 HTTP 下载 + 本地解压安装，
   * 避免 WebView CSP 对跨域下载的限制。
   */
  async downloadAndInstall(downloadUrl: string, slug: string): Promise<string> {
    if (!hasTauri()) {
      throw new Error("安装主题需要 Tauri 环境");
    }
    return tauriInvoke<string>("download_theme_from_store", {
      downloadUrl,
      slug,
    });
  },
};
