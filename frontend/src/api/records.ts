import { http } from './http'

/**
 * Detection record structure
 */
export interface DetectionRecord {
  label: string
  class: string
  confidence: number
  bbox?: [number, number, number, number]
}

/**
 * Record statistics
 */
export interface RecordStatistics {
  total_detections: number  // 总检测数
  grade_a: number           // A级数量（基于水果质量，非置信度）
  grade_b: number           // B级数量（基于水果质量，非置信度）
  grade_c: number           // C级数量（基于水果质量，非置信度）
  avg_confidence: number    // 平均识别置信度（0-1范围）
  unique_classes: number    // 唯一类别数
}

/**
 * Unified record structure
 */
export interface UnifiedRecord {
  id: string
  filename: string
  source: string  // 数据库中的实际source值（如：live_camera, camera_1, mobile_app等）
  record_type: 'upload' | 'detection'
  status: string
  thumbnail_url: string
  detections: DetectionRecord[]
  statistics: RecordStatistics
  created_at: string
}

/**
 * Pagination information
 */
export interface PaginationInfo {
  page: number
  limit: number
  total: number
  pages: number
}

/**
 * Records summary
 */
export interface RecordsSummary {
  total_records: number
  upload_records: number
  detection_records: number
  completed: number
  processing: number
  failed: number
}

/**
 * Unified records response
 */
export interface UnifiedRecordsResponse {
  data: UnifiedRecord[]
  pagination: PaginationInfo
  summary: RecordsSummary
}

/**
 * Records query parameters
 *
 * Source过滤说明：
 * - 'all': 所有记录
 * - 'upload': 仅图像上传记录
 * - 'webcam': 摄像头记录（包括live_camera, camera_1, camera_2）
 * - 'mobile': 移动应用记录（mobile_app）
 */
export interface RecordsQuery {
  page?: number
  limit?: number
  source?: 'all' | 'upload' | 'webcam' | 'mobile'
  status?: string
}

/**
 * Unified Records API
 */
export const recordsApi = {
  /**
   * List records with pagination and filtering
   */
  async list(query: RecordsQuery = {}): Promise<UnifiedRecordsResponse> {
    const params = new URLSearchParams()

    if (query.page) params.append('page', query.page.toString())
    if (query.limit) params.append('limit', query.limit.toString())
    if (query.source && query.source !== 'all') params.append('source', query.source)
    if (query.status) params.append('status', query.status)

    const response = await http.get<UnifiedRecordsResponse>(
      `/records?${params.toString()}`
    )
    return response.data
  },

  /**
   * Get single record detail by ID
   */
  async getDetail(id: string): Promise<UnifiedRecord> {
    const response = await http.get<UnifiedRecord>(`/records/${id}`)
    return response.data
  },

  /**
   * Get record image/snapshot by ID
   * Returns a blob URL that can be used in <img> src
   */
  async getImage(id: string): Promise<string> {
    const response = await http.get(`/records/${id}/image`, {
      responseType: 'blob'
    })
    return URL.createObjectURL(response.data)
  },

  /**
   * Delete a record by ID
   */
  async delete(id: string): Promise<void> {
    await http.delete(`/records/${id}`)
  },

  /**
   * Get statistics for specific source
   */
  async getStatsBySource(source: string): Promise<RecordsSummary> {
    const response = await http.get<UnifiedRecordsResponse>(
      `/records?source=${source}&limit=1`
    )
    return response.data.summary
  }
}

/**
 * Helper function to format confidence percentage
 */
export function formatConfidence(confidence: number): string {
  return `${(confidence * 100).toFixed(1)}%`
}

/**
 * Helper function to get source display name
 * 将数据库中的source值映射为用户友好的显示名称
 */
export function getSourceDisplayName(source: string): string {
  const sourceNames: Record<string, string> = {
    upload: '图像上传',
    live_camera: '实时摄像头',
    camera_1: '摄像头1',
    camera_2: '摄像头2',
    mobile_app: '移动应用',
    webcam: '摄像头',
    mobile: '移动应用'
  }
  return sourceNames[source] || source
}

/**
 * Helper function to get status display name
 */
export function getStatusDisplayName(status: string): string {
  const statusNames: Record<string, string> = {
    completed: '已完成',
    processing: '处理中',
    failed: '失败',
    pending: '待处理'
  }
  return statusNames[status] || status
}

/**
 * Helper function to get grade color
 */
export function getGradeColor(grade: string): string {
  const gradeColors: Record<string, string> = {
    A: '#52c41a',
    B: '#faad14',
    C: '#f5222d'
  }
  return gradeColors[grade.toUpperCase()] || '#d9d9d9'
}
