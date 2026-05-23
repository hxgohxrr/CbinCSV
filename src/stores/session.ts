import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface TextEntry {
  index: number
  value: string
}

export interface FileEntry {
  path: string
  ev_name: string
  language: string
  entries: TextEntry[]
}

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

  async function exportCsv(outputPath: string, langs: string[], format: 'single' | 'per_file', separator: string) {
    error.value = null
    progress.value = { current: 0, total: files.value.length, message: '' }
    const unlisten = await listen<Progress>('progress', e => {
      progress.value = e.payload
    })
    try {
      await invoke('export_csv', {
        session: { files: files.value },
        format,
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

  async function importCsvRows(csvPath: string, separator: string) {
    error.value = null
    try {
      const rows = await invoke<[string, number, Record<string, string>][]>('import_csv_rows', { csvPath, separator })
      for (const [ev_name, index, langMap] of rows) {
        for (const [language, value] of Object.entries(langMap)) {
          updateEntry(ev_name, language, index, value)
        }
      }
    } catch (e) {
      error.value = String(e)
    }
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
  }

  return {
    files,
    progress,
    error,
    visibleLangs,
    searchQuery,
    parseMode,
    languages,
    evNames,
    totalEntries,
    untranslatedCount,
    loadFiles,
    loadFolder,
    updateEntry,
    exportCsv,
    importCsvRows,
    writeCfgbin,
    clear
  }
})
