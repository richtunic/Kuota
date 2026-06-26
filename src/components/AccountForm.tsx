import { useState } from 'react'
import { ACCENT_COLORS } from '../types'
import type { ConfiguredAccount } from '../types'

interface Props {
  account?: ConfiguredAccount
  onSave: (account: Omit<ConfiguredAccount, 'id'> & { id?: string }) => void
  onClose: () => void
}

export function AccountForm({ account, onSave, onClose }: Props) {
  const [name, setName] = useState(account?.name ?? '')
  const [email, setEmail] = useState(account?.email ?? '')
  const [color, setColor] = useState(account?.color ?? '#A3A3A3')
  const [notes, setNotes] = useState(account?.notes ?? '')

  const valid = Boolean(name.trim() && email.trim())

  function handleSave() {
    onSave({
      id: account?.id,
      name: name.trim(),
      email: email.trim(),
      color,
      browser: 'Codex',
      profile: undefined,
      notes: notes.trim(),
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-[340px] overflow-hidden rounded-lg border border-[#242424] bg-[#151515] shadow-2xl">
        <div className="flex items-center justify-between border-b border-[#242424] px-4 py-3">
          <div>
            <div className="text-[14px] leading-5 text-white">{account ? 'Editar cuenta Codex' : 'Nueva cuenta Codex'}</div>
            <div className="text-[10px] leading-4 text-[#8F8F8F]">Debe coincidir con codex-auth list.</div>
          </div>
          <button
            onClick={onClose}
            className="flex h-7 w-7 items-center justify-center rounded-md text-[#A8A8A8] transition-colors hover:bg-[#242424]"
            aria-label="Cerrar"
          >
            <svg className="h-3.5 w-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>

        <div className="space-y-3 px-4 py-4">
          <Field label="Nombre visible">
            <Input value={name} onChange={setName} placeholder="Trabajo, personal, cliente..." />
          </Field>

          <Field label="Email o alias de codex-auth">
            <Input value={email} onChange={setEmail} placeholder="cuenta@dominio.com" />
          </Field>

          <Field label="Color">
            <div className="flex gap-2">
              {ACCENT_COLORS.map((option) => (
                <button
                  key={option}
                  onClick={() => setColor(option)}
                  className="h-6 w-6 rounded-full border transition-transform"
                  style={{
                    background: option,
                    borderColor: color === option ? '#FFFFFF' : '#303030',
                    transform: color === option ? 'scale(1.08)' : 'scale(1)',
                  }}
                  aria-label={`Usar color ${option}`}
                />
              ))}
            </div>
          </Field>

          <Field label="Notas">
            <Input value={notes} onChange={setNotes} placeholder="Uso, proyecto o limite a vigilar" />
          </Field>
        </div>

        <div className="flex gap-2 px-4 pb-4">
          <button
            onClick={onClose}
            className="h-9 flex-1 rounded-md border border-[#303030] bg-[#1D1D1D] text-[12px] text-[#D6D6D6] transition-colors hover:bg-[#242424]"
          >
            Cancelar
          </button>
          <button
            onClick={handleSave}
            disabled={!valid}
            className="h-9 flex-1 rounded-md bg-[#E8E8E8] text-[12px] text-[#111111] transition-colors hover:enabled:bg-white disabled:bg-[#303030] disabled:text-[#777777]"
          >
            {account ? 'Guardar' : 'Agregar'}
          </button>
        </div>
      </div>
    </div>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-[10px] leading-3 text-[#8F8F8F]">{label}</span>
      {children}
    </label>
  )
}

function Input({
  value,
  onChange,
  placeholder,
}: {
  value: string
  onChange: (value: string) => void
  placeholder: string
}) {
  return (
    <input
      value={value}
      onChange={(event) => onChange(event.target.value)}
      placeholder={placeholder}
      className="h-9 w-full rounded-md border border-[#303030] bg-[#1D1D1D] px-3 text-[12px] text-white outline-none placeholder:text-[#6F6F6F] focus:border-[#5A5A5A]"
    />
  )
}
