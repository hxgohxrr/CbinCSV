<template>
  <div class="overlay">
    <div class="card">
      <h2 class="title">{{ t('import.title') }}</h2>
      <p class="sub">{{ t('import.sub') }}</p>

      <div class="dropzone" :class="{ dragging: isDragging }" @click="importFiles">
        <svg class="drop-icon" viewBox="0 0 40 40" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="6" y="4" width="28" height="32" rx="3"/>
          <line x1="13" y1="14" x2="27" y2="14"/>
          <line x1="13" y1="20" x2="27" y2="20"/>
          <line x1="13" y1="26" x2="20" y2="26"/>
        </svg>
        <span class="drop-text">{{ t('import.drop') }}</span>
        <span class="drop-sub">{{ t('import.dropSub') }}</span>
      </div>

      <div class="divider"><span>{{ t('import.continue') }}</span></div>

      <div class="btns">
        <button class="btn btn-secondary" style="flex:1" @click="importFiles">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="1" width="10" height="12" rx="1.5"/>
            <line x1="4.5" y1="5" x2="9.5" y2="5"/>
          </svg>
          {{ t('import.files') }}
        </button>
        <button class="btn btn-secondary" style="flex:1" @click="importFolder">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M1 4h12v8H1zM1 4l2-2h4l2 2"/>
          </svg>
          {{ t('import.folder') }}
        </button>
      </div>

      <div v-if="session.progress" class="progress-wrap">
        <ProgressBar :current="session.progress.current" :total="session.progress.total" />
      </div>
      <p v-if="session.error" class="error-msg">{{ session.error }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'
import ProgressBar from './ProgressBar.vue'

defineProps<{ isDragging: boolean }>()

const session = useSessionStore()
const { t } = useI18n()

async function importFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'cfg.bin', extensions: ['bin'] }]
  })
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
.overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-base);
}
.card {
  background: var(--bg-panel);
  border: 1px solid var(--border-md);
  border-radius: var(--radius-lg);
  padding: 40px;
  width: 500px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
}
.title { font-size: 18px; font-weight: 600; }
.sub {
  font-size: 12px;
  color: var(--text-2);
  text-align: center;
  line-height: 1.7;
  white-space: pre-line;
}
.dropzone {
  width: 100%;
  border: 2px dashed var(--border-md);
  border-radius: var(--radius-lg);
  padding: 28px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.2s;
}
.dropzone:hover, .dropzone.dragging {
  border-color: var(--accent-soft);
  background: var(--accent-dim);
}
.drop-icon { width: 36px; height: 36px; color: var(--text-3); }
.drop-text { font-size: 13px; font-weight: 500; color: var(--text-2); }
.drop-sub { font-size: 11px; color: var(--text-3); }
.divider {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}
.divider::before, .divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border-md);
}
.divider span {
  font-size: 10px;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.btns { display: flex; gap: 8px; width: 100%; }
.progress-wrap { width: 100%; }
.error-msg { font-size: 11px; color: var(--danger); text-align: center; }
</style>
