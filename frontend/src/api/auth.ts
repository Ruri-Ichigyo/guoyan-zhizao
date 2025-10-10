import { http } from './http'
import type { UserProfile } from '@/stores/user'

export interface LoginPayload {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  expires_at: string
  user: UserProfile
}

export const authApi = {
  async login(payload: LoginPayload): Promise<LoginResponse> {
    const response = await http.post('/auth/login', payload)
    return response.data
  },

  async me(): Promise<UserProfile> {
    const response = await http.get('/auth/me')
    return response.data
  }
}
