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
        <!-- 更新弹窗只在需要时加载，避免把弹窗代码常驻在首屏分块里。 -->
        <UpdateDialog v-if="updateStore.showDialog" />
        <QuickStartWizard
            :visible="quickStart.visible"
            @close="quickStart.close()"
        />
    </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useConfigStore } from "@/stores/config";
import { useLocaleStore } from "@/stores/locale";
import { useQuickStartStore } from "@/stores/quickStart";
import { useUpdateStore, type UpdateProgress } from "@/stores/update";
import TitleBar from "@/components/TitleBar.vue";
import Toast from "@/components/Toast.vue";
import QuickStartWizard from "@/components/QuickStartWizard.vue";
import { hasTauri, tauriInvoke } from "@/utils";

const UpdateDialog = defineAsyncComponent(
    () => import("@/components/UpdateDialog.vue"),
);

const configStore = useConfigStore();
const localeStore = useLocaleStore();
const quickStart = useQuickStartStore();
const updateStore = useUpdateStore();
const route = useRoute();

// 配置已在挂载前加载完毕（见 main.ts），这里同步应用语言，
// 保证首帧文案就是用户设置的语言，不会出现「先中文再切换」的闪烁。
void localeStore.initLocale();

// 页面切换动效：按导航顺序决定滑入方向（主题 -> 商店 -> 设置 -> 关于 依次向右）。
// 必须比较「来源页」和「目标页」的序号，只看目标页会导致后退时方向错乱。
const PAGE_ORDER: Record<string, number> = { themes: 0, store: 1, settings: 2, about: 3 };
const pageOrder = (name: unknown) => PAGE_ORDER[String(name ?? "themes")] ?? 0;

const pageTransition = ref("page-next");
let previousOrder = pageOrder(route.name);

watch(
    () => route.name,
    (name) => {
        const nextOrder = pageOrder(name);
        pageTransition.value = nextOrder >= previousOrder ? "page-next" : "page-prev";
        previousOrder = nextOrder;
    },
);

onMounted(async () => {
    // 首次启动的快速上手向导会抢占焦点，更新检查等它关闭后再触发。
    const quickStartShown = quickStart.maybeShowOnFirstRun();

    if (configStore.config.auto_check_update) {
        if (quickStartShown) {
            // 引导「暂停」去商店安装主题时不算结束，等真正关闭后再检查更新。
            const stop = watch(
                () => quickStart.visible || quickStart.paused,
                (active) => {
                    if (active) return;
                    stop();
                    setTimeout(() => updateStore.checkForUpdates(), 1000);
                },
            );
        } else {
            setTimeout(() => updateStore.checkForUpdates(), 1000);
        }
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
