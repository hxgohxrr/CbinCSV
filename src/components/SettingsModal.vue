<template>
  <NModal
    :show="true"
    preset="card"
    :title="t('settings.title')"
    style="width: 400px; max-width: 95vw"
    :bordered="false"
    @update:show="(v) => !v && emit('close')"
  >
    <NSpace vertical :size="20">

      <NFormItem :label="t('settings.theme')" label-placement="top">
        <NRadioGroup v-model:value="settings.theme">
          <NRadioButton v-for="opt in ['dark', 'light', 'system']" :key="opt" :value="opt">
            {{ t(`settings.theme.${opt}`) }}
          </NRadioButton>
        </NRadioGroup>
      </NFormItem>

      <NFormItem :label="t('settings.accent')" label-placement="top">
        <NSpace :size="8" align="center">
          <div
            v-for="color in ACCENT_PRESETS"
            :key="color"
            class="swatch"
            :class="{ active: settings.accent === color }"
            :style="{ background: color }"
            @click="settings.accent = color"
          />
          <input
            type="color"
            class="swatch swatch-custom"
            :class="{ active: !ACCENT_PRESETS.includes(settings.accent) }"
            :value="settings.accent"
            :title="settings.accent"
            @input="settings.accent = ($event.target as HTMLInputElement).value"
          />
        </NSpace>
      </NFormItem>

      <NFormItem :label="t('settings.locale')" label-placement="top">
        <NSelect
          v-model:value="settings.locale"
          :options="localeOptions"
          style="width: 100%"
        />
      </NFormItem>

      <NFormItem :label="t('settings.sourceLang')" label-placement="top">
        <NSelect
          v-model:value="settings.sourceLang"
          :options="sourceLangOptions"
          style="width: 100%"
        />
        <NText depth="3" style="font-size: 11px; margin-top: 4px; display: block">
          {{ t('settings.sourceLang.hint') }}
        </NText>
      </NFormItem>

      <NFormItem label-placement="top">
        <template #label>
          {{ t('settings.splash') }}
        </template>
        <NSwitch v-model:value="settings.showSplash" />
      </NFormItem>

      <NFormItem label-placement="top">
        <template #label>
          {{ t('settings.sound') }}
          <NText depth="3" style="font-size: 11px; margin-left: 8px">
            {{ settings.soundVolume === 0 ? t('settings.soundMuted') : settings.soundVolume + '%' }}
          </NText>
        </template>
        <NSlider
          v-model:value="settings.soundVolume"
          :min="0" :max="100" :step="5"
          style="width: 100%"
        />
      </NFormItem>

    </NSpace>

    <template #footer>
      <NButton type="primary" block @click="emit('close')">
        {{ t('settings.save') }}
      </NButton>
    </template>
  </NModal>
</template>

<script setup lang="ts">
import {
  NModal, NFormItem, NSpace, NRadioGroup, NRadioButton,
  NSelect, NSwitch, NSlider, NButton, NText
} from 'naive-ui'
import { useSettingsStore, ACCENT_PRESETS, SUPPORTED_LOCALES } from '../stores/settings'
import { useI18n } from '../i18n'

const emit = defineEmits<{ close: [] }>()
const settings = useSettingsStore()
const { t } = useI18n()

const langNames: Record<string, string> = {
  es: 'Español', en: 'English', fr: 'Français',
  de: 'Deutsch', ja: '日本語', it: 'Italiano'
}

const localeOptions = SUPPORTED_LOCALES.map(l => ({ label: langNames[l] ?? l, value: l }))
const sourceLangOptions = [
  { label: t('settings.sourceLang.none'), value: '' },
  ...SUPPORTED_LOCALES.map(l => ({ label: langNames[l] ?? l, value: l }))
]
</script>

<style scoped>
.swatch {
  width: 24px; height: 24px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.15s;
  flex-shrink: 0;
}
.swatch.active { border-color: var(--text-1); transform: scale(1.15); }
.swatch-custom {
  padding: 0;
  cursor: pointer;
  background: none;
}
.swatch-custom::-webkit-color-swatch-wrapper { padding: 0; border-radius: 50%; }
.swatch-custom::-webkit-color-swatch { border: none; border-radius: 50%; }
</style>
