export function hasTauri(): boolean {
  return typeof window !== "undefined" && window.__TAURI__ !== undefined;
}

export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!hasTauri() || !window.__TAURI__) throw new Error("Not in Tauri");
  return window.__TAURI__.core.invoke(cmd, args);
}

/** 复制文本到剪贴板，返回是否成功（兼容不支持 Clipboard API 的 WebView） */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* 继续尝试降级方案 */
  }

  try {
    const el = document.createElement("textarea");
    el.value = text;
    el.style.position = "fixed";
    el.style.opacity = "0";
    document.body.appendChild(el);
    el.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(el);
    return ok;
  } catch {
    return false;
  }
}
