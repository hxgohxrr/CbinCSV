<template>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th class="col-ev">EV_NAME</th>
          <th class="col-idx">#</th>
          <th class="col-type">TYPE</th>
          <th v-for="lang in visibleLangs" :key="lang" class="col-lang">
            <span :class="`lang-badge lang-${lang}`">{{ lang.toUpperCase() }}</span>
            {{ langLabel(lang) }}
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="ev in filteredEvNames" :key="ev">
          <tr v-for="idx in entryIndicesForEv(ev)" :key="`${ev}-${idx}`">
            <td class="td-ev">
              <strong>{{ ev }}</strong>
            </td>
            <td class="td-idx">{{ idx }}</td>
            <td class="td-type">
              <span :class="['type-badge', `type-${getType(ev, idx)}`]">
                {{ getType(ev, idx) }}
              </span>
            </td>
            <td v-for="lang in visibleLangs" :key="lang" class="td-text">
              <div :class="['cell-wrap', session.diffMode && diffClass(ev, lang, idx)]">
                <component
                  :is="inputComponent(getType(ev, idx))"
                  v-bind="inputProps(getType(ev, idx), getVal(ev, lang, idx))"
                  :class="cellClass(getType(ev, idx), getVal(ev, lang, idx), `${ev}-${lang}-${idx}`)"
                  @input="onInput(ev, lang, idx, $event)"
                  @change="onInput(ev, lang, idx, $event)"
                  @focus="onCellFocus(ev, lang, idx, $event)"
                  @blur="maybeResize"
                />
                <div
                  v-if="activeTmKey === `${ev}-${lang}-${idx}` && tmSuggestions.length"
                  class="tm-chips"
                >
                  <button
                    v-for="s in tmSuggestions"
                    :key="s"
                    class="tm-chip"
                    @mousedown.prevent="fillFromTm(ev, lang, idx, s)"
                  >{{ s.length > 50 ? s.slice(0, 50) + '…' : s }}</button>
                </div>
              </div>
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSessionStore } from '../stores/session'
import { useSettingsStore } from '../stores/settings'
import { useI18n } from '../i18n'

const session = useSessionStore()
const settings = useSettingsStore()
const { t } = useI18n()

const activeTmKey = ref<string | null>(null)
const tmSuggestions = ref<string[]>([])
const invalidCells = ref<Set<string>>(new Set())

const visibleLangs = computed(() => session.visibleLangs)

const langLabels: Record<string, string> = {
  ja: '日本語', es: 'Español', en: 'English',
  fr: 'Français', de: 'Deutsch', it: 'Italiano'
}
function langLabel(l: string) { return langLabels[l] ?? l }

const filteredEvNames = computed(() => {
  const q = session.searchQuery.toLowerCase()
  if (!q) return session.evNames
  return session.evNames.filter(ev =>
    session.files.filter(f => f.ev_name === ev)
      .some(f => f.entries.some(e => e.value.toLowerCase().includes(q)))
  )
})

function entryIndicesForEv(ev: string): number[] {
  const evFiles = session.files.filter(f => f.ev_name === ev)
  const max = Math.max(...evFiles.map(f => f.entries.length), 0)
  return Array.from({ length: max }, (_, i) => i)
}


function getEntry(ev: string, idx: number) {
  const file = session.files.find(f => f.ev_name === ev)
  return file?.entries[idx]
}

function getType(ev: string, idx: number): string {
  return getEntry(ev, idx)?.field_type ?? 'string'
}

function getVal(ev: string, lang: string, idx: number): string {
  const file = session.files.find(f => f.ev_name === ev && f.language === lang)
  return file?.entries[idx]?.value ?? ''
}

function inputComponent(type: string): string {
  return type === 'string' ? 'textarea' : 'input'
}

function inputProps(type: string, value: string): Record<string, unknown> {
  const base = { value }
  if (type === 'int') return { ...base, type: 'number', step: 1 }
  if (type === 'float') return { ...base, type: 'number', step: 'any' }
  return { ...base, rows: 1, placeholder: t('table.untranslated') }
}

function isValidForType(type: string, value: string): boolean {
  if (value === '') return true
  if (type === 'int') return /^-?\d+$/.test(value.trim())
  if (type === 'float') return !isNaN(Number(value)) && /^-?[\d.eE+\-]+$/.test(value.trim())
  return true
}

function cellClass(type: string, value: string, cellKey: string): string[] {
  const classes = ['cell-input', `cell-${type}`]
  if (type === 'string' && !value) classes.push('empty')
  if (invalidCells.value.has(cellKey)) classes.push('cell-invalid')
  return classes
}

function onInput(ev: string, lang: string, idx: number, event: Event) {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement
  const value = target.value
  const type = getType(ev, idx)
  const key = `${ev}-${lang}-${idx}`
  if (!isValidForType(type, value)) {
    invalidCells.value = new Set(invalidCells.value).add(key)
    return
  }
  if (invalidCells.value.has(key)) {
    const s = new Set(invalidCells.value)
    s.delete(key)
    invalidCells.value = s
  }
  session.updateEntry(ev, lang, idx, value)
  if (target.tagName === 'TEXTAREA') maybeResize(event)
}

function onCellFocus(ev: string, lang: string, idx: number, event: Event) {
  maybeResize(event)
  if (getType(ev, idx) !== 'string' || !settings.sourceLang || settings.sourceLang === lang) {
    activeTmKey.value = null
    tmSuggestions.value = []
    return
  }
  const sourceText = getVal(ev, settings.sourceLang, idx)
  const suggestions = session.getTranslationSuggestions(sourceText, lang, settings.sourceLang)
  activeTmKey.value = suggestions.length ? `${ev}-${lang}-${idx}` : null
  tmSuggestions.value = suggestions
}

function diffClass(ev: string, lang: string, idx: number): string {
  const d = session.getEntryDiff(ev, lang, idx)
  return d ? `diff-${d}` : ''
}

function fillFromTm(ev: string, lang: string, idx: number, value: string) {
  session.updateEntry(ev, lang, idx, value)
  activeTmKey.value = null
  tmSuggestions.value = []
}

function maybeResize(event: Event) {
  const ta = event.target as HTMLElement
  if (ta.tagName !== 'TEXTAREA') return
  ta.style.height = 'auto'
  ta.style.height = (ta as HTMLTextAreaElement).scrollHeight + 'px'
}
</script>

<style scoped>
.table-wrap { flex: 1; overflow: auto; min-height: 0; }
table { width: 100%; border-collapse: collapse; font-size: 12px; }
thead { position: sticky; top: 0; z-index: 10; }
thead th {
  background: var(--bg-panel);
  border-bottom: 2px solid var(--border-md);
  padding: 8px 12px;
  text-align: left;
  font-weight: 600;
  white-space: nowrap;
}
.col-ev   { width: 170px; border-right: 1px solid var(--border); }
.col-idx  { width: 40px;  border-right: 1px solid var(--border); text-align: center; }
.col-type { width: 72px;  border-right: 1px solid var(--border); }
.col-lang { min-width: 280px; border-right: 1px solid var(--border); }
.col-lang:last-child { border-right: none; }

.lang-badge {
  display: inline-flex; align-items: center;
  padding: 1px 7px; border-radius: 10px;
  font-size: 10px; font-weight: 700; letter-spacing: 0.08em; margin-right: 4px;
  background: var(--accent-dim); color: var(--accent);
}

tbody tr { border-bottom: 1px solid var(--border); }
tbody tr:hover { background: var(--bg-hover); }

td { padding: 0; vertical-align: top; border-right: 1px solid var(--border); }
td:last-child { border-right: none; }

.td-ev {
  padding: 8px 12px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px; color: var(--text-2); vertical-align: middle;
}
.td-ev strong { display: block; color: var(--text-1); font-size: 11px; margin-bottom: 1px; }
.td-idx {
  padding: 8px; text-align: center;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px; color: var(--text-3); vertical-align: middle;
}
.td-type { padding: 6px 8px; vertical-align: middle; }

.type-badge {
  display: inline-block; padding: 2px 6px; border-radius: 4px;
  font-size: 9px; font-weight: 700; letter-spacing: 0.06em;
  text-transform: uppercase;
}
.type-string  { background: var(--accent-dim);  color: var(--accent); }
.type-int     { background: rgba(94,175,112,0.15); color: #5eaf70; }
.type-float   { background: rgba(230,162,60,0.15); color: #e6a23c; }

.cell-input {
  width: 100%; padding: 8px 12px;
  background: transparent; border: none; outline: none;
  color: var(--text-1); font-family: 'JetBrains Mono', monospace;
  font-size: 11px; line-height: 1.6; display: block;
  transition: background 0.1s;
}
.cell-input:focus {
  background: var(--accent-dim);
  box-shadow: inset 2px 0 0 var(--accent);
}

textarea.cell-input {
  min-height: 36px; resize: none; overflow: hidden;
}
textarea.cell-input.empty::placeholder { color: var(--text-3); font-style: italic; }

input.cell-input { height: 36px; }
input.cell-int:focus   { box-shadow: inset 2px 0 0 #5eaf70; background: rgba(94,175,112,0.07); }
input.cell-float:focus { box-shadow: inset 2px 0 0 #e6a23c; background: rgba(230,162,60,0.07); }

input.cell-invalid { box-shadow: inset 2px 0 0 var(--danger) !important; background: rgba(194,107,107,0.10) !important; }
.cell-wrap { display: flex; flex-direction: column; }
.diff-modified { background: rgba(230,162,60,0.08); box-shadow: inset 3px 0 0 #e6a23c; }
.diff-filled   { background: rgba(94,175,112,0.08); box-shadow: inset 3px 0 0 #5eaf70; }
.diff-cleared  { background: rgba(220,80,80,0.08);  box-shadow: inset 3px 0 0 #dc5050; }
.tm-chips {
  display: flex; flex-wrap: wrap; gap: 4px;
  padding: 4px 12px 6px;
  border-top: 1px solid var(--border);
  background: var(--bg-card);
}
.tm-chip {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  background: var(--accent-dim);
  color: var(--accent);
  border: 1px solid var(--accent-soft);
  cursor: pointer;
  white-space: nowrap;
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background 0.1s;
}
.tm-chip:hover { background: var(--accent-soft); }
</style>
