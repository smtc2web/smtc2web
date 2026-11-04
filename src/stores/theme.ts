import { defineStore } from "pinia";
import { ref, shallowRef, computed } from "vue";
import { useI18n } from "vue-i18n";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useToastStore } from "./toast";
import type { Theme, GitThemeInfo } from "@/types/theme";
import { hasTauri, tauriInvoke } from "@/utils";

interface PreloadedThemeData {
  themes: Theme[];
  currentTheme: string;
  gitThemes: Record<string, GitThemeInfo>;
}

/**
 * 启动阶段预取的主题数据。
 *
 * 刻意不依赖 Pinia / i18n，因此可以在 `main.ts` 中（组件挂载前）安全调用，
 * 让窗口显示时主题列表已经就绪，不会先闪一下骨架屏或空状态。
 */
let preloaded: PreloadedThemeData | null = null;

export async function preloadThemeData(): Promise<void> {
  if (!hasTauri()) return;

  try {
    const [themes, currentTheme, gitThemes] = await Promise.all([
      tauriInvoke<Theme[]>("get_themes"),
      tauriInvoke<string>("get_current_theme"),
      tauriInvoke<Record<string, GitThemeInfo>>("get_git_themes"),
    ]);
    preloaded = { themes, currentTheme, gitThemes };
  } catch (e) {
    console.error("预加载主题数据失败:", e);
  }
}

export const useThemeStore = defineStore("theme", () => {
  const { t } = useI18n();
  const toast = useToastStore();

  // 主题对象只整体替换、不就地修改，使用 shallowRef 避免为每个字段建立
  // 深层响应式代理（主题列表里可能带有较长的截图路径/字符串）。
  const initial = preloaded;
  const themes = shallowRef<Theme[]>(initial?.themes ?? []);
  const currentTheme = ref(initial?.currentTheme ?? "");
  // 只有确实没有缓存数据时才展示骨架屏，避免已有数据时刷新造成闪烁。
  const loading = ref(themes.value.length === 0);
  const uploadLoading = ref(false);
  const gitThemes = shallowRef<Record<string, GitThemeInfo>>(initial?.gitThemes ?? {});

  // 启动时已经预取过数据，首次挂载时无需再重复请求（避免重复扫描主题目录）。
  let hydrated = initial !== null;
  // 预取数据已移交给 Pinia，释放模块级引用，避免长期持有同一份列表。
  preloaded = null;
  function consumeHydrated(): boolean {
    const value = hydrated;
    hydrated = false;
    return value;
  }

  const hasThemes = computed(() => themes.value.length > 0);

  const mockThemes: Theme[] = [
    {
      name: "默认主题",
      folder_name: "default",
      author: "smtc2web",
      version: "1.0.0",
      screenshot_path: "",
      is_default: true,
    },
  ];

  async function loadThemes() {
    loading.value = themes.value.length === 0;
    try {
      if (hasTauri()) {
        themes.value = await tauriInvoke<Theme[]>("get_themes");
      } else {
        themes.value = mockThemes;
      }
    } catch (e) {
      console.error("加载主题失败:", e);
      themes.value = mockThemes;
      toast.error(t("messages.theme.loadError"));
    } finally {
      loading.value = false;
    }
  }

  async function loadCurrentTheme() {
    try {
      if (hasTauri()) {
        currentTheme.value = await tauriInvoke<string>("get_current_theme");
      } else {
        currentTheme.value = "default";
      }
    } catch (e) {
      console.error("加载当前主题失败:", e);
    }
  }

  async function selectTheme(folderName: string) {
    if (folderName === currentTheme.value) return;

    try {
      if (hasTauri()) {
        await tauriInvoke("set_theme", { themeName: folderName });
        await loadCurrentTheme();
        toast.success(t("messages.theme.switchSuccess"));
      } else {
        currentTheme.value = folderName;
        toast.success(t("messages.theme.switchSuccess") + " (模拟模式)");
      }
    } catch (e: any) {
      console.error("切换主题失败:", e);
      const message = e.message || e.toString() || String(e);
      toast.error(t("messages.theme.switchError", { message }));
    }
  }

  async function deleteTheme(theme: Theme) {
    toast.confirm(
      t("messages.theme.deleteConfirm", { name: theme.name }),
      {
        confirmText: t("common.delete"),
        cancelText: t("common.cancel"),
        onConfirm: async () => {
          try {
            if (hasTauri()) {
              await tauriInvoke("delete_theme", { themeFolder: theme.folder_name });
              await loadThemes();
            } else {
              themes.value = themes.value.filter(
                (t: Theme) => t.folder_name !== theme.folder_name,
              );
            }
          } catch (e: any) {
            console.error("删除主题失败:", e);
            const message = e.message || e.toString() || String(e);
            toast.error(t("messages.theme.deleteError", { message }));
          }
        },
      },
      t("common.confirm"),
    );
  }

  async function uploadTheme() {
    if (!hasTauri()) {
      toast.error("上传功能需要 Tauri 环境");
      return;
    }

    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".zip";

    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      if (!file.name.endsWith(".zip")) {
        toast.error(t("messages.theme.invalidFormat"));
        return;
      }

      uploadLoading.value = true;
      try {
        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);

        const themeName = await tauriInvoke<string>("upload_theme_from_bytes", {
          fileName: file.name,
          fileData: Array.from(uint8Array),
        });

        await new Promise((resolve) => setTimeout(resolve, 500));
        await loadThemes();
        toast.success(t("messages.theme.uploadSuccess", { name: themeName }));
      } catch (e: any) {
        console.error("上传主题失败:", e);
        // 处理 Tauri 返回的错误信息
        const errorMessage = e.message || e.toString() || String(e);

        // 检查是否是验证错误，提取其中的详细信息
        if (errorMessage.includes("此主题无效")) {
          // 直接使用后端返回的详细错误信息
          toast.error(errorMessage, t("common.error"));
        } else {
          toast.error(
            t("messages.theme.uploadError", { message: errorMessage }),
          );
        }
      } finally {
        uploadLoading.value = false;
      }
    };

    input.click();
  }

  async function loadGitThemes() {
    try {
      if (hasTauri()) {
        gitThemes.value = await tauriInvoke<Record<string, GitThemeInfo>>("get_git_themes");
      }
    } catch (e) {
      console.error("加载 Git 主题信息失败:", e);
    }
  }

  async function installFromGit(repoUrl: string, branch: string): Promise<boolean> {
    if (!repoUrl.trim()) {
      toast.error(t("messages.theme.gitUrlRequired"));
      return false;
    }

    try {
      if (hasTauri()) {
        await tauriInvoke<string>("install_theme_from_git", { repoUrl, branch });
        await Promise.all([loadThemes(), loadGitThemes()]);
        toast.success(t("messages.theme.gitInstallSuccess"));
        return true;
      } else {
        toast.error("Git 安装功能需要 Tauri 环境");
        return false;
      }
    } catch (e: any) {
      console.error("从 Git 安装主题失败:", e);
      const message = e.message || e.toString() || String(e);
      toast.error(t("messages.theme.gitInstallError", { message }));
      return false;
    }
  }

  async function updateGitTheme(folderName: string) {
    try {
      if (hasTauri()) {
        await tauriInvoke("update_git_theme", { folderName });
        toast.success(t("messages.theme.gitUpdateSuccess"));
      }
    } catch (e: any) {
      console.error("更新 Git 主题失败:", e);
      const message = e.message || e.toString() || String(e);
      toast.error(t("messages.theme.gitUpdateError", { message }));
    }
  }

  async function checkGitUpdate(folderName: string): Promise<boolean> {
    try {
      if (hasTauri()) {
        return await tauriInvoke<boolean>("check_git_theme_update", { folderName });
      }
    } catch (e) {
      console.error("检查 Git 主题更新失败:", e);
    }
    return false;
  }

  function isGitTheme(folderName: string): boolean {
    return folderName in gitThemes.value;
  }

  /**
   * 把后端返回的截图地址转换成 WebView 可加载的 URL。
   *
   * - 默认主题的截图内嵌在二进制里，后端仍返回 data URI；
   * - 普通主题返回磁盘绝对路径，改用 Tauri asset 协议按需读取。
   *   这样截图不会再以 Base64 字符串的形式常驻 JS 堆，也避免一次
   *   性解码所有主题截图，显著降低前端内存占用。
   */
  function getScreenshotUrl(path: string): string | null {
    if (!path) return null;
    if (path.startsWith("data:")) return path;
    if (!hasTauri()) return null;

    try {
      return convertFileSrc(path);
    } catch (e) {
      console.error("生成主题截图地址失败:", e);
      return null;
    }
  }

  return {
    themes,
    currentTheme,
    loading,
    uploadLoading,
    gitThemes,
    hasThemes,
    consumeHydrated,
    loadThemes,
    loadCurrentTheme,
    loadGitThemes,
    selectTheme,
    deleteTheme,
    uploadTheme,
    installFromGit,
    updateGitTheme,
    checkGitUpdate,
    isGitTheme,
    getScreenshotUrl,
  };
});
