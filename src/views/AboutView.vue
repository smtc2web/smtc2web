<template>
    <div class="about-view">
        <header class="page-header">
            <h2>{{ t("about.title") }}</h2>
        </header>

        <section class="about-card">
            <div class="about-hero">
                <img class="app-icon" src="/app-icon.png" alt="smtc2web" />
                <div class="app-meta">
                    <h3 class="app-name">smtc2web</h3>
                    <p class="app-version">
                        {{ t("about.version") }}
                        <code>v{{ version }}</code>
                    </p>
                    <p class="app-description">{{ t("about.description") }}</p>
                </div>
            </div>
            <div class="about-actions">
                <button type="button" class="replay-btn" @click="quickStart.open()">
                    <font-awesome-icon icon="rotate" />
                    {{ t("about.quickStart") }}
                </button>
                <span class="replay-desc">{{ t("about.quickStartDesc") }}</span>
            </div>
        </section>

        <section class="about-links">
            <button
                v-for="link in links"
                :key="link.url"
                type="button"
                class="link-card"
                @click="openExternal(link.url)"
            >
                <font-awesome-icon :icon="link.icon" class="link-icon" />
                <span class="link-text">
                    <span class="link-label">{{ t(link.labelKey) }}</span>
                    <span class="link-url">{{ link.url }}</span>
                </span>
                <font-awesome-icon icon="external-link" class="link-arrow" />
            </button>
        </section>

        <footer class="about-footer">
            <p>{{ t("about.license") }}</p>
            <p class="copyright">
                {{ t("about.copyright", { year: currentYear }) }}
            </p>
        </footer>
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useQuickStartStore } from "@/stores/quickStart";
import { hasTauri, tauriInvoke } from "@/utils";

interface AboutLink {
    icon: string;
    labelKey: string;
    url: string;
}

const { t } = useI18n();
const quickStart = useQuickStartStore();

const version = ref("—");
const currentYear = new Date().getFullYear();

const links: AboutLink[] = [
    {
        icon: "code-branch",
        labelKey: "about.links.github",
        url: "https://github.com/smtc2web/smtc2web",
    },
    {
        icon: "globe",
        labelKey: "about.links.website",
        url: "https://smtc2web.org",
    },
    {
        icon: "store",
        labelKey: "about.links.store",
        url: "https://themes.smtc2web.org",
    },
    {
        icon: "book",
        labelKey: "about.links.docs",
        url: "https://smtc2web.org/wiki/compile",
    },
];

async function openExternal(url: string) {
    if (hasTauri()) {
        try {
            await tauriInvoke("open_url", { url });
            return;
        } catch (e) {
            console.error("打开外部链接失败:", e);
        }
    }
    window.open(url, "_blank", "noopener,noreferrer");
}

onMounted(async () => {
    if (!hasTauri()) return;
    try {
        const { getVersion } = await import("@tauri-apps/api/app");
        version.value = await getVersion();
    } catch (e) {
        console.error("读取应用版本失败:", e);
    }
});
</script>

<style scoped>
.about-view {
    width: 100%;
    max-width: 720px;
}

.about-card {
    background-color: var(--ui-bg-card);
    border-radius: var(--ui-radius-lg);
    box-shadow: var(--ui-shadow-md);
    padding: var(--ui-space-lg);
}

.about-hero {
    display: flex;
    align-items: flex-start;
    gap: var(--ui-space-lg);
}

.app-icon {
    width: 72px;
    height: 72px;
    border-radius: var(--ui-radius-md);
    flex-shrink: 0;
}

.app-meta {
    min-width: 0;
}

.app-name {
    font-size: 22px;
    font-weight: 700;
    color: var(--ui-text-primary);
}

.app-version {
    margin-top: var(--ui-space-xs);
    font-size: 13px;
    color: var(--ui-text-secondary);
}

.app-version code {
    font-size: 13px;
    color: var(--ui-text-primary);
    background-color: var(--ui-bg-secondary);
    padding: 2px 8px;
    border-radius: var(--ui-radius-sm);
}

.app-description {
    margin-top: var(--ui-space-sm);
    font-size: 14px;
    line-height: 1.6;
    color: var(--ui-text-secondary);
}

.about-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--ui-space-sm);
    margin-top: var(--ui-space-md);
    padding-top: var(--ui-space-md);
    border-top: 1px solid var(--ui-border);
}

.replay-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 1px solid var(--ui-border);
    border-radius: var(--ui-radius-md);
    background-color: var(--ui-bg-secondary);
    color: var(--ui-text-primary);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition:
        border-color var(--ui-transition-fast),
        color var(--ui-transition-fast),
        background-color var(--ui-transition-fast);
}

.replay-btn:hover {
    border-color: var(--ui-accent);
    color: var(--ui-accent);
}

.replay-desc {
    font-size: 12px;
    color: var(--ui-text-secondary);
}

.about-links {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--ui-space-sm);
    margin-top: var(--ui-space-lg);
}

.link-card {
    display: flex;
    align-items: center;
    gap: var(--ui-space-sm);
    padding: var(--ui-space-md);
    background-color: var(--ui-bg-card);
    border: 1px solid var(--ui-border);
    border-radius: var(--ui-radius-md);
    box-shadow: var(--ui-shadow-sm);
    color: var(--ui-text-primary);
    text-align: left;
    cursor: pointer;
    transition:
        border-color var(--ui-transition-fast),
        background-color var(--ui-transition-fast);
}

.link-card:hover {
    border-color: var(--ui-accent);
    background-color: var(--ui-bg-secondary);
}

.link-icon {
    font-size: 16px;
    color: var(--ui-accent);
    flex-shrink: 0;
}

.link-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
}

.link-label {
    font-size: 14px;
    font-weight: 600;
}

.link-url {
    font-size: 12px;
    color: var(--ui-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.link-arrow {
    font-size: 12px;
    color: var(--ui-text-secondary);
    flex-shrink: 0;
}

.about-footer {
    margin-top: var(--ui-space-lg);
    font-size: 12px;
    color: var(--ui-text-secondary);
    line-height: 1.8;
}

.copyright {
    opacity: 0.8;
}
</style>
