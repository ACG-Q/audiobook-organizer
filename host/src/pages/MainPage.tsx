import { useState, useEffect, useCallback, useRef } from 'react'
import { Link } from 'react-router-dom'
import type { SegmentInput } from '../types'
import * as ipc from '../ipc'
import { t } from '../i18n'
import { useSettings } from '../SettingsContext'
import { useLog } from '../hooks/useLog'
import { useFiles } from '../hooks/useFiles'
import { useFileSelection } from '../hooks/useFileSelection'
import { usePipeline } from '../hooks/usePipeline'
import { useTauriEvents } from '../hooks/useTauriEvents'
import { FileTable } from '../components/FileTable'
import { Modal } from '../components/Modal'
import {
  IconFile,
  IconFolder,
  IconScan,
  IconTranscribe,
  IconOrganize,
  IconSplit,
  IconPlay,
  IconStop,
  IconRemove,
  IconTrash,
  IconLoading,
  IconSettings,
} from '../components/Icons'

export default function MainPage() {
  const { settings } = useSettings()
  const lang = settings.lang

  const { logs, addLog, logRef } = useLog()
  const { files, setFiles, loading, setLoading, addFiles, removeFiles, refreshFiles } = useFiles()
  const { selected, selectAll, setSelectAll, handleCheck, checkedIds, firstCheckedId, setSelected } = useFileSelection(files)
  const { pipelineRunning, setPipelineRunning, handleCancel } = usePipeline()
  const [dragOver, setDragOver] = useState(false)
  const tableRef = useRef<HTMLDivElement>(null)

  useTauriEvents({ addLog, setFiles, setPipelineRunning, setDragOver, lang })

  useEffect(() => {
    setLoading(true)
    ipc.getFiles().then(f => { setFiles(f); setLoading(false) }).catch(() => setLoading(false))
  }, [])

  const [contextPos, setContextPos] = useState<{ x: number; y: number } | null>(null)
  const [contextFileId, setContextFileId] = useState<number | null>(null)
  const [showSplit, setShowSplit] = useState(false)
  const [splitFileId, setSplitFileId] = useState(0)
  const [sampleDuration, setSampleDuration] = useState(30)
  const [showOrganize, setShowOrganize] = useState(false)
  const [organizeIds, setOrganizeIds] = useState<number[]>([])
  const [orgTemplate, setOrgTemplate] = useState('{{artist}}/{{album}}/{{format track "02"}} - {{title}}.{{ext}}')
  const [orgDest, setOrgDest] = useState('')
  const [orgDryRun, setOrgDryRun] = useState(false)

  const handleAddFiles = useCallback(async () => {
    const paths = await ipc.pickFiles()
    if (paths.length === 0) return
    const added = await addFiles(paths)
    addLog(t(lang, 'added', { n: added.length }))
  }, [addLog, addFiles, lang])

  const handleAddFolder = useCallback(async () => {
    const paths = await ipc.pickFolder()
    if (paths.length === 0) return
    const added = await addFiles(paths)
    addLog(t(lang, 'added', { n: added.length }))
  }, [addLog, addFiles, lang])

  const handleRemove = useCallback(async (ids: number[]) => {
    await removeFiles(ids)
    setSelected(prev => { const s = new Set(prev); ids.forEach(id => s.delete(id)); return s })
    addLog(t(lang, 'removed', { n: ids.length }))
  }, [removeFiles, addLog, lang])

  const handleScan = useCallback(async () => {
    if (checkedIds.length === 0) { addLog(t(lang, 'pleaseSelect')); return }
    addLog(t(lang, 'scanning'))
    await ipc.scanMetadata(checkedIds)
    await refreshFiles()
    addLog(t(lang, 'scanDone'))
  }, [checkedIds, addLog, refreshFiles, lang])

  const handleTranscribe = useCallback(async () => {
    if (checkedIds.length === 0) { addLog(t(lang, 'pleaseSelect')); return }
    addLog(t(lang, 'transcribing'))
    await ipc.transcribe(checkedIds, 'large-v3-turbo', 'zh')
    await refreshFiles()
    addLog(t(lang, 'transcribeDone'))
  }, [checkedIds, addLog, refreshFiles, lang])

  const handlePipeline = useCallback(async () => {
    if (checkedIds.length === 0) { addLog(t(lang, 'pleaseSelect')); return }
    setPipelineRunning(true)
    addLog(t(lang, 'pipelineStart'))
    try {
      await ipc.executePipeline(checkedIds)
      await refreshFiles()
      addLog(t(lang, 'pipelineDone'))
    } catch (e) {
      addLog(`${t(lang, 'pipelineError')}: ${e}`)
    }
    setPipelineRunning(false)
  }, [checkedIds, addLog, refreshFiles, lang, setPipelineRunning])

  const onCancel = useCallback(async () => {
    await handleCancel(addLog)
  }, [handleCancel, addLog])

  useEffect(() => {
    const el = tableRef.current
    if (!el) return
    const onDragOver = (e: DragEvent) => { e.preventDefault(); setDragOver(true) }
    const onDragLeave = (e: DragEvent) => {
      if (el.contains(e.relatedTarget as Node)) return
      setDragOver(false)
    }
    el.addEventListener('dragover', onDragOver)
    el.addEventListener('dragleave', onDragLeave)
    return () => {
      el.removeEventListener('dragover', onDragOver)
      el.removeEventListener('dragleave', onDragLeave)
    }
  }, [])

  const handleContext = useCallback((e: React.MouseEvent, fileId: number) => {
    e.preventDefault()
    setContextPos({ x: e.clientX, y: e.clientY })
    setContextFileId(fileId)
  }, [])

  useEffect(() => {
    if (contextPos) {
      const close = () => setContextPos(null)
      document.addEventListener('click', close)
      return () => document.removeEventListener('click', close)
    }
  }, [contextPos])

  const execContextAction = useCallback(async (action: string) => {
    setContextPos(null)
    if (contextFileId === null) return
    const id = contextFileId
    switch (action) {
      case 'scan':
        await ipc.scanMetadata([id]); await refreshFiles(); addLog(t(lang, 'scanDone')); break
      case 'transcribe':
        await ipc.transcribe([id], 'large-v3-turbo', 'zh'); await refreshFiles(); addLog(t(lang, 'transcribeDone')); break
      case 'organize':
        setOrganizeIds([id]); setShowOrganize(true); break
      case 'split':
        setSplitFileId(id); setShowSplit(true); break
      case 'remove':
        await handleRemove([id]); break
    }
  }, [contextFileId, handleRemove, refreshFiles, addLog, lang])

  const splitFile = files.find(f => f.id === splitFileId)
  const isVideoSplit = splitFile?.kind === 'Video'

  const doSplit = useCallback(async () => {
    setShowSplit(false)
    let segs: SegmentInput[]
    if (isVideoSplit) {
      segs = [{ start: 0, end: 0, label: 'full' }]
    } else {
      segs = [{ start: 0, end: sampleDuration, label: 'sample' }]
    }
    try {
      await ipc.splitVideo(splitFileId, segs, 'mp3')
      await refreshFiles()
      addLog(t(lang, 'splitDone'))
    } catch (e) {
      addLog(`${t(lang, 'splitError')}: ${e}`)
    }
  }, [isVideoSplit, sampleDuration, splitFileId, refreshFiles, addLog, lang])

  const doOrganize = useCallback(async () => {
    if (!orgTemplate.trim()) { addLog(t(lang, 'templatePlaceholder')); return }
    if (!orgDest) { addLog(t(lang, 'destRequired')); return }
    setShowOrganize(false)
    try {
      await ipc.organize(organizeIds, orgTemplate, orgDest, orgDryRun)
      addLog(t(lang, 'organizeDone'))
    } catch (e) {
      addLog(`${t(lang, 'organizeError')}: ${e}`)
    }
  }, [orgTemplate, orgDest, orgDryRun, organizeIds, addLog, lang])

  const pickOrgDest = useCallback(async () => {
    const dir = await ipc.pickFolder()
    if (dir.length > 0) setOrgDest(dir[0])
  }, [])

  return (
    <>
      <header id="topbar">
        <span id="logo">{t(lang, 'appTitle')}</span>
        <div id="topbar-actions">
          <button className="btn" onClick={handleAddFiles}>
            <IconFile size={14} /> {t(lang, 'addFile')}
          </button>
          <button className="btn" onClick={handleAddFolder}>
            <IconFolder size={14} /> {t(lang, 'addFolder')}
          </button>
          {files.length > 0 && (
            <button className="btn btn-danger" onClick={() => handleRemove(checkedIds)} disabled={checkedIds.length === 0}>
              <IconTrash size={14} /> {t(lang, 'delete')}
            </button>
          )}
          <Link className="btn" to="/settings" aria-label={t(lang, 'settings')}>
            <IconSettings size={14} />
          </Link>
        </div>
      </header>

      <main id="main">
        {loading ? (
          <div className="loading-center">
            <IconLoading size={24} />
            <span>{t(lang, 'loading')}</span>
          </div>
        ) : (
          <FileTable
            files={files}
            selected={selected}
            selectAll={selectAll}
            onSelectAll={setSelectAll}
            onCheck={handleCheck}
            onContext={handleContext}
            onAddFile={handleAddFiles}
            onAddFolder={handleAddFolder}
            dragOver={dragOver}
            tableRef={tableRef}
          />
        )}
      </main>

      <footer id="bottombar">
        <div id="bottombar-left">
          <span id="file-count">
            {t(lang, 'fileCount', { n: files.length })}
            {checkedIds.length > 0 ? ` (${t(lang, 'selected', { n: checkedIds.length })})` : ''}
          </span>
          <button className="btn" onClick={handleScan} disabled={pipelineRunning || checkedIds.length === 0}>
            <IconScan size={14} /> {t(lang, 'scan')}
          </button>
          <button className="btn" onClick={handleTranscribe} disabled={pipelineRunning || checkedIds.length === 0}>
            <IconTranscribe size={14} /> {t(lang, 'transcribe')}
          </button>
        </div>
        <div id="bottombar-right">
          <button className="btn" onClick={() => {
            if (checkedIds.length === 0) { addLog(t(lang, 'pleaseSelect')); return }
            setOrganizeIds(checkedIds); setShowOrganize(true)
          }} disabled={pipelineRunning || checkedIds.length === 0}>
            <IconOrganize size={14} /> {t(lang, 'organize')}
          </button>
          <button className="btn" onClick={() => {
            if (checkedIds.length === 0) { addLog(t(lang, 'pleaseSelect')); return }
            setSplitFileId(firstCheckedId); setShowSplit(true)
          }} disabled={pipelineRunning || checkedIds.length === 0}>
            <IconSplit size={14} /> {t(lang, 'split')}
          </button>
          <button className="btn btn-primary" onClick={handlePipeline} disabled={pipelineRunning || checkedIds.length === 0}>
            {pipelineRunning ? <IconLoading size={14} /> : <IconPlay size={14} />} {t(lang, 'execute')}
          </button>
          <button className="btn btn-danger" onClick={onCancel} disabled={!pipelineRunning}>
            <IconStop size={14} /> {t(lang, 'stop')}
          </button>
        </div>
      </footer>

      <section id="log-panel">
        <div id="log-header">
          {t(lang, 'logTitle')}
          {logs.length > 0 && <span id="log-count">{logs.length}</span>}
        </div>
        <div id="log-content" ref={logRef}>
          {logs.length === 0 ? (
            <div className="log-empty">{t(lang, 'logEmpty')}</div>
          ) : (
            logs.map((msg, i) => <div key={i} className="log-line">{msg}</div>)
          )}
        </div>
      </section>

      {contextPos && (
        <div className="context-menu" style={{ left: contextPos.x, top: contextPos.y }} role="menu">
          <button className="context-item" role="menuitem" onClick={() => execContextAction('scan')}>
            <IconScan size={14} /> {t(lang, 'ctxScan')}
          </button>
          <button className="context-item" role="menuitem" onClick={() => execContextAction('organize')}>
            <IconOrganize size={14} /> {t(lang, 'ctxOrganize')}
          </button>
          <button className="context-item" role="menuitem" onClick={() => execContextAction('transcribe')}>
            <IconTranscribe size={14} /> {t(lang, 'ctxTranscribe')}
          </button>
          <button className="context-item" role="menuitem" onClick={() => execContextAction('split')}>
            <IconSplit size={14} /> {t(lang, 'ctxSplit')}
          </button>
          <div className="context-divider" />
          <button className="context-item context-danger" role="menuitem" onClick={() => execContextAction('remove')}>
            <IconRemove size={14} /> {t(lang, 'ctxRemove')}
          </button>
        </div>
      )}

      <Modal
        open={showSplit}
        onClose={() => setShowSplit(false)}
        title={isVideoSplit ? t(lang, 'splitVideoExtract') : t(lang, 'splitAudioSample')}
      >
        {isVideoSplit ? (
          <div className="split-info">
            <p className="split-desc">{t(lang, 'splitDescVideo')}</p>
          </div>
        ) : (
          <div className="split-option">
            <p className="split-desc">{t(lang, 'splitDescAudio', { s: sampleDuration })}</p>
            <label className="field-label" style={{ marginTop: 12 }}>{t(lang, 'splitSampleDuration')}</label>
            <input
              type="number"
              className="input"
              value={sampleDuration}
              min={5}
              max={300}
              onChange={e => setSampleDuration(Math.max(5, Math.min(300, Number(e.target.value))))}
            />
          </div>
        )}
        <div className="dialog-actions">
          <button className="btn btn-primary" onClick={doSplit}>{t(lang, 'modalConfirm')}</button>
          <button className="btn" onClick={() => setShowSplit(false)}>{t(lang, 'modalCancel')}</button>
        </div>
      </Modal>

      <Modal open={showOrganize} onClose={() => setShowOrganize(false)} title={t(lang, 'organize')}>
        <div className="field-group">
          <label className="field-label">{t(lang, 'orgTemplate')}</label>
          <input
            type="text"
            className="input input-mono"
            value={orgTemplate}
            onChange={e => setOrgTemplate(e.target.value)}
          />
        </div>
        <div className="field-group">
          <label className="field-label">{t(lang, 'orgDest')}</label>
          <div className="field-row">
            <span className="field-path">{orgDest || t(lang, 'noSelection')}</span>
            <button className="btn" onClick={pickOrgDest}>{t(lang, 'browse')}</button>
          </div>
        </div>
        <div className="field-group">
          <label className="checkbox-row">
            <input type="checkbox" checked={orgDryRun} onChange={e => setOrgDryRun(e.target.checked)} />
            <span>{t(lang, 'dryRun')}</span>
          </label>
        </div>
        <div className="dialog-actions">
          <button className="btn btn-primary" onClick={doOrganize}>{t(lang, 'modalConfirm')}</button>
          <button className="btn" onClick={() => setShowOrganize(false)}>{t(lang, 'modalCancel')}</button>
        </div>
      </Modal>
    </>
  )
}
