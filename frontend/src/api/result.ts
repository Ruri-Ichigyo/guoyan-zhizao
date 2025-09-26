import axios from 'axios'

const API_BASE = 'http://localhost:8000'

export interface DetectionResult {
  class_name: string
  confidence: number
  bbox: number[]
  quality_grade: string
}

export interface JobResult {
  job_id: string
  status: string
  results: DetectionResult[]
  created_at: string
  total_fruits?: number
  grade_a?: number
  grade_b?: number
  grade_c?: number
}

export interface PaginatedResults {
  data: JobResult[]
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
  from_date?: string
  to_date?: string
}

export const resultApi = {
  // Get specific job result
  async getResult(jobId: string): Promise<JobResult> {
    const response = await axios.get(`${API_BASE}/api/result/${jobId}`)
    return response.data
  },

  // Get paginated results
  async getResults(params: QueryParams = {}): Promise<PaginatedResults> {
    const response = await axios.get(`${API_BASE}/api/results`, {
      params
    })
    return response.data
  },

  // Get statistics
  async getStatistics(timeRange: string = '24h'): Promise<any> {
    const response = await axios.get(`${API_BASE}/api/statistics`, {
      params: { range: timeRange }
    })
    return response.data
  },

  // Export results
  async exportResults(format: string = 'csv', params: QueryParams = {}): Promise<Blob> {
    const response = await axios.get(`${API_BASE}/api/results/export`, {
      params: { ...params, format },
      responseType: 'blob'
    })
    return response.data
  },

  // Delete result
  async deleteResult(jobId: string): Promise<void> {
    await axios.delete(`${API_BASE}/api/result/${jobId}`)
  },

  // Retry analysis
  async retryAnalysis(jobId: string): Promise<JobResult> {
    const response = await axios.post(`${API_BASE}/api/result/${jobId}/retry`)
    return response.data
  }
}