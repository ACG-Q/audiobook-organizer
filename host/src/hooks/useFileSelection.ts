import { useState, useCallback, useEffect } from 'react'
import type { FileEntry } from '../types'

export function useFileSelection(files: FileEntry[]) {
  const [selected, setSelected] = useState<Set<number>>(new Set())
  const [selectAll, setSelectAll] = useState(false)

  useEffect(() => {
    if (selectAll) {
      setSelected(new Set(files.map(f => f.id)))
    } else if (files.length > 0) {
      setSelected(new Set())
    }
  }, [selectAll, files.length])

  const handleCheck = useCallback((id: number, checked: boolean) => {
    setSelected(prev => {
      const s = new Set(prev)
      checked ? s.add(id) : s.delete(id)
      return s
    })
  }, [])

  const checkedIds = files.filter(f => selected.has(f.id)).map(f => f.id)
  const firstCheckedId = checkedIds[0]

  return { selected, selectAll, setSelectAll, handleCheck, checkedIds, firstCheckedId, setSelected }
}
