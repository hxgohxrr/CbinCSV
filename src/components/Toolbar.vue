<template>
  <div class="toolbar">
    <span class="label">{{ t('toolbar.langs') }}</span>
    <div class="lang-chips">
      <button
        v-for="lang in session.languages"
        :key="lang"
        :class="['chip', session.visibleLangs.includes(lang) && 'active']"
        @click="toggleLang(lang)"
      >{{ lang.toUpperCase() }}</button>
    </div>
    <div class="sep" />
    <button class="btn-add" @click="addFiles" :title="t('sidebar.import')">
      <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
        <line x1="7" y1="2" x2="7" y2="12"/>
        <line x1="2" y1="7" x2="12" y2="7"/>
      </svg>
    </button>
    <div class="search">
      <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="6" cy="6" r="4"/>
        <line x1="9.5" y1="9.5" x2="12.5" y2="12.5"/>
      </svg>
      <input v-model="session.searchQuery" :placeholder="t('toolbar.search')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

const session = useSessionStore()
const { t } = useI18n()

async function addFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'cfg.bin', extensions: ['bin'] }]
  })
  if (!selected) return
  await session.loadFiles(Array.isArray(selected) ? selected : [selected])
}

function toggleLang(lang: string) {
  const idx = session.visibleLangs.indexOf(lang)
  if (idx >= 0) {
    if (session.visibleLangs.length > 1) session.visibleLangs.splice(idx, 1)
  } else {
    session.visibleLangs.push(lang)
  }
}
</script>

<style scoped>
.toolbar {
  height: 44px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 10px;
  flex-shrink: 0;
}
.label { font-size: 11px; color: var(--text-2); font-weight: 500; white-space: nowrap; }
.lang-chips { display: flex; gap: 4px; }
.chip {
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  cursor: pointer;
  border: 1px solid var(--border-md);
  background: transparent;
  color: var(--text-2);
  transition: all 0.15s;
  font-family: 'Outfit', sans-serif;
}
.chip.active { background: var(--accent-dim); border-color: var(--accent-soft); color: var(--accent); }
.sep { flex: 1; }
.search {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-input);
  border: 1px solid var(--border-md);
  border-radius: var(--radius);
  padding: 0 10px;
  height: 28px;
}
.btn-add {
  width: 28px; height: 28px; padding: 0;
  border: 1px solid var(--border-md);
  border-radius: var(--radius);
  background: transparent;
  color: var(--text-2);
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.15s, color 0.15s;
  flex-shrink: 0;
}
.btn-add:hover { background: var(--accent-dim); color: var(--accent); border-color: var(--accent-soft); }
.btn-add svg { width: 12px; height: 12px; }
.search svg { width: 12px; height: 12px; color: var(--text-3); flex-shrink: 0; }
.search input {
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-1);
  font-family: 'Outfit', sans-serif;
  font-size: 12px;
  width: 160px;
}
.search input::placeholder { color: var(--text-3); }
</style>
