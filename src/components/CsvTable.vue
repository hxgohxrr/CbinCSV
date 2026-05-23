<template>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th class="col-ev">EV_NAME</th>
          <th class="col-idx">#</th>
          <th v-for="lang in visibleLangs" :key="lang" class="col-lang">
            <span :class="`lang-badge lang-${lang}`">{{ lang.toUpperCase() }}</span>
            {{ langLabel(lang) }}
          </th>
        </tr>
      </thead>
      <tbody>
        <template v-for="ev in filteredEvNames" :key="ev">
          <tr class="row-group">
            <td :colspan="2 + visibleLangs.length">
              {{ ev }} — {{ entriesForEv(ev) }} {{ t('table.entries') }}
            </td>
          </tr>
          <tr v-for="idx in entryIndicesForEv(ev)" :key="`${ev}-${idx}`">
            <td v-if="idx === 0" class="td-ev" :rowspan="entryIndicesForEv(ev).length">
              <strong>{{ ev }}</strong>
            </td>
            <td class="td-idx">{{ idx }}</td>
            <td v-for="lang in visibleLangs" :key="lang" class="td-text">
              <textarea
                class="text-cell"
                :class="{ empty: !getVal(ev, lang, idx) }"
                :value="getVal(ev, lang, idx)"
                :placeholder="t('table.untranslated')"
                rows="1"
                @input="onInput(ev, lang, idx, $event)"
                @focus="autoResize"
                @blur="autoResize"
              />
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../stores/session'
import { useI18n } from '../i18n'

const session = useSessionStore()
const { t } = useI18n()

const visibleLangs = computed(() => session.visibleLangs)

const langLabels: Record<string, string> = {
  ja: '日本語',
  es: 'Español',
  en: 'English',
  fr: 'Français',
  de: 'Deutsch',
  it: 'Italiano'
}
function langLabel(l: string) { return langLabels[l] ?? l }

const filteredEvNames = computed(() => {
  const q = session.searchQuery.toLowerCase()
  if (!q) return session.evNames
  return session.evNames.filter(ev => {
    return session.files
      .filter(f => f.ev_name === ev)
      .some(f => f.entries.some(e => e.value.toLowerCase().includes(q)))
  })
})

function entryIndicesForEv(ev: string): number[] {
  const evFiles = session.files.filter(f => f.ev_name === ev)
  const max = Math.max(...evFiles.map(f => f.entries.length), 0)
  return Array.from({ length: max }, (_, i) => i)
}

function entriesForEv(ev: string): number {
  return entryIndicesForEv(ev).length
}

function getVal(ev: string, lang: string, idx: number): string {
  const file = session.files.find(f => f.ev_name === ev && f.language === lang)
  return file?.entries[idx]?.value ?? ''
}

function onInput(ev: string, lang: string, idx: number, event: Event) {
  const val = (event.target as HTMLTextAreaElement).value
  session.updateEntry(ev, lang, idx, val)
  autoResize(event)
}

function autoResize(event: Event) {
  const ta = event.target as HTMLTextAreaElement
  ta.style.height = 'auto'
  ta.style.height = ta.scrollHeight + 'px'
}
</script>

<style scoped>
.table-wrap { flex: 1; overflow: auto; }
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
.col-ev { width: 170px; border-right: 1px solid var(--border); }
.col-idx { width: 48px; border-right: 1px solid var(--border); text-align: center; }
.col-lang { min-width: 280px; border-right: 1px solid var(--border); }
.col-lang:last-child { border-right: none; }

.lang-badge {
  display: inline-flex;
  align-items: center;
  padding: 1px 7px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  margin-right: 4px;
  background: var(--accent-dim);
  color: var(--accent);
}

tbody tr { border-bottom: 1px solid var(--border); }
tbody tr:hover { background: var(--bg-hover); }

.row-group td {
  background: var(--bg-card);
  padding: 5px 12px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-3);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

td {
  padding: 0;
  vertical-align: top;
  border-right: 1px solid var(--border);
}
td:last-child { border-right: none; }

.td-ev {
  padding: 8px 12px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-2);
  vertical-align: middle;
}
.td-ev strong {
  display: block;
  color: var(--text-1);
  font-size: 11px;
  margin-bottom: 1px;
}
.td-idx {
  padding: 8px;
  text-align: center;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-3);
  vertical-align: middle;
}

.text-cell {
  width: 100%;
  min-height: 36px;
  padding: 8px 12px;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-1);
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  line-height: 1.6;
  resize: none;
  overflow: hidden;
  transition: background 0.1s;
  display: block;
}
.text-cell:focus {
  background: var(--accent-dim);
  box-shadow: inset 2px 0 0 var(--accent);
}
.text-cell.empty::placeholder { color: var(--text-3); font-style: italic; }
</style>
