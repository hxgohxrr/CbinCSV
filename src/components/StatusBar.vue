<template>
  <footer class="statusbar">
    <span>{{ session.files.length }} {{ t('status.files') }}</span>
    <span>{{ session.totalEntries }} {{ t('status.entries') }}</span>
    <span v-if="session.untranslatedCount > 0">{{ session.untranslatedCount }} {{ t('status.untranslated') }}</span>
    <span class="spacer" />
    <span>{{ session.parseMode === 'standard' ? 'Standard' : 'NNK' }}</span>
    <span>v1.0.0</span>
  </footer>
</template>

<script setup lang="ts">
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

const session = useSessionStore()
const { t } = useI18n()
</script>

<style scoped>
.statusbar {
  height: 26px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 16px;
  font-size: 11px;
  color: var(--text-3);
  flex-shrink: 0;
}
.spacer { flex: 1; }
</style>
