<template>
  <ToastProvider
    :swipe-direction="'right'"
    :duration="defaultDuration"
    :label="'Notifications'"
  >
    <Teleport to="body">
      <div class="toast-container" :class="position">
        <ToastViewport as="div" class="toast-viewport">
          <ToastRoot
            v-for="toast in toasts"
            :key="toast.id"
            class="toast-root"
            :duration="toast.duration"
            @update:open="(v) => { if (!v) scheduleRemove(toast.id) }"
          >
            <div
              class="toast"
              :class="[toast.type, { 'is-confirm': toast.type === 'confirm' }]"
            >
              <div class="toast-content">
                <div class="toast-icon">
                  <font-awesome-icon :icon="getIcon(toast.type)" />
                </div>
                <div class="toast-message">
                  <ToastTitle v-if="toast.title" class="toast-title">{{
                    toast.title
                  }}</ToastTitle>
                  <ToastDescription class="toast-text">{{
                    toast.message
                  }}</ToastDescription>

                  <!-- 确认按钮组 -->
                  <div
                    v-if="toast.type === 'confirm' && toast.actions"
                    class="toast-actions"
                  >
                    <button
                      class="toast-btn toast-btn-confirm"
                      @click="confirmToast(toast.id, true)"
                    >
                      {{ toast.actions.confirmText }}
                    </button>
                    <button
                      class="toast-btn toast-btn-cancel"
                      @click="confirmToast(toast.id, false)"
                    >
                      {{ toast.actions.cancelText }}
                    </button>
                  </div>
                </div>
              </div>

              <ToastClose
                v-if="toast.type !== 'confirm'"
                class="toast-close"
                :aria-label="'Close'"
              >
                <font-awesome-icon icon="times" />
              </ToastClose>

              <div
                v-if="toast.duration > 0 && toast.type !== 'confirm'"
                class="toast-progress"
                :style="{ animationDuration: `${toast.duration}ms` }"
              />
            </div>
          </ToastRoot>
        </ToastViewport>
      </div>
    </Teleport>
  </ToastProvider>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import {
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastTitle,
  ToastViewport,
} from 'reka-ui'
import { useToastStore } from '@/stores/toast'

const toastStore = useToastStore()
const { toasts, position, defaultDuration } = storeToRefs(toastStore)
const { removeToast, confirmToast } = toastStore

// Reka UI 的 Presence 会等待退出动画结束后才卸载节点，因此这里延迟从列表
// 移除，让 data-state='closed' 的离场动画能够完整播放。
function scheduleRemove(id: string) {
  window.setTimeout(() => removeToast(id), 220)
}

function getIcon(type: string): string {
  switch (type) {
    case 'success':
      return 'check-circle'
    case 'error':
      return 'exclamation-circle'
    case 'warning':
    case 'confirm':
      return 'exclamation-triangle'
    case 'info':
    default:
      return 'info-circle'
  }
}
</script>

<style scoped>
/*
 * 注意：这些样式依赖 Toast.vue 自身的作用域属性。
 * reka-ui 的 ToastViewport 会把 class 透传到内部元素，导致作用域属性无法命中，
 * 因此这里用一个外层 div 承载定位与布局，并通过 :deep() 重置内部视口元素。
 */
.toast-container {
  position: fixed;
  z-index: 9999;
  pointer-events: none;
  /* 进出场位移方向，由位置类覆盖（右侧默认） */
  --toast-enter-x: calc(100% + 16px);
  --toast-enter-y: 0px;
}

.toast-container.top-right {
  top: 20px;
  right: 20px;
}

.toast-container.top-left {
  top: 20px;
  left: 20px;
  --toast-enter-x: calc(-100% - 16px);
}

.toast-container.bottom-right {
  bottom: 20px;
  right: 20px;
}

.toast-container.bottom-left {
  bottom: 20px;
  left: 20px;
  --toast-enter-x: calc(-100% - 16px);
}

.toast-container.top-center {
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  --toast-enter-x: 0px;
  --toast-enter-y: -12px;
}

.toast-container.bottom-center {
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  --toast-enter-x: 0px;
  --toast-enter-y: 12px;
}

.toast-container :deep(.toast-viewport) {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0;
  margin: 0;
  list-style: none;
}

.toast-container.bottom-right :deep(.toast-viewport),
.toast-container.bottom-left :deep(.toast-viewport),
.toast-container.bottom-center :deep(.toast-viewport) {
  flex-direction: column-reverse;
}

/*
 * Reka UI 会在 ToastRoot 上暴露 data-state / data-swipe 属性，并借助 Presence
 * 等待退出动画播放完毕后才卸载节点，因此可以通过纯 CSS 动画驱动进/出场。
 */
.toast-container :deep(.toast-root[data-state='open']) {
  animation: toast-enter 0.28s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-container :deep(.toast-root[data-state='closed']) {
  animation: toast-leave 0.18s ease-in forwards;
}

/* 滑动关闭：Reka UI 提供的 CSS 变量与 data-swipe 状态 */
.toast-container :deep(.toast-root[data-swipe='move']) {
  transform: translateX(var(--reka-toast-swipe-move-x, 0));
}

.toast-container :deep(.toast-root[data-swipe='cancel']) {
  transform: translateX(0);
  transition: transform 0.2s ease-out;
}

.toast-container :deep(.toast-root[data-swipe='end']) {
  animation: toast-swipe-out 0.2s ease-out forwards;
}

@keyframes toast-enter {
  from {
    opacity: 0;
    transform: translate(var(--toast-enter-x), var(--toast-enter-y));
  }
  to {
    opacity: 1;
    transform: translate(0, 0);
  }
}

@keyframes toast-leave {
  from {
    opacity: 1;
    transform: translate(0, 0);
  }
  to {
    opacity: 0;
    transform: translate(var(--toast-enter-x), var(--toast-enter-y));
  }
}

@keyframes toast-swipe-out {
  from {
    transform: translateX(var(--reka-toast-swipe-end-x, 100%));
  }
  to {
    transform: translateX(calc(100% + 24px));
    opacity: 0;
  }
}

.toast {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 300px;
  max-width: 400px;
  padding: 16px;
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  pointer-events: auto;
  overflow: hidden;
  border-left: 4px solid transparent;
}

.toast.success {
  border-left-color: var(--ui-success);
}

.toast.error {
  border-left-color: var(--ui-error);
}

.toast.warning,
.toast.confirm {
  border-left-color: var(--ui-warning);
}

.toast.info {
  border-left-color: var(--ui-accent);
}

.toast-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex: 1;
}

.toast-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  font-size: 20px;
}

.toast.success .toast-icon {
  color: var(--ui-success);
}

.toast.error .toast-icon {
  color: var(--ui-error);
}

.toast.warning .toast-icon,
.toast.confirm .toast-icon {
  color: var(--ui-warning);
}

.toast.info .toast-icon {
  color: var(--ui-accent);
}

.toast-message {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--ui-text-primary);
  margin-bottom: 4px;
}

.toast-text {
  font-size: 13px;
  color: var(--ui-text-secondary);
  line-height: 1.5;
  word-wrap: break-word;
}

.toast-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.toast-btn {
  padding: 6px 12px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toast-btn-confirm {
  background-color: var(--ui-error);
  color: white;
}

.toast-btn-confirm:hover {
  background-color: var(--ui-error-hover);
}

.toast-btn-cancel {
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
}

.toast-btn-cancel:hover {
  background-color: var(--ui-bg-tertiary);
}

.toast-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--ui-text-secondary);
  cursor: pointer;
  border-radius: var(--ui-radius-sm);
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.toast-close:hover {
  background: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
}

.toast-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: currentColor;
  opacity: 0.3;
  animation: progress linear forwards;
}

.toast.success .toast-progress {
  background: var(--ui-success);
}

.toast.error .toast-progress {
  background: var(--ui-error);
}

.toast.warning .toast-progress {
  background: var(--ui-warning);
}

.toast.info .toast-progress {
  background: var(--ui-accent);
}

@keyframes progress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

/* 尊重系统的“减少动态效果”偏好 */
@media (prefers-reduced-motion: reduce) {
  .toast-container :deep(.toast-root) {
    animation: none !important;
    transition: none !important;
  }

  .toast-progress {
    animation: none !important;
  }
}

/* 响应式 */
@media (max-width: 480px) {
  .toast-container {
    left: 10px !important;
    right: 10px !important;
    top: 10px !important;
    bottom: auto !important;
    transform: none !important;
  }

  .toast {
    min-width: auto;
    max-width: none;
    width: 100%;
  }
}
</style>
