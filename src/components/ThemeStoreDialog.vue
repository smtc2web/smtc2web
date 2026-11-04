<template>
  <DialogRoot :open="visible" @update:open="(v) => { if (!v) $emit('close') }">
    <DialogPortal>
      <DialogOverlay class="dialog-overlay" @click="$emit('close')" />
      <DialogContent class="dialog">
        <div class="dialog__scroll">
        <!-- 加载中 -->
        <div v-if="detailLoading" class="dialog-loading">
          <font-awesome-icon icon="spinner" spin size="2x" />
          <span>{{ t('common.loading') }}</span>
        </div>

        <!-- 错误 -->
        <div v-else-if="detailError" class="dialog-error">
          <font-awesome-icon icon="exclamation-circle" />
          <p>{{ detailError }}</p>
          <button class="btn btn-primary" @click="$emit('retry')">{{ t('common.confirm') }}</button>
        </div>

        <!-- 详情内容 -->
        <template v-else-if="detail">
          <div class="dialog-header">
            <DialogTitle as="h3">{{ detail.theme.name }}</DialogTitle>
            <DialogClose as-child>
              <button class="close-btn" @click="$emit('close')">
                <font-awesome-icon icon="times" />
              </button>
            </DialogClose>
          </div>

          <div class="dialog-body">
            <!-- 截图预览 -->
            <div class="detail-preview">
              <img
                v-if="detail.theme.screenshot_url"
                :src="detail.theme.screenshot_url"
                :alt="detail.theme.name"
                decoding="async"
              />
              <div v-else class="detail-preview__placeholder">
                <font-awesome-icon icon="image" size="3x" />
                <span>{{ t('store.detail.noScreenshot') }}</span>
              </div>
            </div>

            <!-- 基本信息 -->
            <div class="detail-info">
              <p v-if="detail.theme.description" class="detail-desc">
                {{ detail.theme.description }}
              </p>
              <div class="detail-meta">
                <span class="detail-author">
                  <img
                    v-if="detail.theme.author.avatar_url"
                    :src="detail.theme.author.avatar_url"
                    :alt="detail.theme.author.login"
                    class="detail-avatar"
                  />
                  {{ detail.theme.author.login }}
                </span>
                <span class="detail-downloads">
                  <font-awesome-icon icon="download" />
                  {{ detail.theme.downloads }}
                </span>
              </div>
              <div class="detail-tags" v-if="detail.theme.tags.length">
                <span class="detail-tag" v-for="tag in detail.theme.tags" :key="tag">{{ tag }}</span>
              </div>
              <div class="detail-links" v-if="detail.theme.repo_url">
                <a :href="detail.theme.repo_url" target="_blank" rel="noopener" class="detail-link">
                  <font-awesome-icon icon="code-branch" />
                  {{ t('store.detail.viewRepo') }}
                </a>
              </div>
            </div>

            <!-- 版本列表 -->
            <div class="detail-versions">
              <h4>{{ t('store.detail.versions') }}</h4>
              <table>
                <thead>
                  <tr>
                    <th>{{ t('store.detail.version') }}</th>
                    <th>{{ t('store.detail.size') }}</th>
                    <th>{{ t('store.detail.date') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="v in detail.versions" :key="v.version">
                    <td>v{{ v.version }}</td>
                    <td>{{ formatSize(v.size) }}</td>
                    <td>{{ v.created_at }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="dialog-footer">
            <StoreInstallButton
              v-if="detail"
              :download-url="detail.theme.download_url"
              :slug="detail.theme.slug"
              @installed="handleInstalled"
            />
          </div>
        </template>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { useThemeStorePage } from "@/stores/themeStorePage";
import { useToastStore } from "@/stores/toast";
import { storeToRefs } from "pinia";
import StoreInstallButton from "@/components/StoreInstallButton.vue";

interface Props {
  visible: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  close: [];
  retry: [];
  installed: [];
}>();

const { t } = useI18n();
const store = useThemeStorePage();
const toast = useToastStore();
const { detail, detailLoading, detailError } = storeToRefs(store);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function handleInstalled(_folderName: string) {
  toast.success(t("store.installSuccess"));
  emit("installed");
  emit("close");
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 9999;
}

.dialog-overlay[data-state="open"] {
  animation: dialog-overlay-in 0.2s ease;
}

.dialog-overlay[data-state="closed"] {
  animation: dialog-overlay-out 0.16s ease-in forwards;
}

.dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10000;
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  width: 90%;
  max-width: 560px;
  max-height: 85vh;
  overflow: hidden;
  outline: none;
  display: flex;
  flex-direction: column;
}

.dialog__scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.dialog[data-state="open"] {
  animation: dialog-content-in 0.26s cubic-bezier(0.16, 1, 0.3, 1);
}

.dialog[data-state="closed"] {
  animation: dialog-content-out 0.16s ease-in forwards;
}

.dialog-loading,
.dialog-error {
  padding: var(--ui-space-xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--ui-space-md);
  min-height: 200px;
  color: var(--ui-text-secondary);
}

.dialog-error {
  color: var(--ui-error);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--ui-space-md) var(--ui-space-lg);
  border-bottom: 1px solid var(--ui-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.close-btn {
  background: none;
  border: none;
  color: var(--ui-text-secondary);
  cursor: pointer;
  font-size: 16px;
  padding: var(--ui-space-xs);
  border-radius: var(--ui-radius-sm);
  transition: background var(--ui-transition-fast);
}

.close-btn:hover {
  background: var(--ui-bg-secondary);
}

.dialog-body {
  padding: var(--ui-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-md);
}

.detail-preview {
  width: 100%;
  aspect-ratio: 16 / 9;
  border-radius: var(--ui-radius-md);
  overflow: hidden;
  background: var(--ui-bg-secondary);
}

.detail-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.detail-preview__placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--ui-space-sm);
  color: var(--ui-text-disabled);
}

.detail-info {
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-sm);
}

.detail-desc {
  font-size: 14px;
  color: var(--ui-text-secondary);
  line-height: 1.6;
  margin: 0;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: var(--ui-space-md);
  font-size: 13px;
  color: var(--ui-text-tertiary);
}

.detail-author {
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
}

.detail-downloads {
  display: flex;
  align-items: center;
  gap: 4px;
}

.detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.detail-tag {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--ui-bg-secondary);
  color: var(--ui-text-secondary);
}

.detail-links {
  display: flex;
  gap: var(--ui-space-sm);
}

.detail-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--ui-accent);
  text-decoration: none;
}

.detail-link:hover {
  text-decoration: underline;
}

.detail-versions {
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-sm);
}

.detail-versions h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.detail-versions table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.detail-versions th {
  text-align: left;
  padding: var(--ui-space-xs) var(--ui-space-sm);
  border-bottom: 1px solid var(--ui-border);
  color: var(--ui-text-tertiary);
  font-weight: 500;
}

.detail-versions td {
  padding: var(--ui-space-xs) var(--ui-space-sm);
  border-bottom: 1px solid var(--ui-border);
  color: var(--ui-text-secondary);
}

.dialog-footer {
  padding: var(--ui-space-md) var(--ui-space-lg);
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid var(--ui-border);
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  gap: var(--ui-space-xs);
}

.btn-primary {
  background: var(--ui-accent);
  color: var(--ui-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background: var(--ui-accent-hover);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@keyframes dialog-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes dialog-overlay-out {
  from { opacity: 1; }
  to { opacity: 0; }
}

@keyframes dialog-content-in {
  from { opacity: 0; transform: translate(-50%, -50%) scale(0.94); }
  to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
}

@keyframes dialog-content-out {
  from { opacity: 1; transform: translate(-50%, -50%) scale(1); }
  to { opacity: 0; transform: translate(-50%, -50%) scale(0.96); }
}

@media (prefers-reduced-motion: reduce) {
  .dialog-overlay,
  .dialog {
    animation: none !important;
  }
}
</style>
