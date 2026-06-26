import { invoke } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { AccountUsage, CodexAuthAccount, CodexAuthStatus, ConfiguredAccount } from '../types'

export type Language = 'es' | 'en'

interface AppState {
  accounts: ConfiguredAccount[]
  activeAccountId: string | null
  codexAuth: CodexAuthStatus | null
  mergedAccounts: AccountUsage[]
  hiddenAccountKeys: string[]
  error: string | null
  lastSync: Date | null
  switchingAccountId: string | null
  bootstrappingCodexAuth: boolean
  checkingUpdates: boolean
  language: Language

  addAccount: (account: Omit<ConfiguredAccount, 'id'>) => void
  updateAccount: (account: ConfiguredAccount) => void
  deleteAccount: (id: string) => void
  setLanguage: (language: Language) => void
  switchAccount: (account: ConfiguredAccount) => Promise<void>
  loginAccount: () => Promise<void>
  refreshCodexAuth: () => Promise<void>
  ensureCodexAuth: () => Promise<void>
  checkAllUpdates: () => Promise<void>
  openCodex: () => Promise<void>
  quitApp: () => Promise<void>
  init: () => void
}

let initialized = false

function mergeAccounts(
  configured: ConfiguredAccount[],
  codexAuth: CodexAuthStatus | null,
  hiddenAccountKeys: string[] = [],
): AccountUsage[] {
  const codexAccounts = codexAuth?.accounts ?? []
  const hidden = new Set(hiddenAccountKeys)
  const visibleCodexAccounts = codexAccounts.filter((account) => !hidden.has(normalizeKey(account.email)))
  const visibleConfigured = configured.filter((account) => !hidden.has(normalizeKey(account.email) || account.id))
  const configuredWithAuth = mergeConfiguredWithCodexAuth(visibleConfigured, visibleCodexAccounts)

  return configuredWithAuth.map((account) => {
    const codexAccount = visibleCodexAccounts.find((item) => normalizeKey(item.email) === normalizeKey(account.email))
    return buildAccountUsage(account, codexAccount)
  })
}

function mergeConfiguredWithCodexAuth(
  configured: ConfiguredAccount[],
  codexAccounts: CodexAuthAccount[],
): ConfiguredAccount[] {
  const byEmail = new Map<string, ConfiguredAccount>()

  for (const account of configured) {
    byEmail.set(normalizeKey(account.email) || account.id, account)
  }

  for (const account of codexAccounts) {
    const key = normalizeKey(account.email)
    if (!key || byEmail.has(key)) continue

    byEmail.set(key, {
      id: `codex-auth-${key}`,
      name: account.email,
      email: account.email,
      color: account.active ? '#8AE234' : '#A3A3A3',
      browser: 'Codex',
      profile: undefined,
      notes: account.active ? 'Activa en codex-auth' : 'codex-auth',
    })
  }

  return [...byEmail.values()]
}

function buildAccountUsage(configured: ConfiguredAccount, codexAuth?: CodexAuthAccount): AccountUsage {
  return {
    configured,
    codexAuth,
    weeklyPercent: 0,
    hourlyPercent: 0,
    hasRealData: Boolean(codexAuth),
  }
}

function normalizeKey(value?: string): string {
  return value?.trim().toLowerCase() ?? ''
}

function humanizeUpdateError(error: unknown): string {
  const message = String(error)
  if (/updater|endpoint|signature|pubkey|public key|latest\.json/i.test(message)) {
    return 'Actualizador de Kuota pendiente de configurar con GitHub Releases y firma.'
  }
  return message
}

export const useStore = create<AppState>()(
  persist(
    (set, get) => ({
    accounts: [],
    activeAccountId: null,
    codexAuth: null,
    mergedAccounts: [],
    hiddenAccountKeys: [],
      error: null,
    lastSync: null,
    switchingAccountId: null,
    bootstrappingCodexAuth: false,
    checkingUpdates: false,
    language: 'es',

      addAccount: (account) => {
      const nextAccount = { ...account, id: crypto.randomUUID() }
      set((state) => {
        const accounts = dedupeConfiguredAccounts([...state.accounts, nextAccount])
        const hiddenAccountKeys = state.hiddenAccountKeys.filter((key) => key !== normalizeKey(nextAccount.email))
        return {
          accounts,
          activeAccountId: state.activeAccountId ?? nextAccount.id,
          hiddenAccountKeys,
          mergedAccounts: mergeAccounts(accounts, state.codexAuth, hiddenAccountKeys),
        }
      })
    },

      updateAccount: (account) => {
        set((state) => {
          const exists = state.accounts.some((item) => item.id === account.id)
          const accounts = dedupeConfiguredAccounts(
            exists
              ? state.accounts.map((item) => (item.id === account.id ? account : item))
              : [...state.accounts, account],
          )
        return { accounts, mergedAccounts: mergeAccounts(accounts, state.codexAuth, state.hiddenAccountKeys) }
      })
    },

    deleteAccount: (id) => {
      set((state) => {
        const target = state.mergedAccounts.find((account) => account.configured.id === id)?.configured
        const hiddenKey = normalizeKey(target?.email) || id
        const hiddenAccountKeys = hiddenKey
          ? Array.from(new Set([...state.hiddenAccountKeys, hiddenKey]))
          : state.hiddenAccountKeys
        const accounts = state.accounts.filter((account) => account.id !== id)
        const mergedAccounts = mergeAccounts(accounts, state.codexAuth, hiddenAccountKeys)

        return {
          accounts,
          hiddenAccountKeys,
          activeAccountId: state.activeAccountId === id ? mergedAccounts[0]?.configured.id ?? null : state.activeAccountId,
          mergedAccounts,
        }
      })
    },

      setLanguage: (language) => set({ language }),

      switchAccount: async (account) => {
        set({ switchingAccountId: account.id, error: null })
        try {
          const status = await invoke<CodexAuthStatus>('switch_codex_account', {
            selector: account.email || account.name,
          })
          set((state) => ({
          codexAuth: status,
          activeAccountId: activeIdFromStatus(status, state.accounts, state.hiddenAccountKeys) ?? account.id,
          lastSync: new Date(),
          mergedAccounts: mergeAccounts(state.accounts, status, state.hiddenAccountKeys),
        }))
        } catch (error) {
          set({ error: String(error) })
        } finally {
          set({ switchingAccountId: null })
        }
      },

      loginAccount: async () => {
        set({ error: null })
        try {
          await invoke('codex_auth_login', { deviceAuth: true })
          scheduleLoginRefreshes(get().refreshCodexAuth)
        } catch (error) {
          set({ error: String(error) })
        }
      },

      refreshCodexAuth: async () => {
        try {
          const status = await invoke<CodexAuthStatus>('codex_auth_status')
          set((state) => ({
          codexAuth: status,
          activeAccountId: activeIdFromStatus(status, state.accounts, state.hiddenAccountKeys) ?? state.activeAccountId,
          lastSync: new Date(),
          mergedAccounts: mergeAccounts(state.accounts, status, state.hiddenAccountKeys),
        }))
        } catch (error) {
          set({ error: String(error) })
        }
      },

    ensureCodexAuth: async () => {
      set({ bootstrappingCodexAuth: true, error: null })
      try {
          const status = await invoke<CodexAuthStatus>('ensure_codex_auth')
          set((state) => ({
          codexAuth: status,
          activeAccountId: activeIdFromStatus(status, state.accounts, state.hiddenAccountKeys) ?? state.activeAccountId,
          lastSync: new Date(),
          mergedAccounts: mergeAccounts(state.accounts, status, state.hiddenAccountKeys),
        }))
        } catch (error) {
          set({ error: String(error) })
        } finally {
        set({ bootstrappingCodexAuth: false })
      }
    },

    checkAllUpdates: async () => {
      set({ checkingUpdates: true, bootstrappingCodexAuth: true, error: null })

      try {
        const status = await invoke<CodexAuthStatus>('ensure_codex_auth')
        set((state) => ({
          codexAuth: status,
          activeAccountId: activeIdFromStatus(status, state.accounts, state.hiddenAccountKeys) ?? state.activeAccountId,
          lastSync: new Date(),
          mergedAccounts: mergeAccounts(state.accounts, status, state.hiddenAccountKeys),
        }))

        const update = await check()
        if (update) {
          await update.downloadAndInstall()
          await relaunch()
        }
      } catch (error) {
        set({ error: humanizeUpdateError(error) })
      } finally {
        set({ checkingUpdates: false, bootstrappingCodexAuth: false })
      }
    },

    openCodex: async () => {
      try {
        await invoke('open_codex')
      } catch (error) {
        set({ error: String(error) })
      }
    },

    quitApp: async () => {
      try {
        await invoke('quit_app')
      } catch (error) {
        set({ error: String(error) })
      }
    },

    init: () => {
        get().ensureCodexAuth()
        if (initialized) return
        initialized = true

        window.setInterval(() => {
          get().refreshCodexAuth()
        }, 60_000)

        window.addEventListener('focus', () => {
          get().refreshCodexAuth()
        })

        document.addEventListener('visibilitychange', () => {
          if (!document.hidden) get().refreshCodexAuth()
        })
      },
    }),
    {
      name: 'gpt-router-store',
      version: 5,
      migrate: (persistedState: unknown) => {
        const state = persistedState as Partial<AppState> | undefined
        if (!state?.accounts) return persistedState

        const accounts = dedupeConfiguredAccounts(removeDemoAccounts(state.accounts))
        return {
          ...state,
          accounts,
          activeAccountId: accounts.some((account) => account.id === state.activeAccountId)
            ? state.activeAccountId
            : accounts[0]?.id ?? null,
          language: state.language ?? 'es',
          hiddenAccountKeys: state.hiddenAccountKeys ?? [],
        }
      },
      partialize: (state) => ({
        accounts: state.accounts,
        activeAccountId: state.activeAccountId,
        hiddenAccountKeys: state.hiddenAccountKeys,
        language: state.language,
      }),
    },
  ),
)

function activeIdFromStatus(
  status: CodexAuthStatus,
  accounts: ConfiguredAccount[],
  hiddenAccountKeys: string[] = [],
): string | null {
  const activeEmail = status.accounts.find((account) => account.active)?.email
  const key = normalizeKey(activeEmail)
  if (!key || hiddenAccountKeys.includes(key)) return null
  return accounts.find((account) => normalizeKey(account.email) === key)?.id ?? `codex-auth-${key}`
}

function removeDemoAccounts(accounts: ConfiguredAccount[]): ConfiguredAccount[] {
  const demoEmails = new Set(['hector@takax.mx', 'forge@antigravity.dev', 'ennex@dev.mx'])
  return accounts.filter((account) => !demoEmails.has(normalizeKey(account.email)))
}

function dedupeConfiguredAccounts(accounts: ConfiguredAccount[]): ConfiguredAccount[] {
  const byKey = new Map<string, ConfiguredAccount>()

  for (const account of accounts) {
    const key = normalizeKey(account.email) || account.id
    byKey.set(key, account)
  }

  return [...byKey.values()]
}

function scheduleLoginRefreshes(refresh: () => Promise<void>) {
  refresh()
  window.setTimeout(refresh, 5_000)
  window.setTimeout(refresh, 15_000)
  window.setTimeout(refresh, 45_000)
}
