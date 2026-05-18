import { useState, useCallback } from 'react'
import * as ipc from '../ipc'

export function usePipeline() {
  const [pipelineRunning, setPipelineRunning] = useState(false)

  const handleCancel = useCallback(async (addLog: (msg: string) => void) => {
    addLog('正在取消...')
    await ipc.cancelOperation()
    setPipelineRunning(false)
  }, [])

  return { pipelineRunning, setPipelineRunning, handleCancel }
}
