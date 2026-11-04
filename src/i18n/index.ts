import { createI18n } from 'vue-i18n'
// 只静态打包回退语言：其余语言按需动态加载，避免启动时把 11 份语言包
// 全部解析成 JS 对象常驻内存。
import zhCN from './locales/zh-CN.json'

export const SUPPORTED_LOCALES = [
  'zh-CN',
  'zh-TW',
  'en',
  'de-DE',
  'fr-FR',
  'it-IT',
  'ja-JP',
  'ko-KR',
  'ko-KP',
  'nl-NL',
  'ru-RU',
] as const

export type LocaleCode = (typeof SUPPORTED_LOCALES)[number]

const localeLoaders: Record<string, () => Promise<{ default: Record<string, unknown> }>> = {
  'zh-CN': async () => ({ default: zhCN as Record<string, unknown> }),
  'zh-TW': () => import('./locales/zh-TW.json'),
  en: () => import('./locales/en.json'),
  'de-DE': () => import('./locales/de-DE.json'),
  'fr-FR': () => import('./locales/fr-FR.json'),
  'it-IT': () => import('./locales/it-IT.json'),
  'ja-JP': () => import('./locales/ja-JP.json'),
  'ko-KR': () => import('./locales/ko-KR.json'),
  'ko-KP': () => import('./locales/ko-KP.json'),
  'nl-NL': () => import('./locales/nl-NL.json'),
  'ru-RU': () => import('./locales/ru-RU.json'),
}

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
  },
})

const loadedLocales = new Set<string>(['zh-CN'])

function isSupportedLocale(locale: string): locale is LocaleCode {
  return (SUPPORTED_LOCALES as readonly string[]).includes(locale)
}

/** 动态加载并注册语言包（重复调用不会重复加载）。 */
export async function loadLocaleMessages(locale: string): Promise<void> {
  const target = isSupportedLocale(locale) ? locale : 'zh-CN'
  if (loadedLocales.has(target)) return

  const loader = localeLoaders[target]
  if (!loader) return

  const module = await loader()
  i18n.global.setLocaleMessage(target, module.default as never)
  loadedLocales.add(target)
}

/** 切换全局 i18n 语言。调用前需确保语言包已通过 loadLocaleMessages 加载。 */
export function setI18nLocale(locale: string): void {
  const target = isSupportedLocale(locale) ? locale : 'zh-CN'
  const localeRef = i18n.global.locale as unknown as { value: string }
  localeRef.value = target
}

export default i18n
