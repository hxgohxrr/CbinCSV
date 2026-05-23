import { defineStore } from 'pinia'
import { ref, watchEffect } from 'vue'

export const ACCENT_PRESETS = [
  '#5BA3E0', '#4ECDC4', '#E8A830', '#E07864', '#9B7EDE', '#5B9B7C'
]

export const SUPPORTED_LOCALES = ['es', 'en', 'fr', 'de', 'ja', 'it']

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light' | 'system'>(
    (localStorage.getItem('theme') as any) ?? 'dark'
  )
  const accent = ref(localStorage.getItem('accent') ?? '#5BA3E0')
  const locale = ref(localStorage.getItem('locale') ?? 'es')
  const showSplash = ref(localStorage.getItem('showSplash') !== 'false')
  const soundVolume = ref(Number(localStorage.getItem('soundVolume') ?? 50))

  watchEffect(() => localStorage.setItem('theme', theme.value))
  watchEffect(() => localStorage.setItem('accent', accent.value))
  watchEffect(() => localStorage.setItem('locale', locale.value))
  watchEffect(() => {
    localStorage.setItem('showSplash', String(showSplash.value))
    const el = document.getElementById('splash')
    if (el && !showSplash.value) el.remove()
  })
  watchEffect(() => localStorage.setItem('soundVolume', String(soundVolume.value)))

  watchEffect(() => {
    const t = theme.value
    const effective = t === 'system'
      ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
      : t
    document.documentElement.setAttribute('data-theme', effective)
  })

  watchEffect(() => {
    const acc = accent.value
    const hex = acc.replace('#', '')
    const r = parseInt(hex.slice(0, 2), 16)
    const g = parseInt(hex.slice(2, 4), 16)
    const b = parseInt(hex.slice(4, 6), 16)
    document.documentElement.style.setProperty('--accent', acc)
    document.documentElement.style.setProperty('--accent-dim', `rgba(${r},${g},${b},0.10)`)
    document.documentElement.style.setProperty('--accent-soft', `rgba(${r},${g},${b},0.22)`)
  })

  return { theme, accent, locale, showSplash, soundVolume }
})
