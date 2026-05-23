<template>
  <div class="app-shell">
    <TitleBar />
    <div class="layout">
      <Sidebar />
      <main class="main">
        <Toolbar v-if="session.files.length" />
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
    </div>
    <StatusBar v-if="session.files.length" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, provide, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSessionStore } from './stores/session'
import { useSettingsStore, SUPPORTED_LOCALES } from './stores/settings'
import { useSound } from './composables/useSound'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import Toolbar from './components/Toolbar.vue'
import CsvTable from './components/CsvTable.vue'
import ExportPanel from './components/ExportPanel.vue'
import ImportOverlay from './components/ImportOverlay.vue'
import SettingsModal from './components/SettingsModal.vue'
import StatusBar from './components/StatusBar.vue'

const session = useSessionStore()
const settings = useSettingsStore()
const showSettings = ref(false)
const isDragging = ref(false)

const { init: initSound, playExported } = useSound()

provide('playExported', playExported)
provide('showSettings', showSettings)


let unlistenDrop: (() => void) | null = null
let unlistenEnter: (() => void) | null = null
let unlistenLeave: (() => void) | null = null

onMounted(async () => {
  initSound()

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
})

onUnmounted(() => {
  unlistenDrop?.()
  unlistenEnter?.()
  unlistenLeave?.()
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
  flex: 1; overflow: hidden; position: relative;
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
