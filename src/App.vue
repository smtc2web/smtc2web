<template>
    <div id="app">
        <NavBar />
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
import NavBar from "@/components/NavBar.vue";
import Toast from "@/components/Toast.vue";
import UpdateDialog from "@/components/UpdateDialog.vue";
import { hasTauri } from "@/utils";

const configStore = useConfigStore();
const localeStore = useLocaleStore();
const updateStore = useUpdateStore();
const route = useRoute();

// 页面切换动效：按导航顺序决定滑入方向（主题 -> 设置向右，设置 -> 主题向左）。
const PAGE_ORDER: Record<string, number> = { themes: 0, settings: 1 };
const pageTransition = computed(() => {
    const name = String(route.name ?? "themes");
    return (PAGE_ORDER[name] ?? 0) >= 1 ? "page-next" : "page-prev";
});

onMounted(async () => {
    await configStore.loadConfig();
    localeStore.initLocale();

    if (configStore.config.auto_check_update) {
        setTimeout(() => updateStore.checkForUpdates(), 1000);
    }

    if (hasTauri()) {
        import("@tauri-apps/api/event").then(({ listen }) => {
            listen("check-update", () => updateStore.checkForUpdates());
            listen<UpdateProgress>("update-progress", (e) =>
                updateStore.setProgress(e.payload),
            );
        });
    }
});
</script>

<style>
#app {
    min-height: 100vh;
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
