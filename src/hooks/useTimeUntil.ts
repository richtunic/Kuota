// src/hooks/useTimeUntil.ts
import { useState, useEffect } from 'react'

export function timeUntilShort(date: Date): string {
  const diff = date.getTime() - Date.now()
  if (diff <= 0) return 'ya'
  const hours = Math.floor(diff / 3_600_000)
  const mins = Math.floor((diff % 3_600_000) / 60_000)
  if (hours > 24) return `${Math.floor(hours / 24)}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

export function useTimeUntil(date?: Date) {
  const [label, setLabel] = useState(date ? timeUntilShort(date) : '')

  useEffect(() => {
    if (!date) return
    setLabel(timeUntilShort(date))
    const id = setInterval(() => setLabel(timeUntilShort(date)), 30_000)
    return () => clearInterval(id)
  }, [date])

  return label
}
