import { useSettingsStore } from '../stores/settings'

const sounds: Record<string, HTMLAudioElement | null> = {
  click: null,
  hover: null,
  loaded: null,
  exported: null,
}

let initialized = false

function loadSounds() {
  for (const key of Object.keys(sounds)) {
    try {
      const audio = new Audio(`/sounds/${key}.ogg`)
      audio.preload = 'auto'
      sounds[key] = audio
    } catch {
      sounds[key] = null
    }
  }
}

function play(key: string, volume: number) {
  const audio = sounds[key]
  if (!audio || volume === 0) return
  try {
    audio.volume = volume / 100
    audio.currentTime = 0
    audio.play().catch(() => {})
  } catch {}
}

export function useSound() {
  const settings = useSettingsStore()

  function init() {
    if (initialized) return
    initialized = true
    loadSounds()

    document.addEventListener('mousedown', (e) => {
      const target = e.target as HTMLElement
      if (target.closest('button, [role="button"], .chip, .swatch, .theme-tab')) {
        play('click', settings.soundVolume)
      }
    }, true)

    document.addEventListener('mouseenter', (e) => {
      const target = e.target as HTMLElement
      if (target.matches('button, [role="button"], .chip, .swatch, .theme-tab')) {
        play('hover', settings.soundVolume)
      }
    }, true)
  }

  function playLoaded() { play('loaded', settings.soundVolume) }
  function playExported() { play('exported', settings.soundVolume) }

  return { init, playLoaded, playExported }
}
