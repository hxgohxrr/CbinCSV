<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="themeOverrides" :inline-theme-disabled="true">
  <NMessageProvider>
  <div class="app-shell">
    <TitleBar />
    <div class="layout">
      <Sidebar />
      <main class="main">
        <Toolbar v-if="session.files.length" :show-replace="showReplace" @toggle-replace="showReplace = !showReplace" />
        <SearchReplace v-if="session.files.length && showReplace" @close="showReplace = false" />
        <div class="main-content" :class="{ dragging: isDragging }">
          <ImportOverlay v-if="!session.files.length" :is-dragging="isDragging" />
          <CsvTable v-else />
          <div v-if="isDragging && session.files.length" class="drop-overlay">
            <div class="drop-hint">
              <svg viewBox="0 0 40 40" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="6" y="4" width="28" height="32" rx="3"/>
                <line x1="13" y1="14" x2="27" y2="14"/>
                <line x1="13" y1="20" x2="27" y2="20"/>
              </svg>
              <span>Soltar para añadir archivos</span>
            </div>
          </div>
        </div>
      </main>
      <ExportPanel v-if="session.files.length" />
      <McpPanel v-if="session.files.length" />
    </div>
    <StatusBar v-if="session.files.length" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
  </div>
  </NMessageProvider>
  </NConfigProvider>
</template>

<script setup lang="ts">
import { ref, computed, provide, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { NConfigProvider, NMessageProvider, darkTheme, type GlobalThemeOverrides } from 'naive-ui'
import { useSessionStore } from './stores/session'
import { useSettingsStore, SUPPORTED_LOCALES } from './stores/settings'
import { useSound } from './composables/useSound'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import Toolbar from './components/Toolbar.vue'
import CsvTable from './components/CsvTable.vue'
import ExportPanel from './components/ExportPanel.vue'
import McpPanel from './components/McpPanel.vue'
import ImportOverlay from './components/ImportOverlay.vue'
import SettingsModal from './components/SettingsModal.vue'
import StatusBar from './components/StatusBar.vue'
import SearchReplace from './components/SearchReplace.vue'

const session = useSessionStore()
const settings = useSettingsStore()
const showSettings = ref(false)
const showReplace = ref(false)
const isDragging = ref(false)

const naiveTheme = computed(() => {
  const t = settings.theme
  const effective = t === 'system'
    ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
    : t
  return effective === 'dark' ? darkTheme : null
})

const themeOverrides = computed<GlobalThemeOverrides>(() => {
  const acc = settings.accent
  const isDark = naiveTheme.value !== null
  return {
    common: {
      primaryColor: acc,
      primaryColorHover: acc + 'CC',
      primaryColorPressed: acc + '88',
      primaryColorSuppl: acc,
      fontFamily: "'Outfit', sans-serif",
      fontFamilyMono: "'JetBrains Mono', monospace",
      fontSize: '13px',
      borderRadius: '6px',
      bodyColor:    isDark ? '#0D0D0D' : '#FFFFFF',
      cardColor:    isDark ? '#1A1A1A' : '#FFFFFF',
      modalColor:   isDark ? '#141414' : '#F5F5F5',
      popoverColor: isDark ? '#1A1A1A' : '#FFFFFF',
      inputColor:   isDark ? '#090909' : '#FAFAFA',
      textColor1:   isDark ? '#E8E8E8' : '#111111',
      textColor2:   isDark ? '#888888' : '#555555',
      textColor3:   isDark ? '#444444' : '#999999',
      borderColor:  isDark ? '#2A2A2A' : '#D0D0D0',
      dividerColor: isDark ? '#1E1E1E' : '#E0E0E0',
    }
  }
})

const { init: initSound, playExported } = useSound()

provide('playExported', playExported)
provide('showSettings', showSettings)


let unlistenDrop: (() => void) | null = null
let unlistenEnter: (() => void) | null = null
let unlistenLeave: (() => void) | null = null
let unlistenMcp: (() => void) | null = null

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'h' && session.files.length) {
    e.preventDefault()
    showReplace.value = !showReplace.value
  }
}

onMounted(async () => {
  initSound()
  window.addEventListener('keydown', onKeydown)

  if (!localStorage.getItem('locale')) {
    try {
      const sysLocale = await invoke<string>('get_system_locale')
      const lang = sysLocale.split(/[-_]/)[0]
      settings.locale = SUPPORTED_LOCALES.includes(lang) ? lang : 'es'
    } catch {
      settings.locale = 'es'
    }
  }

  unlistenEnter = await listen('tauri://drag-enter', () => {
    isDragging.value = true
  })

  unlistenLeave = await listen('tauri://drag-leave', () => {
    isDragging.value = false
  })

  unlistenDrop = await listen<{ paths: string[] }>('tauri://drag-drop', (e) => {
    isDragging.value = false
    const paths = ((e.payload as any).paths as string[] ?? [])
      .filter(p => p.endsWith('.cfg.bin') || p.endsWith('.bin'))
    if (paths.length) session.loadFiles(paths)
  })

  unlistenMcp = await listen<{ ev_name: string; index: number; lang: string; value: string }>(
    'mcp_entry_changed',
    (e) => {
      const { ev_name, index, lang, value } = e.payload
      session.updateEntry(ev_name, lang, index, value)
    }
  )
})

onUnmounted(() => {
  unlistenDrop?.()
  unlistenEnter?.()
  unlistenLeave?.()
  unlistenMcp?.()
  window.removeEventListener('keydown', onKeydown)
})
</script>

<style scoped>
.app-shell {
  display: flex; flex-direction: column;
  height: 100vh; overflow: hidden;
}
.layout {
  display: flex; flex: 1; overflow: hidden;
}
.main {
  flex: 1; display: flex; flex-direction: column; overflow: hidden;
}
.main-content {
  flex: 1; display: flex; flex-direction: column; overflow: hidden; position: relative;
  transition: outline 0.1s;
}
.main-content.dragging {
  outline: 2px dashed var(--accent-soft);
  outline-offset: -4px;
}
.drop-overlay {
  position: absolute; inset: 0;
  background: var(--accent-dim);
  display: flex; align-items: center; justify-content: center;
  pointer-events: none; z-index: 50;
}
.drop-hint {
  display: flex; flex-direction: column; align-items: center; gap: 12px;
  color: var(--accent);
}
.drop-hint svg { width: 48px; height: 48px; }
.drop-hint span { font-size: 14px; font-weight: 500; }
</style>
