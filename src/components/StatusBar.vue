<template>
  <footer class="statusbar">
    <span>{{ session.files.length }} {{ t('status.files') }}</span>
    <span>{{ session.totalEntries }} {{ t('status.entries') }}</span>
    <span v-if="session.untranslatedCount > 0" class="untranslated">
      {{ session.untranslatedCount }} {{ t('status.untranslated') }}
    </span>
    <span v-if="session.diffMode && session.dirtyCount > 0" class="modified">
      {{ session.dirtyCount }} {{ t('status.modified') }}
    </span>
    <span class="spacer" />
    <span v-if="session.formatSummary.rdbn > 0" class="fmt-badge fmt-rdbn">{{ session.formatSummary.rdbn }} RDBN</span>
    <span v-if="session.formatSummary.nnk > 0"  class="fmt-badge fmt-nnk">{{ session.formatSummary.nnk }} NNK</span>
    <span v-if="session.formatSummary.t2b > 0"  class="fmt-badge fmt-t2b">{{ session.formatSummary.t2b }} T2B</span>
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
  gap: 12px;
  font-size: 11px;
  color: var(--text-3);
  flex-shrink: 0;
}
.spacer { flex: 1; }
.untranslated { color: #e6a23c; }
.modified { color: #5eaf70; }
.fmt-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  letter-spacing: 0.04em;
}
.fmt-t2b  { background: rgba(94,175,112,0.15);  color: #5eaf70; }
.fmt-rdbn { background: rgba(230,162,60,0.15);   color: #e6a23c; }
.fmt-nnk  { background: rgba(150,100,220,0.15);  color: #9b6fdc; }
</style>
