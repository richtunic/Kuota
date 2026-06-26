// src/types/index.ts

export interface WeeklyUsage {
  limit_requests?: number
  remaining_requests?: number
  resets_at?: string
  messages_sent?: number
}

export interface HourlyUsage {
  limit_tokens?: number
  remaining_tokens?: number
  resets_at?: string
  messages_sent_5h?: number
}

export interface CapturedAccount {
  id: string
  email?: string
  name?: string
  first_seen?: string
  last_request?: string
  weekly?: WeeklyUsage
  hourly?: HourlyUsage
  models?: Record<string, number>
}

export interface UsageState {
  accounts: Record<string, CapturedAccount>
  last_updated?: string
}

export interface CodexAuthAccount {
  email: string
  active: boolean
  plan?: string
  five_hour_usage?: string
  weekly_usage?: string
  last_activity?: string
  raw: string
}

export interface CodexAuthStatus {
  installed: boolean
  path?: string
  version?: string
  latest_version?: string
  update_available: boolean
  status?: string
  accounts: CodexAuthAccount[]
  error?: string
}

// Cuenta configurada manualmente por el usuario
export interface ConfiguredAccount {
  id: string
  name: string
  email: string
  color: string
  browser: string
  profile?: string   // e.g. "Profile 1", "Profile 2"
  notes: string
}

// Merge de cuenta configurada + datos capturados
export interface AccountUsage {
  configured: ConfiguredAccount
  captured?: CapturedAccount
  codexAuth?: CodexAuthAccount

  // Computed
  weeklyPercent: number
  weeklyUsed?: number
  weeklyLimit?: number
  weeklyRemaining?: number
  weeklyResetsAt?: Date

  hourlyPercent: number
  hourlyUsed?: number
  hourlyLimit?: number
  hourlyRemaining?: number
  hourlyResetsAt?: Date

  hasRealData: boolean
  topModel?: string
  lastSeen?: Date
}

export type StatusColor = 'green' | 'amber' | 'red' | 'gray'

export function getStatusColor(percent: number): StatusColor {
  if (percent < 0.6) return 'green'
  if (percent < 0.85) return 'amber'
  return 'red'
}

export const STATUS_HEX: Record<StatusColor, string> = {
  green: '#00FF9C',
  amber: '#FFB800',
  red:   '#FF3B5C',
  gray:  '#374151',
}

export const BROWSERS = [
  { label: 'Brave - Default',   browser: 'Brave', profile: undefined },
  { label: 'Brave - Perfil 1',  browser: 'Brave', profile: 'Profile 1' },
  { label: 'Brave - Perfil 2',  browser: 'Brave', profile: 'Profile 2' },
  { label: 'Brave - Perfil 3',  browser: 'Brave', profile: 'Profile 3' },
  { label: 'Chrome - Default',  browser: 'Chrome', profile: undefined },
  { label: 'Chrome - Perfil 1', browser: 'Chrome', profile: 'Profile 1' },
  { label: 'Chrome - Perfil 2', browser: 'Chrome', profile: 'Profile 2' },
  { label: 'Safari',            browser: 'Safari', profile: undefined },
  { label: 'Firefox',           browser: 'Firefox', profile: undefined },
  { label: 'Arc',               browser: 'Arc', profile: undefined },
]

export const ACCENT_COLORS = [
  '#7C3AED', '#0EA5E9', '#F59E0B', '#10B981',
  '#EF4444', '#EC4899', '#F97316', '#06B6D4',
]
