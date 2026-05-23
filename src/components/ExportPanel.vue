<template>
  <aside class="drawer">
    <div class="drawer-header">{{ t('export.title') }}</div>
    <div class="drawer-body">

      <div class="field">
        <label class="form-label">{{ t('export.format') }}</label>
        <select class="form-select" v-model="format">
          <option value="single">{{ t('export.format.single') }}</option>
          <option value="per_file">{{ t('export.format.perFile') }}</option>
        </select>
      </div>

      <div class="field">
        <label class="form-label">{{ t('export.langs') }}</label>
        <div class="check-list">
          <label v-for="lang in session.languages" :key="lang" class="check-item">
            <input type="checkbox" :value="lang" v-model="selectedLangs" />
            {{ langLabel(lang) }} ({{ lang.toUpperCase() }})
          </label>
        </div>
      </div>

      <div class="field">
        <label class="form-label">{{ t('export.separator') }}</label>
        <select class="form-select" v-model="separator">
          <option value=";">{{ t('export.sep.semicolon') }}</option>
          <option value=",">{{ t('export.sep.comma') }}</option>
          <option value="&#9;">{{ t('export.sep.tab') }}</option>
        </select>
      </div>

      <div v-if="session.progress" class="field">
        <ProgressBar :current="session.progress.current" :total="session.progress.total" />
      </div>

      <p v-if="session.error" class="error-msg">{{ session.error }}</p>
    </div>

    <div class="drawer-footer">
      <button class="btn btn-primary btn-full" @click="doExport">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M2 10V12h10v-2M7 2v7M4 6l3 3 3-3"/>
        </svg>
        {{ t('export.btn') }}
      </button>
      <button class="btn btn-secondary btn-full" @click="doReimport">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M7 12V3M4 9l3 3 3-3M2 12h10"/>
        </svg>
        {{ t('reimport.btn') }}
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch, inject } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'
import ProgressBar from './ProgressBar.vue'

const session = useSessionStore()
const { t } = useI18n()
const playExported = inject<() => void>('playExported', () => {})

const format = ref<'single' | 'per_file'>('single')
const separator = ref(';')
const selectedLangs = ref<string[]>([...session.languages])

watch(() => session.languages, (langs) => {
  selectedLangs.value = [...langs]
})

const langLabels: Record<string, string> = {
  ja: '日本語',
  es: 'Español',
  en: 'English',
  fr: 'Français',
  de: 'Deutsch',
  it: 'Italiano'
}
function langLabel(l: string) { return langLabels[l] ?? l }

async function doExport() {
  if (format.value === 'single') {
    const path = await save({ filters: [{ name: 'CSV', extensions: ['csv'] }] })
    if (!path) return
    await session.exportCsv(path, selectedLangs.value, format.value, separator.value)
  } else {
    const dir = await open({ directory: true })
    if (!dir || Array.isArray(dir)) return
    await session.exportCsv(dir as string, selectedLangs.value, format.value, separator.value)
  }
  playExported()
}

async function doReimport() {
  const csvPath = await open({ filters: [{ name: 'CSV', extensions: ['csv'] }] })
  if (!csvPath || Array.isArray(csvPath)) return
  await session.importCsvRows(csvPath, separator.value)

  const outDir = await open({ directory: true })
  if (!outDir || Array.isArray(outDir)) return
  await session.writeCfgbin(outDir as string)
}
</script>

<style scoped>
.drawer {
  width: 220px;
  background: var(--bg-panel);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.drawer-header {
  padding: 12px;
  border-bottom: 1px solid var(--border);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-2);
}
.drawer-body {
  padding: 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}
.field { display: flex; flex-direction: column; gap: 4px; }
.check-list { display: flex; flex-direction: column; gap: 5px; }
.check-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-2);
  cursor: pointer;
}
.check-item input[type=checkbox] { accent-color: var(--accent); cursor: pointer; }
.drawer-footer {
  padding: 12px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.error-msg { font-size: 11px; color: var(--danger); }
</style>
