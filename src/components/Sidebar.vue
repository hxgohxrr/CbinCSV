<template>
  <aside class="sidebar">
    <div class="section">
      <span class="section-label">{{ t('sidebar.game') }}</span>
      <NSelect
        v-model:value="selectedProfile"
        :options="profileOptions"
        size="small"
        @update:value="applyProfile"
      />
      <div class="game-info">
        <dt>{{ t('game.info.format') }}</dt>
        <dd>{{ currentProfile.format }}</dd>
        <dt>{{ t('game.info.encoding') }}</dt>
        <dd>{{ currentProfile.encoding }}</dd>
        <dt v-if="currentProfile.notes">{{ t('game.info.notes') }}</dt>
        <dd v-if="currentProfile.notes">{{ currentProfile.notes }}</dd>
      </div>
    </div>

    <span class="section-label" style="padding: 10px 12px 4px; display:block;">{{ t('sidebar.files') }}</span>
    <div class="file-list">
      <div v-for="ev in session.evNames" :key="ev" class="file-group">
        <div class="file-group-name">{{ ev.split('_').slice(0, 2).join('_') }}</div>
        <div v-for="file in filesForEv(ev)" :key="file.path" class="file-item">
          <svg class="file-icon" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="1" width="10" height="12" rx="1.5"/>
            <line x1="4.5" y1="5" x2="9.5" y2="5"/>
            <line x1="4.5" y1="7.5" x2="9.5" y2="7.5"/>
          </svg>
          <span class="file-name">{{ file.ev_name }}</span>
          <NTag :color="fmtColor(file.mode)" size="tiny" :bordered="false">{{ fmtLabel(file.mode) }}</NTag>
          <NTag type="primary" size="tiny" :bordered="false">{{ file.language.toUpperCase() }}</NTag>
        </div>
      </div>
    </div>

    <div class="sidebar-footer">
      <NButton block @click="importFiles" style="justify-content: flex-start">
        <template #icon>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M7 2v7M4 6l3 3 3-3M2 10v2h10v-2"/>
          </svg>
        </template>
        {{ t('sidebar.import') }}
      </NButton>
      <NButton block quaternary @click="importFolder" style="justify-content: flex-start">
        <template #icon>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M1 4h12v8H1zM1 4l2-2h4l2 2"/>
          </svg>
        </template>
        {{ t('sidebar.openFolder') }}
      </NButton>
      <NDivider style="margin: 4px 0" />
      <NButton block quaternary @click="openSettings" style="justify-content: flex-start">
        <template #icon>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <line x1="2" y1="4" x2="12" y2="4"/>
            <line x1="2" y1="7" x2="12" y2="7"/>
            <line x1="2" y1="10" x2="12" y2="10"/>
            <circle cx="5" cy="4" r="1.2" fill="var(--bg-panel)"/>
            <circle cx="9" cy="7" r="1.2" fill="var(--bg-panel)"/>
            <circle cx="5" cy="10" r="1.2" fill="var(--bg-panel)"/>
          </svg>
        </template>
        {{ t('settings.title') }}
      </NButton>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch, inject } from 'vue'
import { NSelect, NTag, NButton, NDivider } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useSessionStore, type GameProfile } from '../stores/session'
import { useI18n } from '../i18n'

const showSettings = inject<{ value: boolean }>('showSettings')
function openSettings() { if (showSettings) showSettings.value = true }

const session = useSessionStore()
const { t } = useI18n()

interface Profile { id: GameProfile; name: string; format: string; encoding: string; notes?: string; parseMode: 'standard' | 'nnk' }

const profiles: Profile[] = [
  { id: 'level5', name: 'Level-5 Generic',  format: 'T2B',  encoding: 'UTF-8 / Shift-JIS', parseMode: 'standard' },
  { id: 'yokai',  name: 'Yo-kai Watch',     format: 'RDBN', encoding: 'Shift-JIS',         parseMode: 'standard' },
  { id: 'tt',     name: 'Time Travelers',   format: 'T2B',  encoding: 'Shift-JIS',         parseMode: 'standard', notes: '3DS / Vita' },
  { id: 'nnk',    name: 'Ni no Kuni',       format: 'NNK',  encoding: 'UTF-16 LE',         parseMode: 'nnk' },
]

const profileOptions = profiles.map(p => ({ label: p.name, value: p.id }))
const selectedProfile = ref<GameProfile>(session.gameProfile)
const currentProfile = computed(() => profiles.find(p => p.id === selectedProfile.value) ?? profiles[0])

watch(() => session.gameProfile, v => { selectedProfile.value = v })

function applyProfile(val: GameProfile) {
  session.gameProfile = val
  const p = profiles.find(x => x.id === val)
  if (p) session.parseMode = p.parseMode
}

function fmtLabel(mode?: string) {
  return mode === 'rdbn' ? 'RDBN' : mode === 'nnk' ? 'NNK' : 'T2B'
}
function fmtColor(mode?: string) {
  if (mode === 'rdbn') return { color: 'rgba(230,162,60,0.15)', textColor: '#e6a23c', borderColor: 'transparent' }
  if (mode === 'nnk')  return { color: 'rgba(150,100,220,0.15)', textColor: '#9b6fdc', borderColor: 'transparent' }
  return { color: 'rgba(94,175,112,0.15)', textColor: '#5eaf70', borderColor: 'transparent' }
}

function filesForEv(ev: string) { return session.files.filter(f => f.ev_name === ev) }

async function importFiles() {
  const selected = await open({ multiple: true, filters: [{ name: 'cfg.bin', extensions: ['bin'] }] })
  if (!selected) return
  await session.loadFiles(Array.isArray(selected) ? selected : [selected])
}
async function importFolder() {
  const selected = await open({ directory: true })
  if (!selected || Array.isArray(selected)) return
  await session.loadFolder(selected)
}
</script>

<style scoped>
.sidebar {
  width: 240px;
  background: var(--bg-panel);
  border-right: 1px solid var(--border);
  display: flex; flex-direction: column;
  flex-shrink: 0; overflow: hidden; min-height: 0;
}
.section { padding: 12px; border-bottom: 1px solid var(--border); }
.section-label {
  display: block; font-size: 10px; font-weight: 600;
  letter-spacing: 0.1em; text-transform: uppercase;
  color: var(--text-3); margin-bottom: 8px;
}
.game-info {
  background: var(--accent-dim); border: 1px solid var(--accent-soft);
  border-radius: var(--radius); padding: 8px 10px;
  font-size: 11px; color: var(--text-2); margin-top: 8px;
}
.game-info dt { color: var(--accent); font-weight: 600; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; margin-top: 4px; }
.game-info dt:first-child { margin-top: 0; }
.game-info dd { margin-bottom: 2px; }
.file-list { flex: 1; overflow-y: auto; padding: 8px; }
.file-group { margin-bottom: 8px; }
.file-group-name { font-size: 10px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-3); padding: 2px 6px 4px; }
.file-item { display: flex; align-items: center; gap: 5px; padding: 5px 8px; border-radius: var(--radius); cursor: default; }
.file-item:hover { background: var(--bg-hover); }
.file-icon { width: 14px; height: 14px; flex-shrink: 0; color: var(--text-3); }
.file-name { font-size: 11px; font-family: 'JetBrains Mono', monospace; color: var(--text-2); flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.sidebar-footer { padding: 10px 12px; border-top: 1px solid var(--border); display: flex; flex-direction: column; gap: 4px; }
</style>
