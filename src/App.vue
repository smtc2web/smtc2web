<template>
    <div class="app-shell">
        <TitleBar />
        <main class="page">
            <RouterView v-slot="{ Component, route }">
                <Transition :name="pageTransition" mode="out-in">
                    <component :is="Component" :key="route.path" />
                </Transition>
            </RouterView>
        </main>
        <Toast />
        <UpdateDialog />
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRoute } from "vue-router";
import { useConfigStore } from "@/stores/config";
import { useLocaleStore } from "@/stores/locale";
import { useUpdateStore, type UpdateProgress } from "@/stores/update";
import TitleBar from "@/components/TitleBar.vue";
import Toast from "@/components/Toast.vue";
import UpdateDialog from "@/components/UpdateDialog.vue";
import { hasTauri, tauriInvoke } from "@/utils";

const configStore = useConfigStore();
const localeStore = useLocaleStore();
const updateStore = useUpdateStore();
const route = useRoute();

// 配置已在挂载前加载完毕（见 main.ts），这里同步应用语言，
// 保证首帧文案就是用户设置的语言，不会出现「先中文再切换」的闪烁。
localeStore.initLocale();

// 页面切换动效：按导航顺序决定滑入方向（主题 -> 商店 -> 设置 -> 关于 依次向右）。
const PAGE_ORDER: Record<string, number> = { themes: 0, store: 1, settings: 2, about: 3 };
const pageTransition = computed(() => {
    const name = String(route.name ?? "themes");
    return (PAGE_ORDER[name] ?? 0) >= 1 ? "page-next" : "page-prev";
});

onMounted(async () => {
    if (configStore.config.auto_check_update) {
        setTimeout(() => updateStore.checkForUpdates(), 1000);
    }

    if (hasTauri()) {
        import("@tauri-apps/api/event").then(async ({ listen }) => {
            await listen("check-update", () => updateStore.checkForUpdates(true));
            await listen<UpdateProgress>("update-progress", (e) =>
                updateStore.setProgress(e.payload),
            );

            // 轻量模式下托盘触发的检查：窗口重建并注册监听后取走待办，
            // 避免事件在 WebView 就绪前发送而丢失。
            const pending = await tauriInvoke<boolean>("take_pending_update_check");
            if (pending) {
                updateStore.checkForUpdates(true);
            }
        });
    }
});
</script>

<style>
.app-shell {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

/* ===== 页面切换动效（Transition 进出场钩子）===== */
.page-next-enter-active,
.page-prev-enter-active,
.page-next-leave-active,
.page-prev-leave-active {
    transition:
        opacity 0.2s ease,
        transform 0.24s cubic-bezier(0.16, 1, 0.3, 1);
}

.page-next-enter-from,
.page-prev-leave-to {
    opacity: 0;
    transform: translateX(28px);
}

.page-prev-enter-from,
.page-next-leave-to {
    opacity: 0;
    transform: translateX(-28px);
}

@media (prefers-reduced-motion: reduce) {
    .page-next-enter-active,
    .page-prev-enter-active,
    .page-next-leave-active,
    .page-prev-leave-active {
        transition: none;
    }
}
</style>
