<template>
  <div class="pb-wrap">
    <div class="pb-track">
      <div class="pb-fill" :style="{ width: pct + '%' }" />
    </div>
    <span class="pb-label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../i18n'

const props = defineProps<{ current: number; total: number }>()
const { t } = useI18n()
const pct = computed(() => props.total ? Math.round((props.current / props.total) * 100) : 0)
const label = computed(() => t('export.progress', { current: props.current, total: props.total }))
</script>

<style scoped>
.pb-wrap { display: flex; flex-direction: column; gap: 4px; }
.pb-track {
  background: var(--bg-input);
  border: 1px solid var(--border-md);
  border-radius: 3px;
  overflow: hidden;
  height: 4px;
}
.pb-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.pb-label { font-size: 10px; color: var(--text-3); text-align: center; }
</style>
