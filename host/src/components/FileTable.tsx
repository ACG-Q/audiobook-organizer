import type { FileEntry, FileStatus } from '../types'
import { ProgressBar } from './ProgressBar'
import { EmptyState } from './EmptyState'
import {
  IconAudio,
  IconVideo,
} from './Icons'

function filename(path: string) {
  return path.replace(/\\/g, '/').split('/').pop() || path
}

function formatSize(bytes: number) {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const val = (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)
  return `${val} ${units[i]}`
}

function formatTime(secs?: number) {
  if (!secs || secs <= 0) return '00:00:00'
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.floor(secs % 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function statusLabel(status: FileStatus) {
  const map: Record<FileStatus, string> = {
    Waiting: '等待',
    Running: '进行中',
    Completed: '完成',
    Error: '错误',
    Cancelled: '已取消',
  }
  return map[status] || status
}

function metaTags(m: FileEntry['metadata']) {
  if (!m) return <span className="text-muted">─</span>
  const parts: string[] = []
  if (m.artist && m.artist !== 'unknown') parts.push(m.artist)
  if (m.album && m.album !== 'unknown') parts.push(m.album)
  if (m.title && m.title !== 'unknown') parts.push(m.title)
  if (m.duration && m.duration > 0) parts.push(formatTime(m.duration))
  return parts.map((p, i) => <span key={i} className="tag tag-meta">{p}</span>)
}

interface FileTableProps {
  files: FileEntry[]
  selected: Set<number>
  selectAll: boolean
  onSelectAll: (v: boolean) => void
  onCheck: (id: number, checked: boolean) => void
  onContext: (e: React.MouseEvent, fileId: number) => void
  onAddFile: () => void
  onAddFolder: () => void
  dragOver: boolean
  tableRef: React.RefObject<HTMLDivElement | null>
}

export function FileTable({
  files,
  selected,
  selectAll,
  onSelectAll,
  onCheck,
  onContext,
  onAddFile,
  onAddFolder,
  dragOver,
  tableRef,
}: FileTableProps) {
  if (files.length === 0) {
    return (
      <div
        ref={tableRef}
        className={`table-area${dragOver ? ' drag-over' : ''}`}
      >
        <div className="drop-indicator">
          <div className="drop-icon">+</div>
          <div className="drop-text">拖拽文件或文件夹到此处</div>
        </div>
        <EmptyState onAddFile={onAddFile} onAddFolder={onAddFolder} />
      </div>
    )
  }

  return (
    <div
      ref={tableRef}
      className={`table-area${dragOver ? ' drag-over' : ''}`}
    >
      <div className="drop-indicator">
        <div className="drop-icon">+</div>
        <div className="drop-text">拖拽文件或文件夹到此处</div>
      </div>
      <table className="file-table">
        <thead>
          <tr>
            <th className="col-check">
              <input
                type="checkbox"
                checked={selectAll}
                onChange={e => onSelectAll(e.target.checked)}
                aria-label="全选"
              />
            </th>
            <th className="col-name">文件名</th>
            <th className="col-size">大小</th>
            <th className="col-meta">元数据</th>
            <th className="col-split">拆分结果</th>
            <th className="col-trans">识别结果</th>
            <th className="col-rename">重命名预览</th>
            <th className="col-progress">进度</th>
            <th className="col-status">状态</th>
          </tr>
        </thead>
        <tbody>
          {files.map(file => (
            <FileRow
              key={file.id}
              file={file}
              checked={selected.has(file.id)}
              onCheck={onCheck}
              onContext={onContext}
            />
          ))}
        </tbody>
      </table>
    </div>
  )
}

interface FileRowProps {
  file: FileEntry
  checked: boolean
  onCheck: (id: number, checked: boolean) => void
  onContext: (e: React.MouseEvent, fileId: number) => void
}

function FileRow({ file, checked, onCheck, onContext }: FileRowProps) {
  const icon = file.kind === 'Video' ? <IconVideo size={14} /> : <IconAudio size={14} />

  const splitDisp = file.segments.length > 0
    ? file.segments.map(s => <span key={s.id} className="tag tag-seg">{s.label}</span>)
    : <span className="text-muted">─</span>

  const transDisp = file.transcript
    ? <span className="cell-truncate">{file.transcript.substring(0, 30)}{file.transcript.length > 30 ? '…' : ''}</span>
    : <span className="text-muted">─</span>

  return (
    <>
      <tr className="file-row" onContextMenu={e => onContext(e, file.id)}>
        <td className="col-check">
          <input
            type="checkbox"
            checked={checked}
            onChange={e => onCheck(file.id, e.target.checked)}
            aria-label={`选择 ${filename(file.path)}`}
          />
        </td>
        <td className="col-name">
          <span className="file-icon">{icon}</span>
          <span>{filename(file.path)}</span>
        </td>
        <td className="col-size">{formatSize(file.size)}</td>
        <td className="col-meta"><div className="tag-group">{metaTags(file.metadata)}</div></td>
        <td className="col-split">{splitDisp}</td>
        <td className="col-trans">{transDisp}</td>
        <td className="col-rename">
          <span className="cell-truncate">
            {file.rename_preview || <span className="text-muted">─</span>}
          </span>
        </td>
        <td className="col-progress">
          <ProgressBar progress={file.progress} />
        </td>
        <td className="col-status">
          <span className={`status-indicator status-${file.status.toLowerCase()}`}>
            <span className="status-dot" />
            {statusLabel(file.status)}
          </span>
        </td>
      </tr>
      {file.segments.map(seg => (
        <tr key={seg.id} className="sub-row">
          <td className="col-check" />
          <td className="col-name">
            <span className="sub-name">└─ {seg.label}</span>
          </td>
          <td className="col-size"><span className="text-muted">─</span></td>
          <td className="col-meta"><span className="text-muted">─</span></td>
          <td className="col-split">
            <span className="tag tag-seg">{formatTime(seg.start)}–{formatTime(seg.end)}</span>
          </td>
          <td className="col-trans">
            {seg.transcript
              ? <span className="cell-truncate">{seg.transcript.substring(0, 25)}</span>
              : <span className="text-muted">─</span>}
          </td>
          <td className="col-rename"><span className="text-muted">─</span></td>
          <td className="col-progress">
            <ProgressBar progress={seg.progress} />
          </td>
          <td className="col-status">
            <span className={`status-indicator status-${seg.status.toLowerCase()}`}>
              <span className="status-dot" />
              {statusLabel(seg.status)}
            </span>
          </td>
        </tr>
      ))}
    </>
  )
}
