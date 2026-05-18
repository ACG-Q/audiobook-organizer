import type { ProgressInfo } from '../types'

export function ProgressBar({ progress }: { progress?: ProgressInfo }) {
  const pct = progress && progress.total > 0
    ? Math.min(100, Math.round((progress.current / progress.total) * 100))
    : 0

  return (
    <div
      className="progress-bar"
      role="progressbar"
      aria-valuenow={pct}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={`${pct}% complete`}
    >
      <div className="progress-fill" style={{ width: `${pct}%` }} />
      <span className="progress-text">{pct}%</span>
    </div>
  )
}
