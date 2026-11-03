/**
 * 启动脚本：在 Vue 应用加载前尽早通知 Rust 显示窗口。
 *
 * 轻量模式每次打开窗口都会重建 WebView，页面需要重新加载。若在窗口创建时
 * 立即显示，用户会先看到一片尚未渲染的空白表面；这里在静态启动外壳
 * （index.html 内联样式）已解析后立刻请求显示，使窗口的第一帧就是标题栏外壳。
 *
 * 该脚本不依赖任何模块，会在 main.ts 之前执行。
 */
interface TauriGlobal {
    core?: {
        invoke: (cmd: string) => Promise<unknown>
    }
}

const tauri = (window as unknown as { __TAURI__?: TauriGlobal }).__TAURI__

try {
    void tauri?.core?.invoke("boot_shell_ready")?.catch(() => {
        /* 非 Tauri 环境或命令尚未注册：由 main.ts 的显示逻辑兜底 */
    })
} catch {
    /* ignore */
}
