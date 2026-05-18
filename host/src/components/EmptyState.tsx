import { IconUpload } from './Icons'

interface EmptyStateProps {
  onAddFile: () => void
  onAddFolder: () => void
}

export function EmptyState({ onAddFile, onAddFolder }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-icon">
        <IconUpload size={48} />
      </div>
      <h3 className="empty-title">没有文件</h3>
      <p className="empty-desc">
        拖拽文件到此处，或点击下方按钮添加
      </p>
      <div className="empty-actions">
        <button className="btn btn-primary" onClick={onAddFile}>
          添加文件
        </button>
        <button className="btn" onClick={onAddFolder}>
          添加文件夹
        </button>
      </div>
    </div>
  )
}
