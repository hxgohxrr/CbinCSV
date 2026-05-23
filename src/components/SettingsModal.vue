<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <span>{{ t('settings.title') }}</span>
        <button class="btn-icon" @click="emit('close')">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <line x1="2" y1="2" x2="12" y2="12"/>
            <line x1="12" y1="2" x2="2" y2="12"/>
          </svg>
        </button>
      </div>
      <div class="modal-body">

        <div class="field">
          <label class="form-label">{{ t('settings.theme') }}</label>
          <div class="theme-tabs">
            <button v-for="opt in ['dark', 'light', 'system']" :key="opt"
              :class="['theme-tab', settings.theme === opt && 'active']"
              @click="settings.theme = opt as 'dark' | 'light' | 'system'">
              {{ t(`settings.theme.${opt}`) }}
            </button>
          </div>
        </div>

        <div class="field">
          <label class="form-label">{{ t('settings.accent') }}</label>
          <div class="swatches">
            <button
              v-for="color in ACCENT_PRESETS"
              :key="color"
              class="swatch"
              :class="{ active: settings.accent === color }"
              :style="{ background: color }"
              @click="settings.accent = color"
            />
            <input type="color" class="color-picker" v-model="settings.accent" />
          </div>
        </div>

        <div class="field">
          <label class="form-label">{{ t('settings.locale') }}</label>
          <select class="form-select" v-model="settings.locale">
            <option v-for="l in SUPPORTED_LOCALES" :key="l" :value="l">{{ langName(l) }}</option>
          </select>
        </div>

        <div class="field">
          <label class="form-label">{{ t('settings.splash') }}</label>
          <button
            class="toggle"
            :class="{ active: settings.showSplash }"
            @click="settings.showSplash = !settings.showSplash"
          >
            <span class="toggle-thumb" />
          </button>
        </div>

        <div class="field">
          <label class="form-label">
            {{ t('settings.sound') }}
            <span class="field-hint">{{ settings.soundVolume === 0 ? t('settings.soundMuted') : settings.soundVolume + '%' }}</span>
          </label>
          <input
            type="range"
            min="0"
            max="100"
            step="5"
            class="vol-slider"
            v-model.number="settings.soundVolume"
          />
        </div>

      </div>
      <div class="modal-footer">
        <button class="btn btn-primary btn-full" @click="emit('close')">{{ t('settings.save') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useSettingsStore, ACCENT_PRESETS, SUPPORTED_LOCALES } from '../stores/settings'
import { useI18n } from '../i18n'

const emit = defineEmits<{ close: [] }>()
const settings = useSettingsStore()
const { t } = useI18n()

const langNames: Record<string, string> = {
  es: 'Español',
  en: 'English',
  fr: 'Français',
  de: 'Deutsch',
  ja: '日本語',
  it: 'Italiano'
}
function langName(l: string) { return langNames[l] ?? l }
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}
.modal {
  background: var(--bg-panel);
  border: 1px solid var(--border-md);
  border-radius: var(--radius-lg);
  width: 380px;
  display: flex;
  flex-direction: column;
}
.modal-header {
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-weight: 600;
  font-size: 14px;
}
.modal-body {
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.field { display: flex; flex-direction: column; gap: 6px; }
.theme-tabs { display: flex; gap: 4px; }
.theme-tab {
  flex: 1;
  padding: 6px;
  text-align: center;
  font-size: 12px;
  font-weight: 500;
  border-radius: var(--radius);
  border: 1px solid var(--border-md);
  background: transparent;
  color: var(--text-2);
  cursor: pointer;
  font-family: 'Outfit', sans-serif;
  transition: all 0.15s;
}
.theme-tab.active { background: var(--accent-dim); border-color: var(--accent-soft); color: var(--accent); }
.swatches { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.swatch {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}
.swatch.active { border-color: var(--text-1); transform: scale(1.15); }
.color-picker {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid var(--border-md);
  cursor: pointer;
  padding: 0;
  background: transparent;
}
.modal-footer { padding: 14px 16px; border-top: 1px solid var(--border); }

.field-hint {
  font-size: 10px;
  color: var(--text-3);
  font-weight: 400;
  margin-left: 8px;
}

.toggle {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: var(--border-md);
  border: none;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  transition: background 0.2s;
  position: relative;
}
.toggle.active { background: var(--accent); }
.toggle-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  display: block;
  transition: transform 0.2s;
  pointer-events: none;
}
.toggle.active .toggle-thumb { transform: translateX(18px); }

.vol-slider {
  width: 100%;
  height: 4px;
  accent-color: var(--accent);
  cursor: pointer;
}
</style>
