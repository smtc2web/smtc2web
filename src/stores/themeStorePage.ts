/**
 * 主题商店 Pinia Store
 *
 * 管理远程主题商店的浏览、搜索、标签筛选、排序、分页和安装状态。
 */
import { defineStore } from "pinia";
import { ref, shallowRef, reactive } from "vue";
import { storeApi } from "@/api/store";
import type { ThemeSummary, ThemeDetail, TagCount, ThemeListParams } from "@/types/store";

export const useThemeStorePage = defineStore("themeStorePage", () => {
  /* ---------- 列表状态 ---------- */
  // 列表/详情都是整体替换，不需要深层响应式代理。
  const themes = shallowRef<ThemeSummary[]>([]);
  const total = ref(0);
  const page = ref(1);
  const totalPages = ref(1);
  const loading = ref(false);
  const errorMsg = ref("");

  /* ---------- 搜索 / 筛选 ---------- */
  const filters = reactive<ThemeListParams>({
    q: "",
    tag: "",
    sort: "newest",
  });

  /* ---------- 标签 ---------- */
  const tags = shallowRef<TagCount[]>([]);

  /* ---------- 详情弹窗 ---------- */
  const selectedSlug = ref<string | null>(null);
  const detail = shallowRef<ThemeDetail | null>(null);
  const detailLoading = ref(false);
  const detailError = ref("");

  /* ---------- 安装状态 ---------- */
  const installing = ref<Record<string, boolean>>({});

  // 离开页面时递增，丢弃仍在途的请求结果，避免旧数据被重新写回
  // 已经释放的 store 里（否则会一直持有列表和详情）。
  let listRequest = 0;
  let detailRequest = 0;
  let tagsRequest = 0;

  /* ---------- 列表操作 ---------- */

  async function loadThemes() {
    const request = ++listRequest;
    loading.value = true;
    errorMsg.value = "";
    try {
      const data = await storeApi.listThemes({
        q: filters.q,
        tag: filters.tag,
        sort: filters.sort,
        page: page.value,
      });
      if (request !== listRequest) return;
      themes.value = data.themes;
      total.value = data.total;
      totalPages.value = data.total_pages;
      page.value = data.page;
    } catch (e) {
      if (request !== listRequest) return;
      errorMsg.value = e instanceof Error ? e.message : "加载主题列表失败";
    } finally {
      if (request === listRequest) loading.value = false;
    }
  }

  async function loadTags() {
    const request = ++tagsRequest;
    try {
      const data = await storeApi.listTags();
      if (request !== tagsRequest) return;
      tags.value = data.tags;
    } catch {
      /* 标签加载失败不影响列表 */
    }
  }

  function search(query: string) {
    filters.q = query.trim();
    page.value = 1;
  }

  function setTag(tag: string) {
    filters.tag = filters.tag === tag ? "" : tag;
    page.value = 1;
  }

  function setSort(sort: string) {
    filters.sort = sort;
    page.value = 1;
  }

  function setPage(p: number) {
    page.value = Math.max(1, Math.min(p, totalPages.value));
  }

  /* ---------- 详情操作 ---------- */

  async function loadDetail(slug: string) {
    const request = ++detailRequest;
    selectedSlug.value = slug;
    detailLoading.value = true;
    detailError.value = "";
    detail.value = null;
    try {
      const data = await storeApi.getTheme(slug);
      if (request !== detailRequest) return;
      detail.value = data;
    } catch (e) {
      if (request !== detailRequest) return;
      detailError.value = e instanceof Error ? e.message : "加载主题详情失败";
    } finally {
      if (request === detailRequest) detailLoading.value = false;
    }
  }

  function closeDetail() {
    detailRequest++;
    selectedSlug.value = null;
    detail.value = null;
    detailError.value = "";
    detailLoading.value = false;
  }

  /** 离开商店页时释放列表/详情数据，避免其随 Pinia 常驻。 */
  function reset() {
    listRequest++;
    detailRequest++;
    tagsRequest++;
    themes.value = [];
    total.value = 0;
    page.value = 1;
    totalPages.value = 1;
    loading.value = false;
    errorMsg.value = "";
    tags.value = [];
    installing.value = {};
    closeDetail();
  }

  /* ---------- 安装操作 ---------- */

  async function installTheme(theme: ThemeSummary): Promise<boolean> {
    const key = theme.slug;
    if (installing.value[key]) return false;
    installing.value = { ...installing.value, [key]: true };
    try {
      await storeApi.downloadAndInstall(theme.download_url, theme.slug);
      return true;
    } catch (e) {
      console.error("安装主题失败:", e);
      throw e;
    } finally {
      const next = { ...installing.value };
      delete next[key];
      installing.value = next;
    }
  }

  function isInstalling(slug: string): boolean {
    return Boolean(installing.value[slug]);
  }

  return {
    /* state */
    themes,
    total,
    page,
    totalPages,
    loading,
    errorMsg,
    filters,
    tags,
    selectedSlug,
    detail,
    detailLoading,
    detailError,
    installing,
    /* actions */
    loadThemes,
    loadTags,
    search,
    setTag,
    setSort,
    setPage,
    loadDetail,
    closeDetail,
    reset,
    installTheme,
    isInstalling,
  };
});
