import axios from 'axios'

const API_BASE = 'http://localhost:8000'

export interface UploadResponse {
  upload_id: string
  presigned_url?: string
  status: string
}

export interface PresignRequest {
  filename: string
  content_type: string
}

export const uploadApi = {
  // Direct upload
  async uploadFile(file: File): Promise<UploadResponse> {
    const formData = new FormData()
    formData.append('file', file)

    const response = await axios.post(`${API_BASE}/api/upload`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        const percentCompleted = Math.round(
          (progressEvent.loaded * 100) / (progressEvent.total || 1)
        )
        console.log(`Upload progress: ${percentCompleted}%`)
      }
    })

    return response.data
  },

  // Get presigned URL for direct upload to MinIO
  async getPresignedUrl(request: PresignRequest): Promise<UploadResponse> {
    const response = await axios.post(`${API_BASE}/api/presign`, request)
    return response.data
  },

  // Upload to presigned URL
  async uploadToPresignedUrl(presignedUrl: string, file: File): Promise<void> {
    await axios.put(presignedUrl, file, {
      headers: {
        'Content-Type': file.type,
      },
      onUploadProgress: (progressEvent) => {
        const percentCompleted = Math.round(
          (progressEvent.loaded * 100) / (progressEvent.total || 1)
        )
        console.log(`Direct upload progress: ${percentCompleted}%`)
      }
    })
  },

  // Chunked upload for large files
  async uploadFileChunked(file: File, chunkSize: number = 5 * 1024 * 1024): Promise<UploadResponse> {
    const totalChunks = Math.ceil(file.size / chunkSize)
    const uploadId = `upload_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

    for (let i = 0; i < totalChunks; i++) {
      const start = i * chunkSize
      const end = Math.min(start + chunkSize, file.size)
      const chunk = file.slice(start, end)

      const formData = new FormData()
      formData.append('file', chunk)
      formData.append('upload_id', uploadId)
      formData.append('chunk_index', i.toString())
      formData.append('total_chunks', totalChunks.toString())
      formData.append('filename', file.name)

      await axios.post(`${API_BASE}/api/upload/chunk`, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        }
      })
    }

    // Complete multipart upload
    const response = await axios.post(`${API_BASE}/api/upload/complete`, {
      upload_id: uploadId,
      filename: file.name
    })

    return response.data
  }
}