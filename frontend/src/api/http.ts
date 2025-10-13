import axios from 'axios'

export const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api/v1'

export const http = axios.create({
  baseURL: API_BASE
})
