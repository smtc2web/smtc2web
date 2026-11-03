import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import './config/fontAwesome'
import './styles/global.css'
import { useConfigStore } from './stores/config'
import { preloadThemeData } from './stores/theme'
import { hasTauri } from './utils'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(i18n)
app.use(router)

app.component('font-awesome-icon', FontAwesomeIcon)

async function bootstrap() {
    const configStore = useConfigStore(pinia)

    // 窗口默认隐藏（tauri.conf.json 中 visible: false）。先并行准备首帧所需
    // 的全部内容：用户配置、主题列表、首个路由组件，避免界面逐步填充。
    await Promise.all([
        configStore.loadConfig(),
        preloadThemeData(),
        router.isReady().catch(() => {}),
    ])

    app.mount('#app')

    // 首帧内容已提交：淡出静态启动外壳，露出真正的前端界面。
    // （窗口在 boot.ts 通知后就会显示，外壳负责覆盖 WebView 的加载过程。）
    const shell = document.getElementById('boot-shell')
    if (shell) {
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            shell.remove()
        } else {
            shell.style.transition = 'opacity 120ms ease'
            shell.style.opacity = '0'
            window.setTimeout(() => shell.remove(), 140)
        }
    }

    if (!hasTauri()) return

    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')

    // 启动时若选择「最小化到托盘」，则保持隐藏，等待托盘左键唤出。
    // 轻量模式下窗口是托盘点击后重建的，此时必须显示。
    const { minimize_to_tray, lightweight_mode } = configStore.config
    if (!minimize_to_tray || lightweight_mode) {
        await WebviewWindow.getCurrent().show()
    }
}

bootstrap()
