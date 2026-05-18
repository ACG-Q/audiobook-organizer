import { useEffect, useRef } from 'react'
import * as ipc from '../ipc'
import type { FileEntry } from '../types'
import { t } from '../i18n'
import type { Lang } from '../i18n'

interface UseTauriEventsDeps {
  addLog: (msg: string) => void
  setFiles: (f: FileEntry[] | ((prev: FileEntry[]) => FileEntry[])) => void
  setPipelineRunning: (v: boolean) => void
  setDragOver: (v: boolean) => void
  lang: Lang
}

export function useTauriEvents(deps: UseTauriEventsDeps) {
  const { addLog, setFiles, setPipelineRunning, setDragOver, lang } = deps
  const unsubsRef = useRef<(() => void)[]>([])

  useEffect(() => {
    let cancelled = false
    const prev = unsubsRef.current
    unsubsRef.current = []

    const setup = async () => {
      prev.forEach(fn => fn())

      const unsubs = await Promise.all([
        ipc.listen<{ file_id: number; current: number; total: number; status: string }>('progress', e => {
          setFiles(prev => prev.map(f =>
            f.id === e.payload.file_id
              ? { ...f, progress: { current: e.payload.current, total: e.payload.total }, status: e.payload.status as any }
              : f
          ))
        }),
        ipc.listen<{ message: string; level: string }>('log', e => {
          addLog(e.payload.message)
        }),
        ipc.listen<{ success: number; failed: number }>('pipeline_done', e => {
          addLog(`${t(lang, 'pipelineDone')}: ${e.payload.success} success, ${e.payload.failed} failed`)
          setPipelineRunning(false)
          ipc.getFiles().then(setFiles)
        }),
        ipc.listen('segment_added', () => {
          ipc.getFiles().then(setFiles)
        }),
        ipc.listen<{ paths: string[] }>('tauri://drag-drop', async e => {
          setDragOver(false)
          const paths = e.payload.paths
          if (paths.length > 0) {
            const added = await ipc.addFiles(paths)
            setFiles(prev => [...prev, ...added])
            addLog(t(lang, 'added', { n: added.length }))
          }
        }),
        ipc.listen('tauri://drag-over', () => setDragOver(true)),
        ipc.listen('tauri://drag-leave', () => setDragOver(false)),
      ])

      if (cancelled) {
        unsubs.forEach(fn => fn())
      } else {
        unsubsRef.current = unsubs
      }
    }

    setup()

    return () => {
      cancelled = true
      unsubsRef.current.forEach(fn => fn())
      unsubsRef.current = []
    }
  }, [addLog, setFiles, setPipelineRunning, setDragOver, lang])
}
