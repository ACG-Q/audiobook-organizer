import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'
import type { Lang } from './i18n'

export interface Settings {
  lang: Lang
  theme: 'dark' | 'light'
  scannerPath: string
  transcriberPath: string
  splitterPath: string
  organizerPath: string
  splitOutputDir: string
}

const defaults: Settings = {
  lang: 'zh',
  theme: 'dark',
  scannerPath: '',
  transcriberPath: '',
  splitterPath: '',
  organizerPath: '',
  splitOutputDir: '',
}

const STORAGE_KEY = 'audiobook-settings'

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...defaults, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return defaults
}

function saveSettings(s: Settings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(s))
  } catch { /* ignore */ }
}

interface SettingsCtx {
  settings: Settings
  update: (partial: Partial<Settings>) => void
  reset: () => void
}

const Ctx = createContext<SettingsCtx>(null!)

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(loadSettings)

  useEffect(() => {
    saveSettings(settings)
    document.documentElement.setAttribute('data-theme', settings.theme)
  }, [settings])

  const update = useCallback((partial: Partial<Settings>) => {
    setSettings(prev => ({ ...prev, ...partial }))
  }, [])

  const reset = useCallback(() => {
    setSettings(defaults)
  }, [])

  return (
    <Ctx.Provider value={{ settings, update, reset }}>
      {children}
    </Ctx.Provider>
  )
}

export function useSettings() {
  return useContext(Ctx)
}
