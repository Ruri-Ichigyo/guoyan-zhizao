import axios from 'axios'

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api/v1'

export interface DetectionResult {
  class_name: string
  confidence: number
  bbox: number[]
  quality_grade: string
}

export interface ResultSummary {
  upload_id: string
  filename: string
  status: string
  created_at: string
  total_fruits?: number
  grade_a?: number
  grade_b?: number
  grade_c?: number
  avg_confidence?: number
}

export interface PaginatedResults {
  data: ResultSummary[]
  pagination: {
    page: number
    limit: number
    total: number
    pages: number
  }
}

export interface QueryParams {
  page?: number
  limit?: number
  status?: string
}

export interface DetailedResult extends ResultSummary {
  detections: DetectionResult[]
}

export interface StatisticsResponse {
  totals: Record<string, number>
  averages: Record<string, number>
}

export const resultApi = {
  async getResult(uploadId: string): Promise<DetailedResult> {
    const response = await axios.get(`${API_BASE}/results/${uploadId}`)
    return response.data
  },

  async getResults(params: QueryParams = {}): Promise<PaginatedResults> {
    const response = await axios.get(`${API_BASE}/results`, { params })
    return response.data
  },

  async getStatistics(): Promise<StatisticsResponse> {
    const response = await axios.get(`${API_BASE}/statistics`)
    return response.data
  }
}
