/**
 * 快速上手引导状态
 *
 * 首次启动时自动展示一次，关闭后写入 localStorage 不再打扰；
 * 用户可随时从「关于」页重新打开。
 */
import { defineStore } from "pinia";
import { ref } from "vue";

const STORAGE_KEY = "smtc2web.quickStartSeen";

function readSeen(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

export const useQuickStartStore = defineStore("quickStart", () => {
  const visible = ref(false);
  const seen = ref(readSeen());
  /** 「去商店安装」时暂时收起，离开商店页后自动续上，不算完成 */
  const paused = ref(false);

  function markSeen() {
    seen.value = true;
    try {
      localStorage.setItem(STORAGE_KEY, "1");
    } catch {
      /* localStorage 不可用时仅在本次会话内记住 */
    }
  }

  /** 首次启动自动展示，返回是否实际打开了向导 */
  function maybeShowOnFirstRun(): boolean {
    if (seen.value) return false;
    visible.value = true;
    return true;
  }

  function open() {
    paused.value = false;
    visible.value = true;
  }

  function close() {
    paused.value = false;
    visible.value = false;
    markSeen();
  }

  /** 暂时收起引导（跳转商店安装主题），不标记为已看过 */
  function pause() {
    paused.value = true;
    visible.value = false;
  }

  /** 从暂停状态恢复显示 */
  function resume() {
    if (!paused.value) return;
    paused.value = false;
    visible.value = true;
  }

  return { visible, paused, maybeShowOnFirstRun, open, close, pause, resume };
});
