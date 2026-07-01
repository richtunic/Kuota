import { useEffect, useMemo, useState } from 'react'
import { AccountForm } from './components/AccountForm'
import { useStore, type CodexLoginState, type Language } from './store/useStore'
import codexLogo from './assets/codex-logo.png'
import type { AccountUsage, ConfiguredAccount } from './types'

const copy = {
  es: {
    updated: 'Actualizado',
    notSynced: 'Sin sincronizar',
    active: 'Activa',
    ready: 'Lista',
    plan: 'Plan',
    resets: 'Reinicia',
    noUsage: 'Sin datos de uso',
    remaining: 'restante',
    weekly: 'Semanal',
    account: 'Cuenta',
    email: 'Email',
    activity: 'Actividad',
    current: 'Cuenta activa en Codex',
    switching: 'Cambiando...',
    useInCodex: 'Usar en Codex',
    connectAccount: 'Conectar cuenta',
    waitingAuthorization: 'Esperando autorización...',
    waitingConfirmation: 'Esperando confirmación...',
    authorizationCode: 'Código de autorización',
    copyCode: 'Copiar código',
    copiedCode: 'Código copiado',
    openOpenAI: 'Abrir OpenAI',
    retry: 'Reintentar',
    noAccounts: 'Sin cuentas Codex',
    noAccountsHelp: 'Agrega una cuenta con el login guiado de codex-auth.',
    addAccount: 'Agregar cuenta',
    prepare: 'Preparar codex-auth',
    preparing: 'Preparando...',
    checkUpdates: 'Buscar actualizaciones',
    downloadingUpdate: 'Descargando actualización',
    upToDateBro: 'Ya tienes la versión más reciente bro',
    aboutKuota: 'Acerca de Kuota',
    manageAccounts: 'Eliminar cuentas',
    settings: 'Ajustes',
    codexAuth: 'codex-auth',
    checking: 'Revisando...',
    update: 'Actualizar',
    check: 'Revisar',
    edit: 'Editar',
    delete: 'Eliminar',
    quit: 'Cerrar app',
    close: 'Cerrar',
    language: 'Idioma',
    spanish: 'Español',
    english: 'Inglés',
    localOnly: 'Preferencias locales',
    detected: 'Detectada por codex-auth',
    version: 'Versión',
    developedBy: 'Desarrollado por RichTunic en GitHub',
    basedOn: 'Basado en Codex-Auth de terminal',
    codexAuthEngine: 'Motor',
  },
  en: {
    updated: 'Updated',
    notSynced: 'Not synced',
    active: 'Active',
    ready: 'Ready',
    plan: 'Plan',
    resets: 'Resets',
    noUsage: 'No usage data',
    remaining: 'remaining',
    weekly: 'Weekly',
    account: 'Account',
    email: 'Email',
    activity: 'Activity',
    current: 'Current Codex Account',
    switching: 'Switching...',
    useInCodex: 'Use in Codex',
    connectAccount: 'Connect account',
    waitingAuthorization: 'Waiting for authorization...',
    waitingConfirmation: 'Waiting for confirmation...',
    authorizationCode: 'Authorization code',
    copyCode: 'Copy code',
    copiedCode: 'Code copied',
    openOpenAI: 'Open OpenAI',
    retry: 'Retry',
    noAccounts: 'No Codex accounts',
    noAccountsHelp: 'Add one with the guided codex-auth login.',
    addAccount: 'Add Account',
    prepare: 'Prepare codex-auth',
    preparing: 'Preparing...',
    checkUpdates: 'Check for updates',
    downloadingUpdate: 'Downloading update',
    upToDateBro: "You're on the latest version",
    aboutKuota: 'About Kuota',
    manageAccounts: 'Remove accounts',
    settings: 'Settings',
    codexAuth: 'codex-auth',
    checking: 'Checking...',
    update: 'Update',
    check: 'Check',
    edit: 'Edit',
    delete: 'Delete',
    quit: 'Quit app',
    close: 'Close',
    language: 'Language',
    spanish: 'Spanish',
    english: 'English',
    localOnly: 'Local preferences',
    detected: 'Detected by codex-auth',
    version: 'Version',
    developedBy: 'Developed by RichTunic on GitHub',
    basedOn: 'Based on terminal Codex-Auth',
    codexAuthEngine: 'Engine',
  },
} satisfies Record<Language, Record<string, string>>

export default function App() {
  const {
    mergedAccounts,
    activeAccountId,
    codexAuth,
    error,
    codexLogin,
    lastSync,
    switchingAccountId,
    bootstrappingCodexAuth,
    checkingUpdates,
    updateDownloadProgress,
    updatesUpToDate,
    language,
    addAccount,
    deleteAccount,
    setLanguage,
    switchAccount,
    loginAccount,
    cancelLoginAccount,
    openLoginAuthorizationUrl,
    dismissLoginAccount,
    ensureCodexAuth,
    checkAllUpdates,
    quitApp,
    init,
  } = useStore()

  const t = copy[language]
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [showAbout, setShowAbout] = useState(false)
  const bottomAnchored = useMemo(() => !/mac/i.test(navigator.platform), [])

  useEffect(() => {
    init()
  }, [init])

  useEffect(() => {
    if (!mergedAccounts.length) {
      setSelectedId(null)
      return
    }

    const preferredId = activeAccountId ?? mergedAccounts[0].configured.id
    setSelectedId((current) =>
      current && mergedAccounts.some((account) => account.configured.id === current) ? current : preferredId,
    )
  }, [activeAccountId, mergedAccounts])

  const selectedAccount = useMemo(
    () => mergedAccounts.find((account) => account.configured.id === selectedId) ?? mergedAccounts[0],
    [mergedAccounts, selectedId],
  )

  function handleSave(data: Omit<ConfiguredAccount, 'id'> & { id?: string }) {
    addAccount(data)
    setShowForm(false)
  }

  const syncLabel = lastSync ? `${t.updated} ${formatAgo(lastSync, language)}` : t.notSynced
  const version = codexAuth?.version?.replace('codex-auth ', '') ?? '-'
  const latest = codexAuth?.latest_version ?? version
  const updateLabel = checkingUpdates
    ? t.checking
    : codexAuth?.update_available
      ? `${t.update} ${latest}`
      : updatesUpToDate
        ? t.upToDateBro
        : t.checkUpdates

  return (
    <div
      className={`flex h-[520px] w-[360px] select-none flex-col overflow-hidden border-x border-[#242424] bg-[#0B0B0C] text-white shadow-2xl ${
        bottomAnchored ? 'rounded-t-xl border-t' : 'rounded-b-xl border-b'
      }`}
    >
      <header className="border-b border-[#242424] bg-[#151515] px-2 pb-1.5 pt-1.5">
        <AccountTabs
          accounts={mergedAccounts}
          selectedId={selectedAccount?.configured.id ?? null}
          activeId={activeAccountId}
          onSelect={setSelectedId}
          onAdd={loginAccount}
        />
      </header>

      <main className="flex-1 overflow-y-auto overflow-x-hidden px-3 py-2.5">
        {!selectedAccount ? (
          <EmptyState
            t={t}
            installed={Boolean(codexAuth?.installed)}
            loading={bootstrappingCodexAuth}
            onPrepare={ensureCodexAuth}
            onLogin={loginAccount}
          />
        ) : (
          <AccountDetail
            t={t}
            account={selectedAccount}
            isActive={selectedAccount.configured.id === activeAccountId || Boolean(selectedAccount.codexAuth?.active)}
            isSwitching={selectedAccount.configured.id === switchingAccountId}
            syncLabel={syncLabel}
            onSwitch={() => switchAccount(selectedAccount.configured)}
          />
        )}
      </main>

      <footer className="border-t border-[#242424] bg-[#101010] px-3 py-2">
        <ActionRow icon="⚙" label={t.settings} onClick={() => setShowSettings(true)} />
        <ActionRow icon="i" label={t.aboutKuota} onClick={() => setShowAbout(true)} />
        <ActionRow icon="×" label={t.quit} onClick={quitApp} />
        {error && <div className="mt-1 truncate text-[10px] leading-3 text-[#FF6B7A]">{error}</div>}
      </footer>

      {showSettings && (
        <SettingsModal
          t={t}
          accounts={mergedAccounts}
          language={language}
          setLanguage={setLanguage}
          onDelete={deleteAccount}
          updateLabel={updateLabel}
          updateProgress={updateDownloadProgress}
          updateDisabled={checkingUpdates || bootstrappingCodexAuth}
          onCheckUpdates={checkAllUpdates}
          onClose={() => setShowSettings(false)}
        />
      )}

      {showAbout && (
        <AboutModal
          t={t}
          appVersion="1.0.3"
          codexAuthVersion={version}
          onClose={() => setShowAbout(false)}
        />
      )}

      {showForm && (
        <AccountForm
          onSave={handleSave}
          onClose={() => setShowForm(false)}
        />
      )}

      {codexLogin.active && (
        <CodexLoginModal
          t={t}
          code={codexLogin.code}
          url={codexLogin.url}
          status={codexLogin.status}
          message={codexLogin.message}
          onOpen={openLoginAuthorizationUrl}
          onRetry={loginAccount}
          onClose={codexLogin.status === 'succeeded' ? dismissLoginAccount : cancelLoginAccount}
        />
      )}
    </div>
  )
}

function AccountTabs({
  accounts,
  selectedId,
  activeId,
  onSelect,
  onAdd,
}: {
  accounts: AccountUsage[]
  selectedId: string | null
  activeId: string | null
  onSelect: (id: string) => void
  onAdd: () => void
}) {
  return (
    <div className="flex items-stretch gap-1 overflow-x-auto pb-0.5">
      {accounts.map((account) => {
        const id = account.configured.id
        const selected = id === selectedId
        const active = id === activeId || account.codexAuth?.active
        const usage = parseUsage(account.codexAuth?.weekly_usage)

        return (
          <button
            key={id}
            onClick={() => onSelect(id)}
            className={`flex h-[44px] min-w-[64px] max-w-[82px] flex-col items-center justify-center rounded-md px-1.5 transition-colors ${
              selected ? 'bg-[#2F7CF6] text-white' : 'text-[#9B9B9B] hover:bg-[#202020]'
            }`}
            title={account.configured.email}
          >
            <span className="mb-0.5 flex h-4 w-4 items-center justify-center">
              <img
                src={codexLogo}
                alt=""
                className={`h-4 w-4 object-contain ${active ? 'opacity-100' : selected ? 'opacity-90' : 'opacity-55'}`}
                draggable={false}
              />
            </span>
            <span className="w-full truncate text-center text-[10px] leading-3">
              {shortAccountLabel(account)}
            </span>
            <span className="mt-1 h-0.5 w-full overflow-hidden rounded-full bg-white/20">
              <span
                className="block h-full rounded-full"
                style={{ width: `${usage.percent ?? 0}%`, background: active ? '#8AE234' : '#5DDFC6' }}
              />
            </span>
          </button>
        )
      })}

      <button
        onClick={onAdd}
        className="flex h-[44px] min-w-[36px] items-center justify-center rounded-md text-[16px] text-[#9B9B9B] hover:bg-[#202020] hover:text-white"
        title="Add account"
      >
        +
      </button>
    </div>
  )
}

function AccountDetail({
  t,
  account,
  isActive,
  isSwitching,
  syncLabel,
  onSwitch,
}: {
  t: Record<string, string>
  account: AccountUsage
  isActive: boolean
  isSwitching: boolean
  syncLabel: string
  onSwitch: () => void
}) {
  const fiveHour = parseUsage(account.codexAuth?.five_hour_usage)
  const weekly = parseUsage(account.codexAuth?.weekly_usage)

  return (
    <div>
      <div className="border-b border-[#242424] pb-2">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <div className="truncate text-[18px] font-semibold leading-6 text-white">
              {account.configured.name || account.configured.email}
            </div>
            <div className="truncate text-[11px] leading-4 text-[#A8A8A8]">{syncLabel}</div>
          </div>
          <div className="shrink-0 text-right">
            <div className="text-[11px] leading-4 text-[#A8A8A8]">{account.codexAuth?.plan ?? t.plan}</div>
            <div className="mt-1 flex items-center justify-end gap-1 text-[10px] text-[#8AE234]">
              <span className="h-1.5 w-1.5 rounded-full bg-[#8AE234]" />
              {isActive ? t.active : t.ready}
            </div>
          </div>
        </div>
      </div>

      <UsageSection title="5 h" usage={fiveHour} t={t} />
      <UsageSection title={t.weekly} usage={weekly} t={t} />

      <div className="border-b border-[#242424] py-2.5">
        <div className="mb-1 text-[15px] font-semibold leading-5 text-white">{t.account}</div>
        <InfoRow label={t.email} value={account.configured.email || '-'} />
        <InfoRow label={t.activity} value={account.codexAuth?.last_activity ?? '-'} />
      </div>

      <div className="border-b border-[#242424] py-2.5">
        <button
          onClick={onSwitch}
          disabled={isSwitching || isActive}
          className="h-8 w-full rounded-md bg-[#E8E8E8] text-[12px] font-medium text-[#111111] transition-colors hover:bg-white disabled:bg-[#303030] disabled:text-[#777777]"
        >
          {isActive ? t.current : isSwitching ? t.switching : t.useInCodex}
        </button>
      </div>
    </div>
  )
}

function UsageSection({ title, usage, t }: { title: string; usage: ParsedUsage; t: Record<string, string> }) {
  return (
    <section className="border-b border-[#242424] py-2.5">
      <div className="mb-1.5 flex items-baseline justify-between">
        <h2 className="text-[15px] font-semibold leading-5 text-white">{title}</h2>
        {usage.reset && <span className="text-[11px] leading-3 text-[#A8A8A8]">{t.resets} {usage.reset}</span>}
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-[#2B2B2B]">
        <div className="h-full rounded-full bg-[#D3865B]" style={{ width: `${usage.percent ?? 0}%` }} />
      </div>
      <div className="mt-1.5 text-[13px] leading-4 text-white">
        {usage.percent === undefined ? t.noUsage : `${usage.percent}% ${t.remaining}`}
      </div>
      {usage.detail && <div className="mt-0.5 text-[11px] leading-3 text-[#8F8F8F]">{usage.detail}</div>}
    </section>
  )
}

function SettingsModal({
  t,
  accounts,
  language,
  setLanguage,
  onDelete,
  updateLabel,
  updateProgress,
  updateDisabled,
  onCheckUpdates,
  onClose,
}: {
  t: Record<string, string>
  accounts: AccountUsage[]
  language: Language
  setLanguage: (language: Language) => void
  onDelete: (id: string) => void
  updateLabel: string
  updateProgress: number | null
  updateDisabled: boolean
  onCheckUpdates: () => void
  onClose: () => void
}) {
  return (
    <Modal title={t.settings} onClose={onClose}>
      <div className="space-y-3">
        <section>
          <div className="mb-1.5 text-[10px] leading-3 text-[#8F8F8F]">{t.language}</div>
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={() => setLanguage('es')}
              className={`h-8 rounded-md border text-[12px] ${
                language === 'es' ? 'border-[#E8E8E8] bg-[#E8E8E8] text-[#111111]' : 'border-[#303030] bg-[#1D1D1D] text-[#D6D6D6]'
              }`}
            >
              {t.spanish}
            </button>
            <button
              onClick={() => setLanguage('en')}
              className={`h-8 rounded-md border text-[12px] ${
                language === 'en' ? 'border-[#E8E8E8] bg-[#E8E8E8] text-[#111111]' : 'border-[#303030] bg-[#1D1D1D] text-[#D6D6D6]'
              }`}
            >
              {t.english}
            </button>
          </div>
        </section>

        <section className="border-t border-[#242424] pt-2">
          <button
            onClick={onCheckUpdates}
            disabled={updateDisabled}
            className="h-8 w-full rounded-md border border-[#303030] bg-[#1D1D1D] text-[12px] text-[#D6D6D6] hover:bg-[#242424] disabled:text-[#777777]"
          >
            {updateLabel}
          </button>
          {updateProgress !== null && (
            <div className="mt-2">
              <div className="mb-1 flex items-center justify-between text-[10px] leading-3 text-[#8F8F8F]">
                <span>{t.downloadingUpdate}</span>
                <span>{Math.round(updateProgress)}%</span>
              </div>
              <div className="h-1.5 overflow-hidden rounded-full bg-[#2B2B2B]">
                <div className="h-full rounded-full bg-[#8AE234]" style={{ width: `${updateProgress}%` }} />
              </div>
            </div>
          )}
        </section>

        <section className="border-t border-[#242424] pt-2">
          <div className="mb-1.5 text-[10px] leading-3 text-[#8F8F8F]">{t.manageAccounts}</div>
          <div className="max-h-[168px] overflow-y-auto">
            {accounts.map((account) => (
              <div key={account.configured.id} className="flex items-center gap-2 border-b border-[#242424] py-2 last:border-b-0">
                <img src={codexLogo} alt="" className="h-5 w-5 opacity-80" draggable={false} />
                <div className="min-w-0 flex-1">
                  <div className="truncate text-[12px] text-white">{account.configured.name || account.configured.email}</div>
                  <div className="truncate text-[10px] text-[#8F8F8F]">
                    {account.configured.id.startsWith('codex-auth-') ? t.detected : t.localOnly}
                  </div>
                </div>
                <button
                  className="rounded px-2 py-1 text-[11px] text-[#FF6B7A] hover:bg-[#242424]"
                  onClick={() => onDelete(account.configured.id)}
                >
                  {t.delete}
                </button>
              </div>
            ))}
          </div>
        </section>

      </div>
    </Modal>
  )
}

function AboutModal({
  t,
  appVersion,
  codexAuthVersion,
  onClose,
}: {
  t: Record<string, string>
  appVersion: string
  codexAuthVersion: string
  onClose: () => void
}) {
  return (
    <Modal title={t.aboutKuota} onClose={onClose}>
      <div className="space-y-2">
        <InfoRow label={t.version} value={appVersion} />
        <InfoRow label={t.codexAuthEngine} value={`codex-auth ${codexAuthVersion}`} />
        <div className="border-t border-[#242424] pt-2 text-[12px] leading-4 text-[#D6D6D6]">
          {t.developedBy}
        </div>
        <div className="text-[11px] leading-4 text-[#8F8F8F]">{t.basedOn}</div>
      </div>
    </Modal>
  )
}

function CodexLoginModal({
  t,
  code,
  url,
  status,
  message,
  onOpen,
  onRetry,
  onClose,
}: {
  t: Record<string, string>
  code: string | null
  url: string | null
  status: CodexLoginState['status']
  message: string | null
  onOpen: () => void
  onRetry: () => void
  onClose: () => void
}) {
  const [copied, setCopied] = useState(false)
  const waiting = status === 'starting' || status === 'waiting' || status === 'browser-opened'
  const succeeded = status === 'succeeded'
  const failed = status === 'failed'

  async function copyCode() {
    if (!code) return
    await navigator.clipboard.writeText(code)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1800)
  }

  return (
    <Modal title={t.connectAccount} onClose={onClose}>
      <div className="space-y-3">
        <div className="flex items-center gap-2 rounded-md border border-[#303030] bg-[#101010] px-3 py-2">
          {waiting && <span className="h-3 w-3 animate-spin rounded-full border border-[#5DDFC6] border-t-transparent" />}
          {succeeded && <span className="h-2.5 w-2.5 rounded-full bg-[#8AE234]" />}
          {failed && <span className="h-2.5 w-2.5 rounded-full bg-[#FF6B7A]" />}
          <span className="text-[12px] leading-4 text-[#E8E8E8]">
            {message ?? (waiting ? t.waitingAuthorization : t.waitingConfirmation)}
          </span>
        </div>

        <section>
          <div className="mb-1 text-[10px] leading-3 text-[#8F8F8F]">{t.authorizationCode}</div>
          <div className="flex items-center gap-2">
            <div className="flex h-10 min-w-0 flex-1 items-center rounded-md border border-[#303030] bg-[#0B0B0C] px-3 font-mono text-[17px] tracking-[0.08em] text-white">
              <span className="truncate">{code ?? '--------'}</span>
            </div>
            <button
              onClick={copyCode}
              disabled={!code}
              className="h-10 rounded-md bg-[#E8E8E8] px-3 text-[12px] font-medium text-[#111111] hover:bg-white disabled:bg-[#303030] disabled:text-[#777777]"
            >
              {copied ? t.copiedCode : t.copyCode}
            </button>
          </div>
        </section>

        <button
          onClick={onOpen}
          disabled={!url}
          className="h-9 w-full rounded-md border border-[#303030] bg-[#1D1D1D] text-[12px] text-[#D6D6D6] hover:bg-[#242424] disabled:text-[#777777]"
        >
          {t.openOpenAI}
        </button>

        {waiting && (
          <div className="flex items-center gap-2 text-[11px] leading-4 text-[#8F8F8F]">
            <span className="h-2 w-2 animate-pulse rounded-full bg-[#5DDFC6]" />
            {t.waitingConfirmation}
          </div>
        )}

        {failed && (
          <button
            onClick={onRetry}
            className="h-9 w-full rounded-md bg-[#E8E8E8] text-[12px] font-medium text-[#111111] hover:bg-white"
          >
            {t.retry}
          </button>
        )}
      </div>
    </Modal>
  )
}

function Modal({ title, children, onClose }: { title: string; children: React.ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-[328px] overflow-hidden rounded-lg border border-[#242424] bg-[#151515] shadow-2xl">
        <div className="flex items-center justify-between border-b border-[#242424] px-3 py-2">
          <div className="text-[13px] leading-5 text-white">{title}</div>
          <button onClick={onClose} className="rounded px-2 py-1 text-[11px] text-[#A8A8A8] hover:bg-[#242424]">
            ×
          </button>
        </div>
        <div className="p-3">{children}</div>
      </div>
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 py-0.5">
      <span className="text-[11px] text-[#8F8F8F]">{label}</span>
      <span className="truncate text-right text-[11px] text-[#D6D6D6]">{value}</span>
    </div>
  )
}

function ActionRow({
  icon,
  label,
  onClick,
  disabled = false,
}: {
  icon: string
  label: string
  onClick: () => void
  disabled?: boolean
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="flex w-full items-center gap-2 rounded-md py-1 text-left text-[13px] leading-4 text-[#E8E8E8] hover:bg-[#1D1D1D] disabled:text-[#777777]"
    >
      <span className="flex w-4 justify-center text-[12px] text-[#A8A8A8]">{icon}</span>
      <span>{label}</span>
    </button>
  )
}

function EmptyState({
  t,
  installed,
  loading,
  onPrepare,
  onLogin,
}: {
  t: Record<string, string>
  installed: boolean
  loading: boolean
  onPrepare: () => void
  onLogin: () => void
}) {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="w-full rounded-lg border border-[#242424] bg-[#151515] px-4 py-5 text-center">
        <div className="text-[15px] leading-5 text-white">{t.noAccounts}</div>
        <div className="mt-1 text-[12px] leading-4 text-[#9B9B9B]">{t.noAccountsHelp}</div>
        <button
          onClick={installed ? onLogin : onPrepare}
          disabled={loading}
          className="mt-4 h-8 rounded-md bg-[#E8E8E8] px-3 text-[12px] text-[#111111] transition-colors hover:bg-white disabled:bg-[#303030] disabled:text-[#777777]"
        >
          {loading ? t.preparing : installed ? t.addAccount : t.prepare}
        </button>
      </div>
    </div>
  )
}

type ParsedUsage = {
  percent?: number
  reset?: string
  detail?: string
}

function parseUsage(value?: string): ParsedUsage {
  if (!value) return {}
  const percentMatch = value.match(/(\d+)%/)
  const resetMatch = value.match(/\(([^)]+)\)/)
  const percent = percentMatch ? Number(percentMatch[1]) : undefined
  const reset = resetMatch?.[1]
  return { percent, reset, detail: value }
}

function shortAccountLabel(account: AccountUsage): string {
  const email = account.configured.email
  if (!email) return account.configured.name || 'Codex'
  const [name] = email.split('@')
  return name.length > 9 ? `${name.slice(0, 8)}...` : name
}

function formatAgo(date: Date, language: Language): string {
  const diff = Math.floor((Date.now() - date.getTime()) / 1000)
  if (diff < 60) return language === 'es' ? 'ahora' : 'just now'
  if (diff < 3600) return language === 'es' ? `hace ${Math.floor(diff / 60)}m` : `${Math.floor(diff / 60)}m ago`
  return language === 'es' ? `hace ${Math.floor(diff / 3600)}h` : `${Math.floor(diff / 3600)}h ago`
}
