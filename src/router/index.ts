import { createRouter, createWebHistory } from 'vue-router'

// 懒加载路由组件
const ThemesView = () => import('@/views/ThemesView.vue')
const ThemeStoreView = () => import('@/views/ThemeStoreView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/themes'
    },
    {
      path: '/themes',
      name: 'themes',
      component: ThemesView
    },
    {
      path: '/store',
      name: 'store',
      component: ThemeStoreView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    }
  ]
})

export default router
