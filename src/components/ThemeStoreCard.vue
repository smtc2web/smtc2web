<template>
  <div class="store-card" @click="$emit('open')">
    <div class="store-card__preview">
      <img
        v-if="theme.screenshot_url"
        :src="theme.screenshot_url"
        :alt="theme.name"
        loading="lazy"
        decoding="async"
      />
      <div v-else class="store-card__placeholder">
        <font-awesome-icon icon="image" />
      </div>
      <div v-if="installing" class="store-card__installing">
        <font-awesome-icon icon="spinner" spin />
        <span>{{ t('store.card.installing') }}</span>
      </div>
    </div>
    <div class="store-card__body">
      <h3 class="store-card__name">{{ theme.name }}</h3>
      <p v-if="theme.description" class="store-card__desc">{{ theme.description }}</p>
      <div class="store-card__meta">
        <span class="store-card__author">
          <img
            v-if="theme.author.avatar_url"
            :src="theme.author.avatar_url"
            :alt="theme.author.login"
            class="store-card__avatar"
          />
          {{ theme.author.login }}
        </span>
        <span class="store-card__version">v{{ theme.version }}</span>
      </div>
      <div class="store-card__footer">
        <div class="store-card__tags" v-if="theme.tags.length">
          <span class="store-card__tag" v-for="tag in theme.tags.slice(0, 3)" :key="tag">
            {{ tag }}
          </span>
        </div>
        <span class="store-card__downloads" :title="t('store.card.downloads')">
          <font-awesome-icon icon="download" />
          {{ theme.downloads }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { ThemeSummary } from "@/types/store";

interface Props {
  theme: ThemeSummary;
  installing?: boolean;
}

defineProps<Props>();
defineEmits<{
  open: [];
}>();

const { t } = useI18n();
</script>

<style scoped>
.store-card {
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  overflow: hidden;
  box-shadow: var(--ui-shadow-md);
  cursor: pointer;
  transition: transform var(--ui-transition-normal), box-shadow var(--ui-transition-normal);
  border: 2px solid transparent;
  display: flex;
  flex-direction: column;
  /* 屏幕外的卡片跳过渲染与图片解码，减少长列表的渲染内存。 */
  content-visibility: auto;
  contain-intrinsic-size: auto 340px;
}

.store-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--ui-shadow-lg);
  border-color: var(--ui-border-hover);
}

.store-card__preview {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: linear-gradient(135deg, var(--ui-bg-secondary) 0%, var(--ui-bg-tertiary) 100%);
  overflow: hidden;
}

.store-card__preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.store-card__placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  color: var(--ui-text-disabled);
}

.store-card__installing {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
}

.store-card__body {
  padding: var(--ui-space-md);
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-sm);
  flex: 1;
}

.store-card__name {
  font-size: 15px;
  font-weight: 600;
  color: var(--ui-text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.store-card__desc {
  font-size: 12px;
  color: var(--ui-text-secondary);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.store-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--ui-text-tertiary);
}

.store-card__author {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  max-width: 60%;
}

.store-card__avatar {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  flex-shrink: 0;
}

.store-card__version {
  background: var(--ui-bg-secondary);
  padding: 2px 6px;
  border-radius: var(--ui-radius-sm);
  font-size: 11px;
  font-weight: 500;
  flex-shrink: 0;
}

.store-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: auto;
}

.store-card__tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  flex: 1;
  overflow: hidden;
}

.store-card__tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 999px;
  background: var(--ui-bg-secondary);
  color: var(--ui-text-secondary);
  white-space: nowrap;
}

.store-card__downloads {
  font-size: 12px;
  color: var(--ui-text-tertiary);
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}
</style>
