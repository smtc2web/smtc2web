<template>
  <DialogRoot :open="visible" @update:open="(v) => { if (!v) $emit('close') }">
    <DialogPortal>
      <DialogOverlay class="qs-overlay" @click="$emit('close')" />
      <DialogContent class="qs-dialog">
        <div class="qs-header">
          <div class="qs-heading">
            <span class="qs-badge">
              <font-awesome-icon icon="circle-question" />
            </span>
            <div>
              <DialogTitle as="h3">{{ t("quickStart.title") }}</DialogTitle>
              <DialogDescription class="qs-subtitle">
                {{ t("quickStart.description") }}
              </DialogDescription>
            </div>
          </div>
          <DialogClose as-child>
            <button
              class="qs-close"
              type="button"
              :aria-label="t('quickStart.skip')"
              @click="$emit('close')"
            >
              <font-awesome-icon icon="times" />
            </button>
          </DialogClose>
        </div>

        <div class="qs-body">
          <section v-if="currentKey === 'theme'" class="qs-step">
            <div class="qs-step-icon">
              <font-awesome-icon icon="download" />
            </div>
            <h4>{{ t("quickStart.steps.theme.title") }}</h4>
            <p>{{ t("quickStart.steps.theme.desc") }}</p>
            <button type="button" class="btn btn-secondary" @click="goStore">
              <font-awesome-icon icon="store" />
              {{ t("quickStart.steps.theme.action") }}
            </button>
          </section>

          <section v-else-if="currentKey === 'connect'" class="qs-step">
            <div class="qs-step-icon">
              <font-awesome-icon icon="globe" />
            </div>
            <h4>{{ t("quickStart.steps.connect.title") }}</h4>
            <p>{{ t("quickStart.steps.connect.desc") }}</p>

            <div class="qs-url-block">
              <span class="qs-url-label">{{ t("quickStart.steps.connect.urlLabel") }}</span>
              <div class="qs-url-row">
                <code class="qs-url">{{ overlayUrl }}</code>
                <button type="button" class="btn btn-secondary" @click="handleCopy">
                  <font-awesome-icon :icon="copied ? 'check' : 'copy'" />
                  {{
                    copied
                      ? t("quickStart.steps.connect.copied")
                      : t("quickStart.steps.connect.copy")
                  }}
                </button>
              </div>
            </div>

            <ol class="qs-obs-steps">
              <li>{{ t("quickStart.steps.connect.obsStep1") }}</li>
              <li>{{ t("quickStart.steps.connect.obsStep2") }}</li>
              <li>{{ t("quickStart.steps.connect.obsStep3") }}</li>
            </ol>

            <button type="button" class="btn btn-ghost" @click="handlePreview">
              <font-awesome-icon icon="external-link" />
              {{ t("quickStart.steps.connect.preview") }}
            </button>
          </section>

          <section v-else class="qs-step">
            <div class="qs-step-icon qs-step-icon--success">
              <font-awesome-icon icon="check-circle" />
            </div>
            <h4>{{ t("quickStart.steps.ready.title") }}</h4>
            <p>{{ t("quickStart.steps.ready.desc") }}</p>
            <ul class="qs-tips">
              <li>
                <font-awesome-icon icon="check" />
                <span>{{ t("quickStart.steps.ready.tip1") }}</span>
              </li>
              <li>
                <font-awesome-icon icon="check" />
                <span>{{ t("quickStart.steps.ready.tip2") }}</span>
              </li>
              <li>
                <font-awesome-icon icon="check" />
                <span>{{ t("quickStart.steps.ready.tip3") }}</span>
              </li>
            </ul>
          </section>

          <span class="sr-only" aria-live="polite">
            {{ t("quickStart.stepOf", { current: step + 1, total: STEP_KEYS.length }) }}
          </span>
        </div>

        <div class="qs-footer">
          <button type="button" class="btn btn-ghost qs-skip" @click="$emit('close')">
            {{ t("quickStart.skip") }}
          </button>
          <div class="qs-dots" aria-hidden="true">
            <span
              v-for="(key, index) in STEP_KEYS"
              :key="key"
              class="qs-dot"
              :class="{ active: index === step }"
            />
          </div>
          <div class="qs-actions">
            <button
              v-if="step > 0"
              type="button"
              class="btn btn-secondary"
              @click="step--"
            >
              {{ t("quickStart.back") }}
            </button>
            <button
              v-if="step < STEP_KEYS.length - 1"
              type="button"
              class="btn btn-primary"
              @click="step++"
            >
              {{ t("quickStart.next") }}
            </button>
            <button v-else type="button" class="btn btn-primary" @click="$emit('close')">
              {{ t("quickStart.finish") }}
            </button>
          </div>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { useConfigStore } from "@/stores/config";
import { useQuickStartStore } from "@/stores/quickStart";
import { useToastStore } from "@/stores/toast";
import { copyText, hasTauri, tauriInvoke } from "@/utils";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const configStore = useConfigStore();
const quickStart = useQuickStartStore();
const toast = useToastStore();

const STEP_KEYS = ["theme", "connect", "ready"] as const;
const step = ref(0);
const currentKey = computed(() => STEP_KEYS[step.value]);

const overlayUrl = computed(() => {
  const raw = (configStore.config.address || "127.0.0.1").trim();
  let host = raw;
  if (raw === "0.0.0.0" || raw === "::" || raw === "[::]") {
    host = "127.0.0.1";
  } else if (raw.includes(":") && !raw.startsWith("[")) {
    host = `[${raw}]`;
  }
  return `http://${host}:${configStore.config.server_port}`;
});

const copied = ref(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

function clearCopyTimer() {
  if (copyTimer) {
    clearTimeout(copyTimer);
    copyTimer = null;
  }
}

async function handleCopy() {
  const ok = await copyText(overlayUrl.value);
  if (!ok) {
    toast.error(t("quickStart.steps.connect.copyFailed"));
    return;
  }
  copied.value = true;
  clearCopyTimer();
  copyTimer = setTimeout(() => (copied.value = false), 1500);
}

async function handlePreview() {
  const url = overlayUrl.value;
  if (hasTauri()) {
    try {
      await tauriInvoke("open_url", { url });
      return;
    } catch (e) {
      console.error("打开预览失败:", e);
    }
  }
  const opened = window.open(url, "_blank", "noopener,noreferrer");
  if (!opened) {
    toast.error(t("quickStart.steps.connect.previewFailed"));
  }
}

function goStore() {
  // 暂时收起引导去安装主题；离开商店页后自动续上，不标记为已看过。
  quickStart.pause();
  router.push("/store");
}

// 续上上次进度时不要回到第一步
let resuming = false;

watch(
  () => route.name,
  (name, previous) => {
    if (!quickStart.paused) return;
    // 等用户离开商店页再恢复，避免刚跳转就把商店盖住。
    if (String(previous) !== "store" || String(name) === "store") return;
    step.value = Math.min(step.value + 1, STEP_KEYS.length - 1);
    resuming = true;
    quickStart.resume();
  },
);

watch(
  () => props.visible,
  (visible) => {
    if (!visible) return;
    copied.value = false;
    clearCopyTimer();
    if (resuming) {
      resuming = false;
      return;
    }
    step.value = 0;
  },
);

onBeforeUnmount(clearCopyTimer);
</script>

<style scoped>
.qs-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  z-index: 9999;
}

.qs-overlay[data-state="open"] {
  animation: qs-overlay-in 0.2s ease;
}

.qs-overlay[data-state="closed"] {
  animation: qs-overlay-out 0.16s ease-in forwards;
}

.qs-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10000;
  display: flex;
  flex-direction: column;
  width: 90%;
  max-width: 560px;
  max-height: 85vh;
  background-color: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  overflow: hidden;
  outline: none;
}

.qs-dialog[data-state="open"] {
  animation: qs-content-in 0.26s cubic-bezier(0.16, 1, 0.3, 1);
}

.qs-dialog[data-state="closed"] {
  animation: qs-content-out 0.16s ease-in forwards;
}

.qs-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--ui-space-md);
  padding: var(--ui-space-md) var(--ui-space-lg);
  border-bottom: 1px solid var(--ui-border);
}

.qs-heading {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 0;
}

.qs-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: 50%;
  background: color-mix(in srgb, var(--ui-accent) 12%, transparent);
  color: var(--ui-accent);
  font-size: 16px;
}

.qs-heading h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.qs-subtitle {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--ui-text-secondary);
}

.qs-close {
  background: none;
  border: none;
  color: var(--ui-text-secondary);
  cursor: pointer;
  font-size: 16px;
  padding: var(--ui-space-xs);
  border-radius: var(--ui-radius-sm);
  flex-shrink: 0;
  transition: background-color var(--ui-transition-fast);
}

.qs-close:hover {
  background-color: var(--ui-bg-secondary);
}

.qs-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: var(--ui-space-lg);
}

.qs-step {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
}

.qs-step-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--ui-radius-md);
  background: color-mix(in srgb, var(--ui-accent) 12%, transparent);
  color: var(--ui-accent);
  font-size: 18px;
}

.qs-step-icon--success {
  background: color-mix(in srgb, var(--ui-success) 14%, transparent);
  color: var(--ui-success);
}

.qs-step h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.qs-step p {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--ui-text-secondary);
}

.qs-url-block {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.qs-url-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--ui-text-secondary);
}

.qs-url-row {
  display: flex;
  align-items: center;
  gap: var(--ui-space-sm);
  width: 100%;
}

.qs-url {
  flex: 1;
  min-width: 0;
  padding: 9px 12px;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  user-select: all;
}

.qs-obs-steps {
  margin: 0;
  padding-left: 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--ui-text-secondary);
}

.qs-obs-steps li::marker {
  color: var(--ui-accent);
  font-weight: 600;
}

.qs-tips {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.qs-tips li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--ui-text-secondary);
}

.qs-tips li svg {
  color: var(--ui-success);
  margin-top: 2px;
  flex-shrink: 0;
}

.qs-footer {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: var(--ui-space-sm);
  padding: var(--ui-space-md) var(--ui-space-lg);
  border-top: 1px solid var(--ui-border);
}

.qs-skip {
  justify-self: start;
}

.qs-dots {
  display: flex;
  align-items: center;
  gap: 6px;
}

.qs-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background-color: var(--ui-border-hover);
  transition:
    width var(--ui-transition-normal),
    background-color var(--ui-transition-normal);
}

.qs-dot.active {
  width: 18px;
  background-color: var(--ui-accent);
}

.qs-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--ui-space-sm);
}

.btn {
  padding: 9px 16px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background-color var(--ui-transition-fast),
    border-color var(--ui-transition-fast),
    color var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--ui-space-xs);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background-color: var(--ui-accent);
  color: var(--ui-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--ui-accent-hover);
}

.btn-secondary {
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
  border: 1px solid var(--ui-border);
}

.btn-secondary:hover:not(:disabled) {
  background-color: var(--ui-bg-tertiary);
  border-color: var(--ui-border-hover);
}

.btn-ghost {
  background: transparent;
  color: var(--ui-text-secondary);
}

.btn-ghost:hover:not(:disabled) {
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

@keyframes qs-overlay-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes qs-overlay-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

@keyframes qs-content-in {
  from {
    opacity: 0;
    transform: translate(-50%, -50%) scale(0.94);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
}

@keyframes qs-content-out {
  from {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
  to {
    opacity: 0;
    transform: translate(-50%, -50%) scale(0.96);
  }
}

@media (max-width: 480px) {
  .qs-footer {
    grid-template-columns: 1fr;
    row-gap: var(--ui-space-sm);
  }

  .qs-skip,
  .qs-dots,
  .qs-actions {
    justify-self: center;
  }

  .qs-actions {
    width: 100%;
  }

  .qs-actions .btn {
    flex: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .qs-overlay,
  .qs-dialog {
    animation: none !important;
  }

  .qs-dot {
    transition: none;
  }
}
</style>
