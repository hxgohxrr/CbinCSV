<template>
  <aside class="drawer">
    <div class="drawer-header">{{ t('export.title') }}</div>
    <div class="drawer-body">

      <div class="field">
        <span class="field-label">{{ t('export.fileFormat') }}</span>
        <NSelect v-model:value="fileFormat" :options="fileFormatOptions" size="small" />
      </div>

      <div class="field">
        <span class="field-label">{{ t('export.format') }}</span>
        <NSelect v-model:value="format" :options="formatOptions" size="small" />
      </div>

      <div class="field">
        <span class="field-label">{{ t('export.langs') }}</span>
        <NSpace vertical :size="6">
          <NCheckbox
            v-for="lang in session.languages"
            :key="lang"
            :value="lang"
            :checked="selectedLangs.includes(lang)"
            @update:checked="(v) => toggleLang(lang, v)"
          >
            {{ langLabel(lang) }} ({{ lang.toUpperCase() }})
          </NCheckbox>
        </NSpace>
      </div>

      <div v-if="fileFormat === 'csv'" class="field">
        <span class="field-label">{{ t('export.separator') }}</span>
        <NSelect v-model:value="separator" :options="sepOptions" size="small" />
      </div>

      <NProgress
        v-if="session.progress"
        type="line"
        :percentage="progressPct"
        :height="4"
        :border-radius="2"
        :show-indicator="false"
        style="margin-top: 4px"
      />

      <NAlert v-if="session.error" type="error" :title="session.error" :bordered="false" size="small" />

    </div>

    <div class="drawer-footer">
      <NButton type="primary" block :loading="!!session.progress" @click="doExport">
        <template #icon>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2 10V12h10v-2M7 2v7M4 6l3 3 3-3"/>
          </svg>
        </template>
        {{ t('export.btn') }}
      </NButton>
      <NButton block :loading="!!session.progress" @click="doReimport" style="margin-top: 6px">
        <template #icon>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M7 12V3M4 9l3 3 3-3M2 12h10"/>
          </svg>
        </template>
        {{ t('reimport.btn') }}
      </NButton>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch, inject } from 'vue'
import { NSelect, NCheckbox, NSpace, NProgress, NButton, NAlert } from 'naive-ui'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

const session = useSessionStore()
const { t } = useI18n()
const playExported = inject<() => void>('playExported', () => {})

type FileFormat = 'csv' | 'json' | 'yaml' | 'toml' | 'xml'

const fileFormat = ref<FileFormat>('csv')
const format = ref<'single' | 'per_file'>('single')
const separator = ref(';')
const selectedLangs = ref<string[]>([...session.languages])

watch(() => session.languages, (langs) => { selectedLangs.value = [...langs] })

const fileFormatOptions = computed(() => [
  { label: t('export.format.csv'),  value: 'csv' },
  { label: t('export.format.json'), value: 'json' },
  { label: t('export.format.yaml'), value: 'yaml' },
  { label: t('export.format.toml'), value: 'toml' },
  { label: t('export.format.xml'),  value: 'xml' },
])
const formatOptions = computed(() => [
  { label: t('export.format.single'),  value: 'single' },
  { label: t('export.format.perFile'), value: 'per_file' },
])
const sepOptions = computed(() => [
  { label: t('export.sep.semicolon'), value: ';' },
  { label: t('export.sep.comma'),     value: ',' },
  { label: t('export.sep.tab'),       value: '\t' },
])

const progressPct = computed(() =>
  session.progress?.total ? Math.round(session.progress.current / session.progress.total * 100) : 0
)

const langLabels: Record<string, string> = {
  ja: '日本語', es: 'Español', en: 'English',
  fr: 'Français', de: 'Deutsch', it: 'Italiano'
}
function langLabel(l: string) { return langLabels[l] ?? l }
function toggleLang(lang: string, checked: boolean) {
  if (checked) { if (!selectedLangs.value.includes(lang)) selectedLangs.value.push(lang) }
  else { selectedLangs.value = selectedLangs.value.filter(l => l !== lang) }
}

const FILE_EXTENSIONS: Record<FileFormat, string[]> = {
  csv: ['csv'], json: ['json'], yaml: ['yaml', 'yml'], toml: ['toml'], xml: ['xml']
}

async function doExport() {
  const ext = FILE_EXTENSIONS[fileFormat.value]
  if (format.value === 'single') {
    const path = await save({ filters: [{ name: fileFormat.value.toUpperCase(), extensions: ext }] })
    if (!path) return
    await session.exportFormatted(path, selectedLangs.value, format.value, fileFormat.value, separator.value)
  } else {
    const dir = await open({ directory: true })
    if (!dir || Array.isArray(dir)) return
    await session.exportFormatted(dir as string, selectedLangs.value, format.value, fileFormat.value, separator.value)
  }
  playExported()
}

async function doReimport() {
  const allExts = ['csv', 'json', 'yaml', 'yml', 'toml', 'xml']
  const filePath = await open({ filters: [{ name: 'Export files', extensions: allExts }] })
  if (!filePath || Array.isArray(filePath)) return
  await session.importFormatted(filePath, separator.value)
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
.field { display: flex; flex-direction: column; gap: 6px; }
.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--text-3);
}
.drawer-footer {
  padding: 12px;
  border-top: 1px solid var(--border);
}
</style>
