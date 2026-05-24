<template>
  <aside class="drawer">
    <div class="drawer-header">
      {{ t('mcp.title') }}
      <span class="badge-experimental">{{ t('mcp.experimental') }}</span>
    </div>
    <div class="drawer-body">

      <div class="field row">
        <span class="field-label">{{ t('mcp.port') }}</span>
        <NInputNumber
          v-model:value="settings.mcpPort"
          :min="1024"
          :max="65535"
          :disabled="running"
          size="small"
          style="width: 90px"
        />
      </div>

      <div class="field row">
        <span class="field-label">{{ running ? t('mcp.status.running', { port: settings.mcpPort }) : t('mcp.status.stopped') }}</span>
        <NSwitch v-model:value="running" @update:value="onToggle" />
      </div>

      <div v-if="running" class="config-block">
        <pre class="config-pre">{{ configSnippet }}</pre>
        <NButton size="tiny" @click="copyConfig">
          <template #icon>
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="4" y="4" width="8" height="8" rx="1"/>
              <path d="M2 10V2h8"/>
            </svg>
          </template>
          {{ t('mcp.copyConfig') }}
        </NButton>
      </div>

      <NAlert v-if="session.error" type="error" :title="session.error" :bordered="false" size="small" />

      <div class="log-header">
        <span class="field-label">{{ t('mcp.log.title') }}</span>
        <NButton size="tiny" text @click="log.splice(0)">{{ t('mcp.log.clear') }}</NButton>
      </div>
      <div class="log-wrap">
        <div v-if="!log.length" class="log-empty">{{ t('mcp.log.empty') }}</div>
        <div v-for="(entry, i) in log" :key="i" class="log-entry">
          <span class="log-ts">{{ entry.ts }}</span>
          <span class="log-tool">{{ entry.tool }}</span>
          <span class="log-detail">{{ entry.detail }}</span>
        </div>
      </div>

    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NSwitch, NInputNumber, NButton, NAlert } from 'naive-ui'
import { listen } from '@tauri-apps/api/event'
import { useSessionStore } from '../stores/session'
import { useSettingsStore } from '../stores/settings'
import { useI18n } from '../i18n'

const session = useSessionStore()
const settings = useSettingsStore()
const { t } = useI18n()

const running = ref(false)

interface LogEntry { ts: string; tool: string; detail: string }
const log = ref<LogEntry[]>([])

const configSnippet = computed(() => JSON.stringify({
  mcpServers: {
    cbincsv: { url: `http://localhost:${settings.mcpPort}/mcp` }
  }
}, null, 2))

async function onToggle(val: boolean) {
  if (val) {
    await session.startMcp(settings.mcpPort)
    if (!session.error) {
      await session.syncSession()
      listenForChanges()
    } else {
      running.value = false
    }
  } else {
    await session.stopMcp()
  }
}

let unlistenFn: (() => void) | null = null
async function listenForChanges() {
  if (unlistenFn) unlistenFn()
  unlistenFn = await listen<{ ev_name: string; index: number; lang: string; value: string }>(
    'mcp_entry_changed',
    (e) => {
      const { ev_name, index, lang } = e.payload
      const ts = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
      if (log.value.length >= 50) log.value.pop()
      log.value.unshift({ ts, tool: 'set_entry', detail: `${ev_name}[${index}] ${lang}` })
    }
  )
}

async function copyConfig() {
  await navigator.clipboard.writeText(configSnippet.value)
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
  display: flex;
  align-items: center;
  gap: 6px;
}
.badge-experimental {
  font-size: 9px;
  padding: 1px 5px;
  border-radius: 4px;
  background: rgba(230,162,60,0.15);
  color: #e6a23c;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
.drawer-body {
  padding: 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}
.field { display: flex; flex-direction: column; gap: 6px; }
.field.row { flex-direction: row; align-items: center; justify-content: space-between; }
.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--text-3);
}
.config-block {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.config-pre {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: var(--text-2);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}
.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.log-wrap {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 4px;
  min-height: 80px;
  max-height: 200px;
  background: var(--bg-card);
}
.log-empty {
  padding: 8px;
  font-size: 10px;
  color: var(--text-3);
  font-style: italic;
  text-align: center;
}
.log-entry {
  display: flex;
  gap: 6px;
  padding: 3px 8px;
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  border-bottom: 1px solid var(--border);
}
.log-entry:last-child { border-bottom: none; }
.log-ts { color: var(--text-3); flex-shrink: 0; }
.log-tool { color: var(--accent); flex-shrink: 0; }
.log-detail { color: var(--text-2); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
