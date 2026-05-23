import { computed } from 'vue'
import { useSettingsStore } from '../stores/settings'

import es from '@lang/es.json'
import en from '@lang/en.json'
import fr from '@lang/fr.json'
import de from '@lang/de.json'
import ja from '@lang/ja.json'
import it from '@lang/it.json'

const locales: Record<string, Record<string, string>> = { es, en, fr, de, ja, it }

export function useI18n() {
  const settings = useSettingsStore()

  const t = computed(() => (key: string, vars?: Record<string, string | number>) => {
    const dict = locales[settings.locale] ?? locales['es']
    let str = dict[key] ?? locales['es'][key] ?? key
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        str = str.replace(`{${k}}`, String(v))
      }
    }
    return str
  })

  return { t: t.value }
}
