import type { DetectionResult } from './result'
import { http } from './http'

export interface UploadResult {
  upload_id: string
  filename: string
  status: string
  created_at: string
  total_fruits: number
  grade_a: number
  grade_b: number
  grade_c: number
  avg_confidence: number
  detections: DetectionResult[]
}

export const uploadApi = {
  async uploadFile(file: File): Promise<UploadResult> {
    const formData = new FormData()
    formData.append('file', file)

    const response = await http.post('/uploads', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })

    return response.data
  }
}
