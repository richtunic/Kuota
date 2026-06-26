import { useState } from 'react'
import type { AccountUsage } from '../types'

interface Props {
  account: AccountUsage
  isActive: boolean
  isSwitching: boolean
  onActivate: () => void
  onEdit: () => void
  onDelete: () => void
}

export function AccountCard({ account, isActive, isSwitching, onActivate, onEdit, onDelete }: Props) {
  const [expanded, setExpanded] = useState(false)
  const [showMenu, setShowMenu] = useState(false)
  const { configured: cfg, codexAuth } = account

  return (
    <section className="overflow-hidden rounded-lg border border-[#242424] bg-[#151515]">
      <div className="flex items-start gap-3 p-3">
        <button
          onClick={onActivate}
          disabled={isSwitching}
          className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-[#303030] bg-[#1D1D1D] text-[12px] font-medium text-white disabled:opacity-60"
          style={{ color: cfg.color }}
          title={isActive ? 'Cuenta activa' : 'Cambiar a esta cuenta'}
        >
          {isSwitching ? '...' : cfg.name[0]?.toUpperCase() ?? 'C'}
        </button>

        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 items-center gap-2">
            <div className="truncate text-[13px] leading-5 text-white">{cfg.name}</div>
            {(isActive || codexAuth?.active) && <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-[#8AE234]" />}
            {!codexAuth && (
              <span className="shrink-0 rounded bg-[#242424] px-1.5 py-0.5 text-[9px] leading-none text-[#8F8F8F]">
                local
              </span>
            )}
          </div>
          <div className="truncate text-[11px] leading-4 text-[#8F8F8F]">
            {cfg.email || 'Cuenta Codex'} · {codexAuth?.plan ?? 'sin plan'}
          </div>
        </div>

        <div className="relative">
          <button
            onClick={() => setShowMenu((value) => !value)}
            className="flex h-7 w-7 items-center justify-center rounded-md text-[#A8A8A8] transition-colors hover:bg-[#242424] hover:text-white"
            aria-label="Opciones de cuenta"
          >
            <svg className="h-3.5 w-3.5" viewBox="0 0 16 16" fill="currentColor">
              <circle cx="8" cy="3.2" r="1.2" />
              <circle cx="8" cy="8" r="1.2" />
              <circle cx="8" cy="12.8" r="1.2" />
            </svg>
          </button>

          {showMenu && (
            <div className="absolute right-0 top-8 z-20 w-32 overflow-hidden rounded-md border border-[#303030] bg-[#1A1A1A] shadow-xl">
              <MenuButton
                onClick={() => {
                  setShowMenu(false)
                  onEdit()
                }}
              >
                Editar
              </MenuButton>
              {!cfg.id.startsWith('codex-auth-') && (
                <MenuButton
                  danger
                  onClick={() => {
                    setShowMenu(false)
                    onDelete()
                  }}
                >
                  Eliminar
                </MenuButton>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="px-3 pb-3">
        <CodexAuthUsage account={account} />

        <button
          onClick={onActivate}
          disabled={isSwitching}
          className="mt-2 h-8 w-full rounded-md border border-[#303030] bg-[#1D1D1D] text-[12px] text-[#E8E8E8] transition-colors hover:bg-[#242424] disabled:text-[#777777]"
        >
          {isActive || codexAuth?.active ? 'Cuenta activa' : isSwitching ? 'Cambiando...' : 'Usar en Codex'}
        </button>

        <button
          onClick={() => setExpanded((value) => !value)}
          className="mt-2 flex w-full items-center justify-center gap-1 text-[10px] leading-3 text-[#8F8F8F] hover:text-[#D6D6D6]"
        >
          {expanded ? 'Ocultar detalle' : 'Ver detalle'}
          <svg className="h-3 w-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.8">
            <path d={expanded ? 'M4 10l4-4 4 4' : 'M4 6l4 4 4-4'} />
          </svg>
        </button>

        {expanded && (
          <div className="mt-2 flex flex-wrap gap-1.5">
            {codexAuth?.last_activity && <Chip label={`actividad ${codexAuth.last_activity}`} />}
            {cfg.notes && <Chip label={cfg.notes} />}
            {codexAuth?.raw && <Chip label={codexAuth.raw} />}
          </div>
        )}
      </div>
    </section>
  )
}

function CodexAuthUsage({ account }: { account: AccountUsage }) {
  const auth = account.codexAuth

  return (
    <div className="rounded-md border border-[#303030] bg-[#1D1D1D] px-3 py-2">
      <div className="mb-2 flex items-center gap-2">
        <span className="text-[#A8A8A8]">
          <svg className="h-3.5 w-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.7">
            <path d="M2.5 8a5.5 5.5 0 0 1 9.4-3.9" />
            <path d="M13.5 8a5.5 5.5 0 0 1-9.4 3.9" />
            <path d="M12 1.8v3.1H8.9" />
            <path d="M4 14.2v-3.1h3.1" />
          </svg>
        </span>
        <span className="text-[15px] leading-none text-white">Uso codex-auth</span>
      </div>

      <UsageRow label="5 h" value={auth?.five_hour_usage ?? '-'} />
      <UsageRow label="Semanal" value={auth?.weekly_usage ?? '-'} />
    </div>
  )
}

function UsageRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[1fr_auto] items-baseline gap-3 py-[3px]">
      <span className="text-[15px] leading-5 text-white">{label}</span>
      <span className="text-right text-[15px] leading-5 text-[#A8A8A8] tabular-nums">{value}</span>
    </div>
  )
}

function MenuButton({
  onClick,
  danger,
  children,
}: {
  onClick: () => void
  danger?: boolean
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full px-3 py-1.5 text-left text-[12px] transition-colors hover:bg-[#242424] ${
        danger ? 'text-[#FF6B7A]' : 'text-[#E8E8E8]'
      }`}
    >
      {children}
    </button>
  )
}

function Chip({ label }: { label: string }) {
  return <span className="max-w-full truncate rounded bg-[#242424] px-2 py-1 text-[10px] leading-none text-[#A8A8A8]">{label}</span>
}
