<template>
  <button
    class="install-btn"
    :class="[`install-btn--${status}`]"
    :disabled="status !== 'idle'"
    @click="handleClick"
  >
    <!-- 进度条背景 -->
    <div
      v-if="status === 'downloading' && hasTotal"
      class="install-btn__progress"
      :style="{ width: `${percent}%` }"
    ></div>
    <!-- 不确定进度条（无 Content-Length 时） -->
    <div
      v-if="status === 'downloading' && !hasTotal"
      class="install-btn__indeterminate"
    ></div>

    <!-- 按钮内容 -->
    <span class="install-btn__content">
      <template v-if="status === 'idle'">
        <font-awesome-icon icon="download" />
        {{ t('store.detail.install') }}
      </template>
      <template v-else-if="status === 'downloading'">
        <font-awesome-icon icon="spinner" spin />
        <span v-if="hasTotal">{{ displayPercent }}%</span>
        <span v-else>{{ t('store.detail.downloading') }}</span>
      </template>
      <template v-else-if="status === 'success'">
        <font-awesome-icon icon="check" />
        {{ t('store.detail.installed') }}
      </template>
      <template v-else-if="status === 'error'">
        <font-awesome-icon icon="times" />
        {{ t('store.detail.installFailed') }}
      </template>
    </span>
  </button>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
import { hasTauri } from "@/utils";

interface Props {
  downloadUrl: string;
  slug: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  installed: [folderName: string];
}>();

const { t } = useI18n();

type Status = "idle" | "downloading" | "success" | "error";

const status = ref<Status>("idle");
const downloaded = ref(0);
const total = ref(0);
const percent = ref(0);
const errorMsg = ref("");

let unlisten: (() => void) | null = null;

const hasTotal = computed(() => total.value > 0);
const displayPercent = computed(() => Math.round(percent.value));

async function handleClick() {
  if (status.value !== "idle") return;
  status.value = "downloading";
  downloaded.value = 0;
  total.value = 0;
  percent.value = 0;
  errorMsg.value = "";

  if (!hasTauri()) {
    status.value = "error";
    errorMsg.value = "需要 Tauri 环境";
    return;
  }

  try {
    const folderName = await window.__TAURI__!.core.invoke<string>(
      "download_theme_from_store",
      { downloadUrl: props.downloadUrl, slug: props.slug },
    );
    status.value = "success";
    emit("installed", folderName);
  } catch (e) {
    status.value = "error";
    errorMsg.value = e instanceof Error ? e.message : String(e);
    console.error("安装主题失败:", e);
  }
}

onMounted(async () => {
  if (!hasTauri()) return;
  const { listen } = await import("@tauri-apps/api/event");
  unlisten = await listen<{
    slug: string;
    downloaded: number;
    total: number;
    percent: number;
  }>("download-progress", (event) => {
    if (event.payload.slug !== props.slug) return;
    downloaded.value = event.payload.downloaded;
    total.value = event.payload.total;
    percent.value = event.payload.percent;
  });
});

onBeforeUnmount(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
});
</script>

<style scoped>
.install-btn {
  position: relative;
  padding: 10px 24px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--ui-space-xs);
  min-width: 140px;
  min-height: 40px;
  overflow: hidden;
  color: var(--ui-text-on-accent);
  background: var(--ui-accent);
}

.install-btn:hover:not(:disabled) {
  background: var(--ui-accent-hover);
}

.install-btn:disabled {
  cursor: default;
}

/* ---- 状态颜色 ---- */
.install-btn--idle {
  background: var(--ui-accent);
  color: var(--ui-text-on-accent);
}

.install-btn--downloading {
  background: var(--ui-bg-tertiary);
  color: var(--ui-text-primary);
}

.install-btn--success {
  background: var(--ui-success);
  color: #fff;
}

.install-btn--error {
  background: var(--ui-error);
  color: #fff;
}

/* ---- 进度条填充 ---- */
.install-btn__progress {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  background: var(--ui-accent);
  opacity: 0.25;
  transition: width 0.3s ease;
  pointer-events: none;
  border-radius: var(--ui-radius-md);
}

/* ---- 不确定进度动画 ---- */
.install-btn__indeterminate {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 40%;
  background: var(--ui-accent);
  opacity: 0.2;
  pointer-events: none;
  border-radius: var(--ui-radius-md);
  animation: indeterminate 1.4s ease-in-out infinite;
}

@keyframes indeterminate {
  0% {
    left: -40%;
  }
  100% {
    left: 100%;
  }
}

/* ---- 内容层 ---- */
.install-btn__content {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  gap: var(--ui-space-xs);
}
</style>
