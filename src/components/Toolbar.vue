<template>
  <div class="toolbar">
    <span class="label">{{ t('toolbar.langs') }}</span>
    <div class="lang-chips">
      <NTag
        v-for="lang in session.languages"
        :key="lang"
        :type="session.visibleLangs.includes(lang) ? 'primary' : 'default'"
        :bordered="!session.visibleLangs.includes(lang)"
        size="small"
        style="cursor: pointer; font-weight: 600; letter-spacing: 0.04em"
        @click="toggleLang(lang)"
      >{{ lang.toUpperCase() }}</NTag>
    </div>
    <div class="sep" />
    <NButton quaternary circle size="small" :title="t('sidebar.import')" @click="addFiles">
      <template #icon>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <line x1="7" y1="2" x2="7" y2="12"/><line x1="2" y1="7" x2="12" y2="7"/>
        </svg>
      </template>
    </NButton>
    <NButton quaternary circle size="small" :title="t('toolbar.close')" @click="session.clear()">
      <template #icon>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <line x1="3" y1="3" x2="11" y2="11"/><line x1="11" y1="3" x2="3" y2="11"/>
        </svg>
      </template>
    </NButton>
    <NButton
      :type="session.diffMode ? 'primary' : 'default'"
      quaternary circle size="small"
      :title="t('toolbar.diff')"
      @click="session.diffMode = !session.diffMode"
    >
      <template #icon>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="1" y="2" width="5" height="10" rx="1"/>
          <rect x="8" y="2" width="5" height="10" rx="1"/>
          <line x1="3.5" y1="5" x2="3.5" y2="9" stroke-width="1"/>
          <line x1="10.5" y1="5" x2="10.5" y2="9" stroke-dasharray="1.5 1" stroke-width="1"/>
        </svg>
      </template>
    </NButton>
    <NButton
      :type="showReplace ? 'primary' : 'default'"
      quaternary circle size="small"
      :title="t('toolbar.replace')"
      @click="$emit('toggle-replace')"
    >
      <template #icon>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M2 5h7M6 2l3 3-3 3"/><path d="M12 9H5M8 6l-3 3 3 3"/>
        </svg>
      </template>
    </NButton>
    <NInput
      v-model:value="session.searchQuery"
      :placeholder="t('toolbar.search')"
      clearable
      size="small"
      style="width: 180px"
      @keydown.escape="session.searchQuery = ''"
    >
      <template #prefix>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" style="width:12px;height:12px;color:var(--text-3)">
          <circle cx="6" cy="6" r="4"/><line x1="9.5" y1="9.5" x2="12.5" y2="12.5"/>
        </svg>
      </template>
    </NInput>
  </div>
</template>

<script setup lang="ts">
import { NTag, NButton, NInput } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

defineProps<{ showReplace: boolean }>()
defineEmits<{ (e: 'toggle-replace'): void }>()

const session = useSessionStore()
const { t } = useI18n()

async function addFiles() {
  const selected = await open({ multiple: true, filters: [{ name: 'cfg.bin', extensions: ['bin'] }] })
  if (!selected) return
  await session.loadFiles(Array.isArray(selected) ? selected : [selected])
}

function toggleLang(lang: string) {
  const idx = session.visibleLangs.indexOf(lang)
  if (idx >= 0) { if (session.visibleLangs.length > 1) session.visibleLangs.splice(idx, 1) }
  else { session.visibleLangs.push(lang) }
}
</script>

<style scoped>
.toolbar {
  height: 44px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 8px;
  flex-shrink: 0;
}
.label { font-size: 11px; color: var(--text-2); font-weight: 500; white-space: nowrap; }
.lang-chips { display: flex; gap: 4px; }
.sep { flex: 1; }
</style>
