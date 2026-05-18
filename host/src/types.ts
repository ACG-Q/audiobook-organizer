export type FileKind = 'Audio' | 'Video'
export type FileStatus = 'Waiting' | 'Running' | 'Completed' | 'Error' | 'Cancelled'

export interface AudioMetadata {
  title?: string
  artist?: string
  album?: string
  date?: string
  track?: string
  duration?: number
  bitrate?: number
  sample_rate?: number
  channels?: number
  language?: string
}

export interface ProgressInfo {
  current: number
  total: number
}

export interface Segment {
  id: number
  label: string
  start: number
  end: number
  path?: string
  status: FileStatus
  transcript?: string
  progress?: ProgressInfo
}

export interface SegmentInput {
  start: number
  end: number
  label: string
}

export interface FileEntry {
  id: number
  path: string
  kind: FileKind
  size: number
  status: FileStatus
  metadata?: AudioMetadata
  segments: Segment[]
  transcript?: string
  rename_preview?: string
  progress?: ProgressInfo
}
