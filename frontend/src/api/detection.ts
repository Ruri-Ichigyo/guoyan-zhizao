import { http } from './http'

export interface DetectionRecord {
  id: string
  labels: string[]
  confidence_scores: number[]
  detected_classes: string[]
  created_at: string
  source: string
}

export interface CreateDetectionRequest {
  snapshot: string // base64 encoded image
  labels: string[]
  confidence_scores: number[]
  detected_classes: string[]
}

export interface PaginatedDetections {
  data: DetectionRecord[]
  pagination: {
    page: number
    limit: number
    total: number
    pages: number
  }
}

export const detectionApi = {
  async createDetection(request: CreateDetectionRequest): Promise<DetectionRecord> {
    const response = await http.post('/detections', request)
    return response.data
  },

  async listDetections(page: number = 1, limit: number = 20): Promise<PaginatedDetections> {
    const response = await http.get('/detections', {
      params: { page, limit }
    })
    return response.data
  },

  async getDetection(id: string): Promise<DetectionRecord> {
    const response = await http.get(`/detections/${id}`)
    return response.data
  },

  async getDetectionSnapshot(id: string): Promise<Blob> {
    const response = await http.get(`/detections/${id}/snapshot`, {
      responseType: 'blob'
    })
    return response.data
  }
}
