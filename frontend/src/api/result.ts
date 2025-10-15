import { http } from './http'

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
    const response = await http.get(`/results/${uploadId}`)
    return response.data
  },

  async getResults(params: QueryParams = {}): Promise<PaginatedResults> {
    const response = await http.get('/results', { params })
    return response.data
  },

  async getStatistics(): Promise<StatisticsResponse> {
    const response = await http.get('/statistics')
    return response.data
  }
}
