import { useState, useCallback } from 'react'
import { IconClose, IconCheck, IconLoading } from './Icons'
import { t, type Lang } from '../i18n'
import { useSettings } from '../SettingsContext'
import * as ipc from '../ipc'

type ToolKey = 'scannerPath' | 'transcriberPath' | 'splitterPath' | 'organizerPath'
type TestStatus = 'idle' | 'testing' | 'ok' | 'fail'

const toolKeys: { key: ToolKey; label: string }[] = [
  { key: 'scannerPath', label: 'scanner' },
  { key: 'transcriberPath', label: 'transcriber' },
  { key: 'splitterPath', label: 'splitter' },
  { key: 'organizerPath', label: 'organizer' },
]

const themeOptions = [
  { value: 'dark', labelKey: 'themeDark' as const },
  { value: 'light', labelKey: 'themeLight' as const },
]

const langOptions: { value: Lang; labelKey: 'langZh' | 'langEn' }[] = [
  { value: 'zh', labelKey: 'langZh' },
  { value: 'en', labelKey: 'langEn' },
]

export function SettingsForm() {
  const { settings, update } = useSettings()
  const [testStatus, setTestStatus] = useState<Record<ToolKey, TestStatus>>({} as any)

  const handleTest = useCallback(async (key: ToolKey) => {
    const path = settings[key]
    if (!path) return
    setTestStatus(prev => ({ ...prev, [key]: 'testing' }))
    try {
      const ok = await ipc.checkBinary(path)
      setTestStatus(prev => ({ ...prev, [key]: ok ? 'ok' : 'fail' }))
    } catch {
      setTestStatus(prev => ({ ...prev, [key]: 'fail' }))
    }
  }, [settings])

  const lang = settings.lang

  return (
    <>
      <div className="settings-section">
        <h4 className="settings-heading">{t(lang, 'settingsGeneral')}</h4>
        <div className="settings-row">
          <span className="settings-label">{t(lang, 'settingsTheme')}</span>
          <div className="settings-options">
            {themeOptions.map(o => (
              <button
                key={o.value}
                className={`btn btn-sm${settings.theme === o.value ? ' btn-active' : ''}`}
                onClick={() => update({ theme: o.value as 'dark' | 'light' })}
              >
                {t(lang, o.labelKey)}
              </button>
            ))}
          </div>
        </div>
        <div className="settings-row">
          <span className="settings-label">{t(lang, 'settingsLang')}</span>
          <div className="settings-options">
            {langOptions.map(o => (
              <button
                key={o.value}
                className={`btn btn-sm${settings.lang === o.value ? ' btn-active' : ''}`}
                onClick={() => update({ lang: o.value })}
              >
                {t(lang, o.labelKey)}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h4 className="settings-heading">{t(lang, 'settingsTools')}</h4>
        {toolKeys.map(({ key, label }) => (
          <div key={key} className="settings-tool-row">
            <span className="settings-label">{label}</span>
            <div className="settings-tool-input-row">
              <input
                type="text"
                className="input input-mono input-sm"
                value={settings[key]}
                onChange={e => update({ [key]: e.target.value })}
                placeholder="留空则使用 PATH 查找"
              />
              <button
                className="btn btn-sm"
                onClick={async () => {
                  const files = await ipc.pickAnyFile()
                  if (files.length > 0) update({ [key]: files[0] })
                }}
              >
                {t(lang, 'browse')}
              </button>
              <button
                className="btn btn-sm"
                onClick={() => handleTest(key)}
                disabled={!settings[key] || testStatus[key] === 'testing'}
                aria-label={t(lang, 'settingsTest')}
              >
                {testStatus[key] === 'testing' ? (
                  <IconLoading size={12} />
                ) : testStatus[key] === 'ok' ? (
                  <IconCheck size={12} />
                ) : testStatus[key] === 'fail' ? (
                  <IconClose size={12} />
                ) : null}
                {testStatus[key] === 'testing'
                  ? t(lang, 'settingsTesting')
                  : testStatus[key] === 'ok'
                  ? t(lang, 'settingsSuccess')
                  : testStatus[key] === 'fail'
                  ? t(lang, 'settingsFail')
                  : t(lang, 'settingsTest')}
              </button>
            </div>
          </div>
        ))}
      </div>

      <div className="settings-section">
        <h4 className="settings-heading">{t(lang, 'settingsSplit')}</h4>
        <div className="settings-row">
          <span className="settings-label">{t(lang, 'settingsOutputDir')}</span>
          <div className="settings-tool-input-row">
            <span className="field-path">
              {settings.splitOutputDir || t(lang, 'noSelection')}
            </span>
            <button
              className="btn btn-sm"
              onClick={async () => {
                const dirs = await ipc.pickFolder()
                if (dirs.length > 0) update({ splitOutputDir: dirs[0] })
              }}
            >
              {t(lang, 'browse')}
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
