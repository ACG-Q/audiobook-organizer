import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type { FileEntry, SegmentInput } from './types'

export async function addFiles(paths: string[]): Promise<FileEntry[]> {
  return invoke('add_files', { paths })
}

export async function removeFiles(ids: number[]): Promise<void> {
  return invoke('remove_files', { ids })
}

export async function getFiles(): Promise<FileEntry[]> {
  return invoke('get_files')
}

export async function scanMetadata(ids: number[]): Promise<void> {
  return invoke('scan_metadata', { ids })
}

export async function transcribe(ids: number[], model: string, lang: string): Promise<void> {
  return invoke('transcribe', { ids, model, lang })
}

export async function splitVideo(id: number, segments: SegmentInput[], format: string): Promise<void> {
  return invoke('split_video', { id, segments, format })
}

export async function organize(ids: number[], template: string, dest: string, dryRun: boolean): Promise<void> {
  return invoke('organize', { ids, template, dest, dryRun })
}

export async function executePipeline(ids: number[]): Promise<void> {
  return invoke('execute_pipeline', { ids })
}

export async function checkBinary(path: string): Promise<boolean> {
  return invoke('check_binary', { path })
}

export async function cancelOperation(): Promise<void> {
  return invoke('cancel')
}

export async function pickFiles(): Promise<string[]> {
  const result = await open({
    multiple: true,
    title: '选择音频或视频文件',
    filters: [{
      name: '音视频文件',
      extensions: ['mp3', 'wav', 'flac', 'm4a', 'ogg', 'mp4', 'mkv', 'avi', 'mov', 'wma', 'aac', 'wmv'],
    }],
  })
  if (!result) return []
  return Array.isArray(result) ? result : [result]
}

export async function pickAnyFile(): Promise<string[]> {
  const result = await open({
    multiple: false,
    title: '选择可执行文件',
  })
  if (!result) return []
  return Array.isArray(result) ? result : [result]
}

export async function pickFolder(): Promise<string[]> {
  const result = await open({
    directory: true,
    multiple: false,
    title: '选择文件夹',
  })
  if (!result) return []
  return Array.isArray(result) ? result : [result]
}

export { listen, open as dialogOpen }
