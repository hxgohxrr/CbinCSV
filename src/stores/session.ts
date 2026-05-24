import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface TextEntry {
  index: number
  field_type: string
  value: string
}

export interface FileEntry {
  path: string
  ev_name: string
  language: string
  entries: TextEntry[]
  mode?: 'standard' | 'nnk' | 'rdbn'
}

export type GameProfile = 'level5' | 'yokai' | 'tt' | 'nnk'

export interface SessionData {
  files: FileEntry[]
}

export interface Progress {
  current: number
  total: number
  message: string
}

export const useSessionStore = defineStore('session', () => {
  const files = ref<FileEntry[]>([])
  const progress = ref<Progress | null>(null)
  const error = ref<string | null>(null)
  const visibleLangs = ref<string[]>([])
  const searchQuery = ref('')
  const parseMode = ref<'standard' | 'nnk'>('standard')
  const gameProfile = ref<GameProfile>('level5')
  const diffMode = ref(false)
  const originalValues = ref<Map<string, string>>(new Map())

  const languages = computed(() => {
    const set = new Set<string>()
    for (const f of files.value) set.add(f.language)
    return [...set].sort()
  })

  const evNames = computed(() => {
    const set = new Set<string>()
    for (const f of files.value) set.add(f.ev_name)
    return [...set].sort()
  })

  const dirtyCount = computed(() => {
    let count = 0
    for (const f of files.value) {
      for (const e of f.entries) {
        const key = `${f.path}:${e.index}`
        const orig = originalValues.value.get(key)
        if (orig !== undefined && e.value !== orig) count++
      }
    }
    return count
  })

  const formatSummary = computed(() => {
    const counts = { t2b: 0, rdbn: 0, nnk: 0 }
    for (const f of files.value) {
      if (f.mode === 'rdbn') counts.rdbn++
      else if (f.mode === 'nnk') counts.nnk++
      else counts.t2b++
    }
    return counts
  })

  const totalEntries = computed(() => {
    let count = 0
    for (const f of files.value) count += f.entries.length
    return count
  })

  const untranslatedCount = computed(() => {
    if (languages.value.length < 2) return 0
    let count = 0
    for (const ev of evNames.value) {
      const byLang: Record<string, FileEntry | undefined> = {}
      for (const f of files.value) {
        if (f.ev_name === ev) byLang[f.language] = f
      }
      const langs = Object.keys(byLang)
      if (langs.length < 2) continue
      const base = byLang[langs[0]]!
      for (let i = 0; i < base.entries.length; i++) {
        for (const lang of langs.slice(1)) {
          const entry = byLang[lang]?.entries[i]
          if (!entry || entry.value === '') count++
        }
      }
    }
    return count
  })

  function mergeSession(incoming: FileEntry[]) {
    for (const inFile of incoming) {
      const idx = files.value.findIndex(f => f.path === inFile.path)
      if (idx >= 0) {
        files.value[idx] = inFile
      } else {
        files.value.push(inFile)
      }
    }
    for (const lang of languages.value) {
      if (!visibleLangs.value.includes(lang)) {
        visibleLangs.value.push(lang)
      }
    }
    for (const inFile of incoming) {
      for (const entry of inFile.entries) {
        const key = `${inFile.path}:${entry.index}`
        if (!originalValues.value.has(key)) {
          originalValues.value.set(key, entry.value)
        }
      }
    }
    const hasRdbn = incoming.some(f => f.mode === 'rdbn')
    const hasNnk  = incoming.some(f => f.mode === 'nnk')
    if (hasNnk) gameProfile.value = 'nnk'
    else if (hasRdbn && gameProfile.value === 'level5') gameProfile.value = 'yokai'
  }

  async function loadFiles(paths: string[]) {
    error.value = null
    progress.value = { current: 0, total: paths.length, message: '' }
    const unlisten = await listen<Progress>('progress', e => {
      progress.value = e.payload
    })
    try {
      const result = await invoke<FileEntry[]>('parse_files', { paths, mode: parseMode.value })
      mergeSession(result)
    } catch (e) {
      error.value = String(e)
    } finally {
      unlisten()
      progress.value = null
    }
  }

  async function loadFolder(folder: string) {
    error.value = null
    progress.value = { current: 0, total: 0, message: '' }
    const unlisten = await listen<Progress>('progress', e => {
      progress.value = e.payload
    })
    try {
      const result = await invoke<FileEntry[]>('parse_folder', { folder, mode: parseMode.value })
      mergeSession(result)
    } catch (e) {
      error.value = String(e)
    } finally {
      unlisten()
      progress.value = null
    }
  }

  function updateEntry(ev_name: string, language: string, index: number, value: string) {
    const file = files.value.find(f => f.ev_name === ev_name && f.language === language)
    if (!file) return
    const entry = file.entries.find(e => e.index === index)
    if (entry) entry.value = value
  }

  function replaceInEntries(
    find: string,
    replace: string,
    lang: string,
    caseSensitive: boolean
  ): number {
    if (!find) return 0
    const escaped = find.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(escaped, caseSensitive ? 'g' : 'gi')
    let count = 0
    for (const file of files.value) {
      if (lang !== 'all' && file.language !== lang) continue
      for (const entry of file.entries) {
        if (entry.field_type !== 'string') continue
        const newVal = entry.value.replace(regex, replace)
        if (newVal !== entry.value) {
          entry.value = newVal
          count++
        }
      }
    }
    return count
  }

  function getEntryDiff(ev: string, lang: string, idx: number): 'modified' | 'filled' | 'cleared' | null {
    const file = files.value.find(f => f.ev_name === ev && f.language === lang)
    if (!file) return null
    const entry = file.entries.find(e => e.index === idx)
    if (!entry) return null
    const key = `${file.path}:${idx}`
    const orig = originalValues.value.get(key)
    if (orig === undefined || orig === entry.value) return null
    if (!orig && entry.value) return 'filled'
    if (orig && !entry.value) return 'cleared'
    return 'modified'
  }

  function getTranslationSuggestions(
    sourceText: string,
    targetLang: string,
    sourceLang: string
  ): string[] {
    if (!sourceText || !sourceLang || sourceLang === targetLang) return []
    const seen = new Set<string>()
    for (const srcFile of files.value) {
      if (srcFile.language !== sourceLang) continue
      for (const srcEntry of srcFile.entries) {
        if (srcEntry.field_type !== 'string' || srcEntry.value !== sourceText) continue
        const tgtFile = files.value.find(f => f.ev_name === srcFile.ev_name && f.language === targetLang)
        const tgtEntry = tgtFile?.entries.find(e => e.index === srcEntry.index)
        if (tgtEntry?.value) seen.add(tgtEntry.value)
      }
    }
    return [...seen].slice(0, 5)
  }

  function countMatches(find: string, lang: string, caseSensitive: boolean): number {
    if (!find) return 0
    const escaped = find.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(escaped, caseSensitive ? 'g' : 'gi')
    let count = 0
    for (const file of files.value) {
      if (lang !== 'all' && file.language !== lang) continue
      for (const entry of file.entries) {
        if (entry.field_type !== 'string') continue
        const m = entry.value.match(regex)
        if (m) count += m.length
      }
    }
    return count
  }

  async function exportFormatted(
    outputPath: string,
    langs: string[],
    format: 'single' | 'per_file',
    fileFormat: 'csv' | 'json' | 'yaml' | 'toml' | 'xml',
    separator: string
  ) {
    error.value = null
    progress.value = { current: 0, total: files.value.length, message: '' }
    const unlisten = await listen<Progress>('progress', e => {
      progress.value = e.payload
    })
    try {
      await invoke('export_formatted', {
        session: { files: files.value },
        format,
        fileFormat,
        outputPath,
        langs,
        separator
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      unlisten()
      progress.value = null
    }
  }

  async function importFormatted(filePath: string, separator: string) {
    error.value = null
    try {
      const rows = await invoke<[string, string, number, Record<string, string>][]>('import_formatted', { filePath, separator })
      for (const [ev_name, _field_type, index, langMap] of rows) {
        for (const [language, value] of Object.entries(langMap)) {
          updateEntry(ev_name, language, index, value)
        }
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  async function syncSession() {
    try {
      await invoke('sync_session', { session: { files: files.value } })
    } catch (_) {}
  }

  async function startMcp(port: number) {
    error.value = null
    try {
      await invoke('start_mcp_server', { port })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function stopMcp() {
    try {
      await invoke('stop_mcp_server')
    } catch (_) {}
  }

  async function writeCfgbin(outputDir: string) {
    error.value = null
    progress.value = { current: 0, total: files.value.length, message: '' }
    const unlisten = await listen<Progress>('progress', e => {
      progress.value = e.payload
    })
    try {
      await invoke('write_cfgbin', {
        session: { files: files.value },
        outputDir
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      unlisten()
      progress.value = null
    }
  }

  function clear() {
    files.value = []
    progress.value = null
    error.value = null
    visibleLangs.value = []
    searchQuery.value = ''
    gameProfile.value = 'level5'
    diffMode.value = false
    originalValues.value = new Map()
  }

  return {
    files,
    progress,
    error,
    visibleLangs,
    searchQuery,
    parseMode,
    gameProfile,
    diffMode,
    languages,
    evNames,
    totalEntries,
    dirtyCount,
    formatSummary,
    untranslatedCount,
    getEntryDiff,
    loadFiles,
    loadFolder,
    updateEntry,
    replaceInEntries,
    countMatches,
    getTranslationSuggestions,
    exportFormatted,
    importFormatted,
    syncSession,
    startMcp,
    stopMcp,
    writeCfgbin,
    clear
  }
})
