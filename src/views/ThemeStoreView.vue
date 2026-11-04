<template>
  <div class="store-view">
    <!-- 页头：搜索与介绍 -->
    <header class="store-header">
      <h2>{{ t('store.title') }}</h2>
      <p class="store-header__desc">{{ t('store.description') }}</p>
      <form class="store-search" @submit.prevent="handleSearch">
        <input
          v-model="searchInput"
          type="search"
          :placeholder="t('store.search.placeholder')"
          :disabled="loading"
        />
        <button class="btn btn-primary" type="submit" :disabled="loading">
          <font-awesome-icon icon="info-circle" v-if="loading" spin />
          <font-awesome-icon icon="check-circle" v-else />
          {{ t('store.search.btn') }}
        </button>
      </form>
    </header>

    <!-- 工具栏：标签筛选 + 排序 -->
    <div class="store-toolbar">
      <div class="store-tags">
        <button
          type="button"
          class="chip"
          :class="{ active: !filters.tag }"
          @click="store.setTag('')"
        >
          {{ t('store.tags.all') }}
        </button>
        <button
          v-for="item in tags"
          :key="item.tag"
          type="button"
          class="chip"
          :class="{ active: filters.tag === item.tag }"
          @click="store.setTag(item.tag)"
        >
          {{ item.tag }}
          <span class="chip-count">{{ item.count }}</span>
        </button>
      </div>
      <SelectRoot :model-value="filters.sort" @update:model-value="onSortChange">
        <SelectTrigger class="store-sort">
          {{ currentSortLabel }}
        </SelectTrigger>
        <SelectPortal>
          <SelectContent
            class="store-sort-menu"
            position="popper"
            align="end"
            :side-offset="4"
          >
            <SelectViewport>
              <SelectItem
                v-for="option in SORT_OPTIONS"
                :key="option.value"
                :value="option.value"
                class="store-sort-item"
              >
                <SelectItemIndicator class="store-sort-item__check">
                  <font-awesome-icon icon="check" />
                </SelectItemIndicator>
                <SelectItemText>{{ t(option.label) }}</SelectItemText>
              </SelectItem>
            </SelectViewport>
          </SelectContent>
        </SelectPortal>
      </SelectRoot>
    </div>

    <!-- 错误提示 -->
    <p v-if="errorMsg" class="store-error">
      <font-awesome-icon icon="exclamation-triangle" />
      {{ errorMsg }}
    </p>

    <!-- 加载骨架 -->
    <div v-if="loading" class="store-grid">
      <div v-for="i in 6" :key="i" class="skeleton-card">
        <div class="skeleton-preview"></div>
        <div class="skeleton-body">
          <div class="skeleton-title"></div>
          <div class="skeleton-meta"></div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!themes.length" class="store-empty">
      <font-awesome-icon icon="paint-brush" size="3x" />
      <p>{{ t('store.empty') }}</p>
    </div>

    <!-- 主题卡片网格 -->
    <div v-else class="store-grid">
      <ThemeStoreCard
        v-for="theme in themes"
        :key="theme.slug"
        :theme="theme"
        :installing="store.isInstalling(theme.slug)"
        @open="openDetail(theme.slug)"
      />
    </div>

    <!-- 分页 -->
    <div v-if="totalPages > 1" class="store-pager">
      <button
        class="btn btn-ghost"
        :disabled="page <= 1"
        @click="goPage(page - 1)"
      >
        {{ t('store.pager.prev') }}
      </button>
      <span class="pager-info">
        {{ t('store.pager.info', { current: page, total: totalPages, count: total }) }}
      </span>
      <button
        class="btn btn-ghost"
        :disabled="page >= totalPages"
        @click="goPage(page + 1)"
      >
        {{ t('store.pager.next') }}
      </button>
    </div>

    <!-- 详情弹窗 -->
    <ThemeStoreDialog
      :visible="Boolean(selectedSlug)"
      @close="store.closeDetail()"
      @retry="selectedSlug && store.loadDetail(selectedSlug)"
      @installed="onInstalled"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectViewport,
} from "reka-ui";
import { useThemeStorePage } from "@/stores/themeStorePage";
import { useToastStore } from "@/stores/toast";
import ThemeStoreCard from "@/components/ThemeStoreCard.vue";
import ThemeStoreDialog from "@/components/ThemeStoreDialog.vue";

const { t } = useI18n();
const store = useThemeStorePage();
const toast = useToastStore();

const {
  themes,
  total,
  page,
  totalPages,
  loading,
  errorMsg,
  filters,
  tags,
  selectedSlug,
} = storeToRefs(store);

const searchInput = ref(filters.value.q || "");

const SORT_OPTIONS = [
  { value: "newest", label: "store.sort.newest" },
  { value: "popular", label: "store.sort.popular" },
  { value: "name", label: "store.sort.name" },
] as const;

const currentSortLabel = computed(
  () =>
    t(
      SORT_OPTIONS.find((option) => option.value === filters.value.sort)?.label ??
        "store.sort.newest",
    ),
);

function onSortChange(value: unknown) {
  if (typeof value === "string") store.setSort(value);
}

function handleSearch() {
  store.search(searchInput.value);
}

function goPage(p: number) {
  store.setPage(p);
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function openDetail(slug: string) {
  store.loadDetail(slug);
}

function onInstalled() {
  toast.success(t("store.installSuccess"));
}

/* 筛选条件变化时自动重新加载 */
watch(
  () => [filters.value.q, filters.value.tag, filters.value.sort, page.value],
  () => {
    store.loadThemes();
  },
);

onMounted(async () => {
  await Promise.all([store.loadThemes(), store.loadTags()]);
});

// 路由离开动画结束后再释放数据，避免离场帧突然变成空状态。
onBeforeUnmount(() => {
  store.reset();
});
</script>

<style scoped>
.store-view {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-md);
}

.store-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--ui-text-primary);
}

.store-header__desc {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--ui-text-secondary);
  line-height: 1.5;
}

.store-search {
  display: flex;
  gap: var(--ui-space-sm);
  margin-top: var(--ui-space-sm);
}

.store-search input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  background: var(--ui-bg-primary);
  color: var(--ui-text-primary);
  transition: border-color var(--ui-transition-fast);
}

.store-search input:focus {
  outline: none;
  border-color: var(--ui-accent);
}

.store-search input::placeholder {
  color: var(--ui-text-tertiary);
}

.store-toolbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--ui-space-md);
}

.store-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex: 1;
}

.chip {
  border: 1px solid var(--ui-border);
  background: var(--ui-bg-secondary);
  color: var(--ui-text-secondary);
  border-radius: 999px;
  padding: 4px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: all var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.chip:hover {
  border-color: var(--ui-border-hover);
  color: var(--ui-text-primary);
}

.chip.active {
  border-color: var(--ui-accent);
  color: var(--ui-text-primary);
  background: color-mix(in srgb, var(--ui-accent) 14%, var(--ui-bg-secondary));
}

.chip-count {
  font-size: 10px;
  color: var(--ui-text-tertiary);
}

.store-sort {
  display: inline-flex;
  align-items: center;
  padding: 6px 32px 6px 10px;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  font-size: 13px;
  background-color: var(--ui-bg-primary);
  background-image: var(--ui-select-chevron);
  background-repeat: no-repeat;
  background-position: right 10px center;
  color: var(--ui-text-primary);
  cursor: pointer;
  flex-shrink: 0;
  user-select: none;
  transition:
    border-color var(--ui-transition-fast),
    background-color var(--ui-transition-fast);
}

.store-sort:hover {
  border-color: var(--ui-border-hover);
}

.store-sort:focus-visible,
.store-sort[data-state="open"] {
  outline: none;
  border-color: var(--ui-accent);
}

.store-error {
  display: flex;
  align-items: center;
  gap: var(--ui-space-sm);
  padding: var(--ui-space-sm) var(--ui-space-md);
  background: color-mix(in srgb, var(--ui-error) 10%, transparent);
  border-radius: var(--ui-radius-md);
  color: var(--ui-error);
  font-size: 13px;
  margin: 0;
}

.store-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: var(--ui-space-md);
}

.store-empty {
  text-align: center;
  padding: var(--ui-space-xl) var(--ui-space-md);
  color: var(--ui-text-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--ui-space-md);
}

.store-pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--ui-space-md);
  margin: var(--ui-space-md) 0;
}

.pager-info {
  font-size: 13px;
  color: var(--ui-text-tertiary);
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

.btn-ghost {
  background: var(--ui-bg-secondary);
  color: var(--ui-text-secondary);
  border: 1px solid var(--ui-border);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--ui-bg-tertiary);
  color: var(--ui-text-primary);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 骨架屏 */
.skeleton-card {
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  overflow: hidden;
  box-shadow: var(--ui-shadow-md);
}

.skeleton-preview {
  width: 100%;
  aspect-ratio: 16 / 9;
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-body {
  padding: var(--ui-space-md);
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-sm);
}

.skeleton-title {
  height: 18px;
  width: 70%;
  border-radius: var(--ui-radius-sm);
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-meta {
  height: 14px;
  width: 50%;
  border-radius: var(--ui-radius-sm);
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

@media (max-width: 768px) {
  .store-grid {
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  }

  .store-toolbar {
    flex-direction: column;
  }
}

@media (prefers-reduced-motion: reduce) {
  .skeleton-preview,
  .skeleton-title,
  .skeleton-meta {
    animation: none;
  }
}
</style>

<style>
/* 下拉菜单经 SelectPortal 挂到 body，作用域样式无法稳定命中，使用全局类名。 */
.store-sort-menu {
  z-index: 9000;
  min-width: var(--reka-select-trigger-width, 120px);
  padding: 4px;
  background: var(--ui-bg-card);
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  box-shadow: var(--ui-shadow-lg);
  color: var(--ui-text-primary);
}

.store-sort-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 7px 10px 7px 28px;
  border-radius: var(--ui-radius-sm);
  font-size: 13px;
  white-space: nowrap;
  color: var(--ui-text-primary);
  outline: none;
  cursor: pointer;
  user-select: none;
}

.store-sort-item[data-highlighted] {
  background: var(--ui-bg-secondary);
}

.store-sort-item[data-state="checked"] {
  color: var(--ui-accent);
}

.store-sort-item__check {
  position: absolute;
  left: 9px;
  display: inline-flex;
  align-items: center;
  font-size: 11px;
}
</style>
