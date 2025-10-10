import { computed, reactive } from 'vue'

export interface UserProfile {
  user_id: string
  username: string
  role: string
}

interface AuthState {
  token: string
  expiresAt: string | null
  user: UserProfile | null
}

const STORAGE_TOKEN = 'gyz_jwt_token'
const STORAGE_EXPIRES = 'gyz_jwt_expires_at'
const STORAGE_USER = 'gyz_user_profile'

const state: AuthState = reactive({
  token: localStorage.getItem(STORAGE_TOKEN) ?? '',
  expiresAt: localStorage.getItem(STORAGE_EXPIRES),
  user: (() => {
    try {
      const raw = localStorage.getItem(STORAGE_USER)
      return raw ? (JSON.parse(raw) as UserProfile) : null
    } catch (error) {
      console.warn('Failed to parse cached user profile', error)
      return null
    }
  })()
})

const persist = () => {
  if (state.token) {
    localStorage.setItem(STORAGE_TOKEN, state.token)
  } else {
    localStorage.removeItem(STORAGE_TOKEN)
  }

  if (state.expiresAt) {
    localStorage.setItem(STORAGE_EXPIRES, state.expiresAt)
  } else {
    localStorage.removeItem(STORAGE_EXPIRES)
  }

  if (state.user) {
    localStorage.setItem(STORAGE_USER, JSON.stringify(state.user))
  } else {
    localStorage.removeItem(STORAGE_USER)
  }
}

const isExpired = () => {
  if (!state.expiresAt) return true
  const expires = Date.parse(state.expiresAt)
  if (Number.isNaN(expires)) return true
  return expires <= Date.now()
}

export const useUserStore = () => {
  const ensureSession = () => {
    if (!state.token) {
      return false
    }
    if (isExpired()) {
      clearAuth()
      return false
    }
    return true
  }

  const setAuth = (token: string, expiresAt: string, profile: UserProfile) => {
    state.token = token
    state.expiresAt = expiresAt
    state.user = profile
    persist()
  }

  const clearAuth = () => {
    state.token = ''
    state.expiresAt = null
    state.user = null
    persist()
  }

  const updateProfile = (profile: UserProfile) => {
    state.user = profile
    persist()
  }

  const token = computed(() => state.token)
  const user = computed(() => state.user)
  const role = computed(() => state.user?.role ?? '')
  const isAuthenticated = computed(() => !!state.token && !isExpired())
  const canUpload = computed(() => {
    const value = role.value.toLowerCase()
    return value === 'admin' || value === 'operator'
  })

  return {
    token,
    user,
    role,
    isAuthenticated,
    canUpload,
    ensureSession,
    setAuth,
    clearAuth,
    updateProfile
  }
}
