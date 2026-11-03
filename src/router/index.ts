import { createRouter, createWebHistory } from 'vue-router'

// 懒加载路由组件
const ThemesView = () => import('@/views/ThemesView.vue')
const ThemeStoreView = () => import('@/views/ThemeStoreView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')
const AboutView = () => import('@/views/AboutView.vue')

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
    },
    {
      path: '/about',
      name: 'about',
      component: AboutView
    }
  ]
})

export default router
