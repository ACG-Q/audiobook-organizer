import { useState, useCallback } from 'react'
import type { FileEntry } from '../types'
import * as ipc from '../ipc'

export function useFiles() {
  const [files, setFiles] = useState<FileEntry[]>([])
  const [loading, setLoading] = useState(true)

  const refreshFiles = useCallback(async () => {
    const f = await ipc.getFiles()
    setFiles(f)
    return f
  }, [])

  const addFiles = useCallback(async (paths: string[]) => {
    const added = await ipc.addFiles(paths)
    setFiles(prev => [...prev, ...added])
    return added
  }, [])

  const removeFiles = useCallback(async (ids: number[]) => {
    await ipc.removeFiles(ids)
    setFiles(prev => prev.filter(f => !ids.includes(f.id)))
  }, [])

  return { files, setFiles, loading, setLoading, addFiles, removeFiles, refreshFiles }
}
