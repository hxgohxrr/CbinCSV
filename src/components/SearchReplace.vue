<template>
  <div class="sr-bar">
    <NInputGroup>
      <NInput
        ref="findRef"
        v-model:value="findText"
        :placeholder="t('replace.find')"
        clearable
        size="small"
        style="width: 220px"
        @keydown.enter="doReplaceAll"
        @keydown.escape="$emit('close')"
      >
        <template #prefix>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" style="width:11px;height:11px">
            <circle cx="6" cy="6" r="4"/><line x1="9.5" y1="9.5" x2="12.5" y2="12.5"/>
          </svg>
        </template>
        <template #suffix>
          <NText v-if="findText" depth="3" style="font-size: 10px; font-family: monospace; white-space: nowrap">
            {{ matchCount }}
          </NText>
        </template>
      </NInput>
      <NInput
        v-model:value="replaceText"
        :placeholder="t('replace.replace')"
        size="small"
        style="width: 220px"
        @keydown.escape="$emit('close')"
      >
        <template #prefix>
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" style="width:11px;height:11px">
            <path d="M2 5h7M6 2l3 3-3 3"/><path d="M12 9H5M8 6l-3 3 3 3"/>
          </svg>
        </template>
      </NInput>
    </NInputGroup>

    <NSelect
      v-model:value="selectedLang"
      :options="langOptions"
      size="small"
      style="width: 130px"
    />

    <NButton
      :type="caseSensitive ? 'primary' : 'default'"
      size="small"
      style="font-family: 'JetBrains Mono', monospace; font-weight: 700; width: 36px; padding: 0"
      :title="t('replace.caseSensitive')"
      @click="caseSensitive = !caseSensitive"
    >Aa</NButton>

    <NButton
      type="primary"
      size="small"
      :disabled="!findText"
      @click="doReplaceAll"
    >{{ t('replace.replaceAll') }}</NButton>

    <NText v-if="lastResult !== null" style="font-size: 11px; color: #5eaf70; white-space: nowrap; font-weight: 500">
      {{ lastResult }} {{ t('replace.replaced') }}
    </NText>

    <div style="flex:1" />

    <NButton quaternary circle size="small" @click="$emit('close')">
      <template #icon>
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <line x1="3" y1="3" x2="11" y2="11"/><line x1="11" y1="3" x2="3" y2="11"/>
        </svg>
      </template>
    </NButton>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { NInput, NInputGroup, NSelect, NButton, NText } from 'naive-ui'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

defineEmits<{ (e: 'close'): void }>()

const session = useSessionStore()
const { t } = useI18n()

const findRef = ref<InstanceType<typeof NInput> | null>(null)
const findText    = ref('')
const replaceText = ref('')
const selectedLang = ref('all')
const caseSensitive = ref(false)
const lastResult = ref<number | null>(null)

const langOptions = computed(() => [
  { label: t('replace.allLangs'), value: 'all' },
  ...session.languages.map(l => ({ label: l.toUpperCase(), value: l }))
])

const matchCount = computed(() =>
  session.countMatches(findText.value, selectedLang.value, caseSensitive.value)
)

watch([findText, selectedLang, caseSensitive], () => { lastResult.value = null })

function doReplaceAll() {
  if (!findText.value) return
  lastResult.value = session.replaceInEntries(findText.value, replaceText.value, selectedLang.value, caseSensitive.value)
}

onMounted(() => (findRef.value as any)?.focus())
</script>

<style scoped>
.sr-bar {
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  padding: 6px 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: wrap;
}
</style>
