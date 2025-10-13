<template>
  <div class="upload-panel">
    <el-card>
      <template #header>
        <div class="flex items-center justify-between">
          <span class="text-lg font-medium">图像上传与检测</span>
        </div>
      </template>

      <!-- Upload Area -->
      <el-upload
        ref="uploadRef"
        class="upload-demo"
        drag
        :multiple="true"
        :show-file-list="false"
        :before-upload="handleBeforeUpload"
        :http-request="handleUpload"
        accept="image/*"
      >
        <el-icon class="el-icon--upload"><UploadFilled /></el-icon>
        <div class="el-upload__text">
          将文件拖到此处，或 <em>点击上传</em>
        </div>
        <template #tip>
          <div class="el-upload__tip text-gray-500">
            支持 jpg/png/gif 格式，单个文件大小不超过 10MB
          </div>
        </template>
      </el-upload>

      <!-- Upload Progress -->
      <div v-if="uploadingFiles.length > 0" class="mt-4">
        <h4 class="mb-2">上传进度</h4>
        <div v-for="file in uploadingFiles" :key="file.id" class="mb-3 p-3 border rounded">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium">{{ file.name }}</span>
            <span class="text-sm text-gray-500">{{ formatFileSize(file.size) }}</span>
          </div>
          <el-progress
            :percentage="file.progress"
            :status="file.status === 'error' ? 'exception' : undefined"
          />
          <div class="text-sm text-gray-500 mt-1">
            状态: {{ getStatusText(file.status) }}
          </div>
        </div>
      </div>

      <!-- Recent Uploads -->
      <div v-if="recentUploads.length > 0" class="mt-6">
        <h4 class="mb-4">最近上传</h4>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="upload in recentUploads"
            :key="upload.upload_id"
            class="border rounded-lg p-4"
          >
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">{{ upload.filename }}</span>
              <el-tag :type="getStatusColor(upload.status)" size="small">
                {{ upload.status }}
              </el-tag>
            </div>
            <div class="text-xs text-gray-500 mb-3">
              {{ formatTime(upload.created_at) }}
            </div>
            <div class="flex justify-between">
              <el-button
                size="small"
                @click="viewResult(upload.upload_id)"
                :disabled="upload.status !== 'completed'"
              >
                查看结果
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="deleteUpload(upload.upload_id)"
              >
                删除
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Result Detail Dialog -->
    <el-dialog
      v-model="resultDialogVisible"
      title="检测结果详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <div v-if="currentResult" class="result-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="上传ID">{{ currentResult.upload_id }}</el-descriptions-item>
          <el-descriptions-item label="文件名">{{ currentUpload?.filename }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusColor(currentResult.status)">
              {{ getStatusText(currentResult.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="检测到的果蔬数量">{{ currentResult.total_fruits }}</el-descriptions-item>
          <el-descriptions-item label="处理时间">{{ formatTime(currentResult.timestamp) }}</el-descriptions-item>
        </el-descriptions>

        <div v-if="currentResult.results && currentResult.results.length > 0" class="mt-4">
          <h4 class="text-base font-semibold mb-3">检测详情</h4>
          <el-table :data="currentResult.results" border stripe max-height="400">
            <el-table-column type="index" label="#" width="50" />
            <el-table-column prop="class_name" label="类别" width="120">
              <template #default="{ row }">
                <el-tag size="small">{{ row.class_name }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="confidence" label="置信度" width="120">
              <template #default="{ row }">
                <el-progress
                  :percentage="Math.round(row.confidence * 100)"
                  :stroke-width="6"
                  :color="getConfidenceColor(row.confidence * 100)"
                />
              </template>
            </el-table-column>
            <el-table-column prop="quality_grade" label="质量等级" width="120">
              <template #default="{ row }">
                <el-tag :type="getQualityTagType(row.quality_grade)" size="small">
                  {{ row.quality_grade || '未评级' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="边界框" min-width="200">
              <template #default="{ row }">
                <span v-if="row.bbox" class="text-xs">
                  [{{ row.bbox.join(', ') }}]
                </span>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <div v-else class="mt-4 text-center text-gray-500">
          未检测到任何果蔬
        </div>

        <div v-if="currentResult.metadata" class="mt-4">
          <h4 class="text-base font-semibold mb-3">其他信息</h4>
          <el-descriptions :column="2" border>
            <el-descriptions-item
              v-for="(value, key) in currentResult.metadata"
              :key="key"
              :label="String(key)"
            >
              {{ value }}
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>

      <div v-else class="text-center text-gray-500 py-8">
        无法加载结果数据
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="resultDialogVisible = false">关闭</el-button>
          <el-button type="primary" @click="exportResult">导出结果</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { UploadFilled } from '@element-plus/icons-vue'
import { uploadApi } from '@/api/upload'
import { useJobStore } from '@/stores/job'
import type { UploadFile, UploadRequestOptions } from 'element-plus'

const jobStore = useJobStore()

const uploadingFiles = ref<any[]>([])
const recentUploads = ref<any[]>([])
const resultDialogVisible = ref(false)
const currentResult = ref<any>(null)
const currentUpload = ref<any>(null)

const handleBeforeUpload = (file: File) => {
  // Validate file type
  const isImage = file.type.startsWith('image/')
  if (!isImage) {
    ElMessage.error('只能上传图片文件!')
    return false
  }

  // Validate file size (10MB)
  const isLt10M = file.size / 1024 / 1024 < 10
  if (!isLt10M) {
    ElMessage.error('上传文件大小不能超过 10MB!')
    return false
  }

  return true
}

const handleUpload = async (options: UploadRequestOptions) => {
  const file = options.file
  const uploadId = `upload_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

  const uploadItem = {
    id: uploadId,
    name: file.name,
    size: file.size,
    progress: 0,
    status: 'uploading' as 'uploading' | 'completed' | 'error'
  }

  uploadingFiles.value.push(uploadItem)

  try {
    // Direct upload only (presigned URL not supported by backend)
    const result = await uploadApi.uploadFile(file)

    uploadItem.progress = 100
    uploadItem.status = 'completed'

    // Add to recent uploads
    recentUploads.value.unshift({
      upload_id: result.upload_id,
      filename: file.name,
      status: result.status,
      created_at: new Date().toISOString()
    })

    // Store in job store
    jobStore.addJob({
      id: result.upload_id,
      filename: file.name,
      status: result.status,
      progress: 100,
      createdAt: new Date(),
      result
    })

    ElMessage.success('文件上传成功!')

  } catch (error) {
    console.error('Upload failed:', error)
    uploadItem.status = 'error'
    ElMessage.error('文件上传失败!')
  }

  // Remove from uploading list after a delay
  setTimeout(() => {
    const index = uploadingFiles.value.findIndex(f => f.id === uploadId)
    if (index !== -1) {
      uploadingFiles.value.splice(index, 1)
    }
  }, 3000)
}

const viewResult = (uploadId: string) => {
  console.log('View result for:', uploadId)

  // Find the job in the store
  const job = jobStore.getJob(uploadId)

  if (job && job.result) {
    currentResult.value = job.result
    currentUpload.value = recentUploads.value.find(u => u.upload_id === uploadId)
    resultDialogVisible.value = true
  } else {
    ElMessage.warning('未找到检测结果数据')
  }
}

const deleteUpload = async (uploadId: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这个上传记录吗？', '确认删除', {
      type: 'warning'
    })

    // Remove from recent uploads
    const index = recentUploads.value.findIndex(u => u.upload_id === uploadId)
    if (index !== -1) {
      recentUploads.value.splice(index, 1)
    }

    // Remove from job store
    jobStore.removeJob(uploadId)

    ElMessage.success('删除成功!')
  } catch {
    // User cancelled
  }
}

const exportResult = () => {
  if (!currentResult.value) {
    ElMessage.warning('无结果数据可导出')
    return
  }

  try {
    const resultData = {
      upload_id: currentResult.value.upload_id,
      filename: currentUpload.value?.filename,
      status: currentResult.value.status,
      total_fruits: currentResult.value.total_fruits,
      timestamp: currentResult.value.timestamp,
      results: currentResult.value.results,
      metadata: currentResult.value.metadata
    }

    const blob = new Blob([JSON.stringify(resultData, null, 2)], {
      type: 'application/json'
    })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `result_${currentResult.value.upload_id}.json`
    link.click()
    URL.revokeObjectURL(url)

    ElMessage.success('结果已导出')
  } catch (error) {
    console.error('Export failed:', error)
    ElMessage.error('导出失败')
  }
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatTime = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const getStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    uploading: '上传中',
    completed: '已完成',
    error: '上传失败',
    processing: '处理中',
    analyzed: '分析完成'
  }
  return statusMap[status] || status
}

const getStatusColor = (status: string) => {
  const colorMap: Record<string, 'success' | 'warning' | 'danger' | 'info'> = {
    completed: 'success',
    processing: 'warning',
    error: 'danger',
    uploading: 'info'
  }
  return colorMap[status] || 'info'
}

const getConfidenceColor = (confidence: number): string => {
  if (confidence >= 90) return '#67c23a'
  if (confidence >= 80) return '#409eff'
  if (confidence >= 70) return '#e6a23c'
  return '#f56c6c'
}

const getQualityTagType = (quality: string): string => {
  const typeMap: Record<string, string> = {
    excellent: 'success',
    good: 'info',
    fair: 'warning',
    poor: 'danger',
    'A': 'success',
    'B': 'info',
    'C': 'warning',
    'D': 'danger'
  }
  return typeMap[quality] || 'info'
}

onMounted(() => {
  // Load recent uploads from localStorage or API
  const saved = localStorage.getItem('recentUploads')
  if (saved) {
    recentUploads.value = JSON.parse(saved)
  }
})
</script>

<style scoped>
.upload-demo {
  margin: 20px 0;
}

.upload-demo .el-upload {
  border: 1px dashed var(--el-border-color);
  border-radius: 6px;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: var(--el-transition-duration-fast);
}

.upload-demo .el-upload:hover {
  border-color: var(--el-color-primary);
}
</style>