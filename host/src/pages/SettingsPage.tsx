import { Link } from 'react-router-dom'
import { t } from '../i18n'
import { useSettings } from '../SettingsContext'
import { SettingsForm } from '../components/Settings'
import { IconArrowLeft } from '../components/Icons'

export default function SettingsPage() {
  const { settings } = useSettings()
  const lang = settings.lang

  return (
    <>
      <header id="topbar">
        <Link className="btn" to="/" aria-label="Back">
          <IconArrowLeft size={18} /> {t(lang, 'settings')}
        </Link>
      </header>

      <div id="settings-page">
        <div id="settings-header">
          <h2 id="settings-title">{t(lang, 'settings')}</h2>
        </div>
        <div id="settings-body">
          <SettingsForm />
        </div>
      </div>
    </>
  )
}
