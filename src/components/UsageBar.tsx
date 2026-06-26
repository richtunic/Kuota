// src/components/UsageBar.tsx
import { STATUS_HEX, getStatusColor } from '../types'
import { useTimeUntil } from '../hooks/useTimeUntil'

interface Props {
  label: string
  percent: number
  used?: number
  limit?: number
  remaining?: number
  resetsAt?: Date
  hasData: boolean
}

export function UsageBar({ label, percent, used, limit, remaining, resetsAt, hasData }: Props) {
  const color = STATUS_HEX[getStatusColor(percent)]
  const resetLabel = useTimeUntil(resetsAt)

  const rightLabel = (() => {
    if (!hasData) return '—'
    if (used !== undefined && limit !== undefined) return `${used}/${limit}`
    if (remaining !== undefined) return `${remaining} rest.`
    return `${Math.round(percent * 100)}%`
  })()

  return (
    <div className="flex flex-col gap-[3px]">
      <div className="flex items-center gap-2">
        {/* Label fijo */}
        <span className="font-mono text-[9px] font-bold tracking-wide text-[#4B5563] w-[52px] shrink-0">
          {label}
        </span>

        {/* Bar */}
        <div className="relative flex-1 h-[5px] rounded-full bg-[#1A1A22] overflow-hidden">
          {hasData && percent > 0 && (
            <div
              className="absolute left-0 top-0 h-full rounded-full transition-all duration-700"
              style={{
                width: `${Math.min(percent * 100, 100)}%`,
                background: `linear-gradient(to right, ${color}99, ${color})`,
                boxShadow: `0 0 6px ${color}66`,
              }}
            />
          )}
        </div>

        {/* Right value */}
        <span
          className="font-mono text-[9px] w-[56px] text-right shrink-0"
          style={{ color: hasData ? color : '#2D2D3A' }}
        >
          {rightLabel}
        </span>
      </div>

      {/* Reset countdown */}
      {hasData && resetsAt && resetLabel && (
        <div className="flex items-center gap-1 pl-[60px]">
          <svg className="w-2 h-2 text-[#2D2D3A]" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2v1z"/>
            <path d="M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466z"/>
          </svg>
          <span className="font-mono text-[8px] text-[#374151]">
            reset en {resetLabel}
          </span>
        </div>
      )}
    </div>
  )
}
