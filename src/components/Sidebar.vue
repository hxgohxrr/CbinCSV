<template>
  <aside class="sidebar">
    <div class="section">
      <span class="section-label">{{ t('sidebar.game') }}</span>
      <select class="form-select" v-model="gameType" style="margin-bottom:6px;">
        <option value="standard">{{ t('game.standard') }}</option>
        <option value="nnk">{{ t('game.nnk') }}</option>
      </select>
      <div class="game-info">
        <dt>{{ t('game.info.mode') }}</dt>
        <dd>{{ t(`game.${gameType}.info.mode`) }}</dd>
        <dt>{{ t('game.info.encoding') }}</dt>
        <dd>{{ t(`game.${gameType}.info.encoding`) }}</dd>
      </div>
    </div>

    <div class="section">
      <span class="section-label">{{ t('sidebar.mode') }}</span>
      <div class="mode-tabs">
        <button :class="['mode-tab', session.parseMode === 'standard' && 'active']"
          @click="session.parseMode = 'standard'">{{ t('mode.standard') }}</button>
        <button :class="['mode-tab', session.parseMode === 'nnk' && 'active']"
          @click="session.parseMode = 'nnk'">{{ t('mode.nnk') }}</button>
      </div>
    </div>

    <span class="section-label" style="padding: 10px 12px 4px; display:block;">{{ t('sidebar.files') }}</span>
    <div class="file-list">
      <div v-for="ev in session.evNames" :key="ev" class="file-group">
        <div class="file-group-name">{{ ev.split('_').slice(0, 2).join('_') }}</div>
        <div
          v-for="file in filesForEv(ev)"
          :key="file.path"
          class="file-item"
        >
          <svg class="file-icon" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="1" width="10" height="12" rx="1.5"/>
            <line x1="4.5" y1="5" x2="9.5" y2="5"/>
            <line x1="4.5" y1="7.5" x2="9.5" y2="7.5"/>
          </svg>
          <span class="file-name">{{ file.ev_name }}</span>
          <span class="file-lang-tag">{{ file.language.toUpperCase() }}</span>
        </div>
      </div>
    </div>

    <div class="sidebar-footer">
      <button class="btn btn-secondary btn-full" @click="importFiles">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M7 2v7M4 6l3 3 3-3M2 10v2h10v-2"/>
        </svg>
        {{ t('sidebar.import') }}
      </button>
      <button class="btn btn-ghost btn-full" @click="importFolder">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M1 4h12v8H1zM1 4l2-2h4l2 2"/>
        </svg>
        {{ t('sidebar.openFolder') }}
      </button>
      <button class="btn btn-ghost btn-full settings-btn" @click="openSettings">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <line x1="2" y1="4" x2="12" y2="4"/>
          <line x1="2" y1="7" x2="12" y2="7"/>
          <line x1="2" y1="10" x2="12" y2="10"/>
          <circle cx="5" cy="4" r="1.2" fill="var(--bg-panel)"/>
          <circle cx="9" cy="7" r="1.2" fill="var(--bg-panel)"/>
          <circle cx="5" cy="10" r="1.2" fill="var(--bg-panel)"/>
        </svg>
        {{ t('settings.title') }}
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

const showSettings = inject<{ value: boolean }>('showSettings')
function openSettings() { if (showSettings) showSettings.value = true }

const session = useSessionStore()
const { t } = useI18n()
const gameType = ref<'standard' | 'nnk'>('standard')

function filesForEv(ev: string) {
  return session.files.filter(f => f.ev_name === ev)
}

async function importFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'cfg.bin', extensions: ['bin'] }]
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  await session.loadFiles(paths)
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
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
  min-height: 0;
}
.section { padding: 12px; border-bottom: 1px solid var(--border); }
.section-label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-3);
  margin-bottom: 8px;
}
.game-info {
  background: var(--accent-dim);
  border: 1px solid var(--accent-soft);
  border-radius: var(--radius);
  padding: 8px 10px;
  font-size: 11px;
  color: var(--text-2);
}
.game-info dt {
  color: var(--accent);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-top: 4px;
}
.game-info dt:first-child { margin-top: 0; }
.game-info dd { margin-bottom: 2px; }
.mode-tabs {
  display: flex;
  background: var(--bg-input);
  border: 1px solid var(--border-md);
  border-radius: var(--radius);
  padding: 2px;
  gap: 2px;
}
.mode-tab {
  flex: 1;
  padding: 4px 0;
  text-align: center;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-2);
  cursor: pointer;
  border-radius: 4px;
  border: none;
  background: transparent;
  font-family: 'Outfit', sans-serif;
  transition: all 0.15s;
}
.mode-tab.active { background: var(--bg-card); color: var(--accent); }
.file-list { flex: 1; overflow-y: auto; padding: 8px; }
.file-group { margin-bottom: 8px; }
.file-group-name {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-3);
  padding: 2px 6px 4px;
}
.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: var(--radius);
  cursor: default;
}
.file-item:hover { background: var(--bg-hover); }
.file-icon { width: 14px; height: 14px; flex-shrink: 0; color: var(--text-3); }
.file-name {
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-2);
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-lang-tag {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--accent-dim);
  color: var(--accent);
  letter-spacing: 0.04em;
}
.sidebar-footer {
  padding: 10px 12px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.settings-btn {
  border-top: 1px solid var(--border);
  padding-top: 10px;
  margin-top: 2px;
}
</style>
