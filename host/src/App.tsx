import { HashRouter, Routes, Route } from 'react-router-dom'
import MainPage from './pages/MainPage'
import SettingsPage from './pages/SettingsPage'
import './design-tokens.css'
import './App.css'

export default function App() {
  return (
    <HashRouter>
      <div id="app">
        <Routes>
          <Route path="/" element={<MainPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </div>
    </HashRouter>
  )
}
