import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hasTauri, tauriInvoke } from '@/utils'

export interface UpdateCheckResult {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  notes: string | null;
  error: string | null;
}

export interface UpdateProgress {
  /** 已下载字节数 */
  downloaded: number;
  /** 总字节数（未知时为 null） */
  total: number | null;
  /** 百分比（0–100，未知时为 null） */
  percent: number | null;
}

export const useUpdateStore = defineStore("update", () => {
  const checking = ref(false);
  const downloading = ref(false);
  const lastResult = ref<UpdateCheckResult | null>(null);
  const showDialog = ref(false);
  const downloadError = ref<string | null>(null);
  const progress = ref<UpdateProgress | null>(null);

  async function checkForUpdates(): Promise<UpdateCheckResult | null> {
    if (!hasTauri()) return null;
    checking.value = true;
    downloadError.value = null;

    try {
      const result = await tauriInvoke<UpdateCheckResult>("check_update");
      lastResult.value = result;
      if (result.has_update || result.error) {
        showDialog.value = true;
      }
      return result;
    } catch (e) {
      console.error("检查更新失败:", e);
      lastResult.value = {
        has_update: false,
        current_version: "",
        latest_version: "",
        notes: null,
        error: String(e),
      };
      return lastResult.value;
    } finally {
      checking.value = false;
    }
  }

  async function downloadAndInstall(): Promise<void> {
    if (!hasTauri()) return;
    if (!lastResult.value?.has_update) return;
    downloading.value = true;
    downloadError.value = null;
    progress.value = { downloaded: 0, total: null, percent: null };

    try {
      await tauriInvoke("start_update");
    } catch (e) {
      console.error("下载更新失败:", e);
      downloadError.value = String(e);
      throw e;
    } finally {
      downloading.value = false;
    }
  }

  function setProgress(payload: UpdateProgress) {
    progress.value = payload;
  }

  function closeDialog() {
    showDialog.value = false;
    downloadError.value = null;
    progress.value = null;
  }

  return {
    checking,
    downloading,
    lastResult,
    showDialog,
    downloadError,
    progress,
    checkForUpdates,
    downloadAndInstall,
    setProgress,
    closeDialog,
  };
});
