import axios from 'axios'

import router from '@/router'
import { useUserStore } from '@/stores/user'

export const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api/v1'

export const http = axios.create({
  baseURL: API_BASE
})

http.interceptors.request.use((config) => {
  const store = useUserStore()
  if (store.ensureSession()) {
    if (!config.headers) {
      config.headers = {}
    }
    if (store.token.value) {
      config.headers.Authorization = `Bearer ${store.token.value}`
    }
  } else if (store.token.value) {
    store.clearAuth()
  }
  return config
})

http.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      const store = useUserStore()
      store.clearAuth()
      if (router.currentRoute.value.name !== 'Login') {
        router.push({ name: 'Login', query: { redirect: router.currentRoute.value.fullPath } })
      }
    }
    return Promise.reject(error)
  }
)
