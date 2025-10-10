<template>
  <div class="upload-wrapper">
    <el-card class="upload-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div>
            <h2 class="card-title">图像上传</h2>
            <p class="card-subtitle">上传果蔬图片并触发 AI 分析</p>
          </div>
          <el-button type="primary" :loading="uploading" :disabled="!canUpload" @click="openFileDialog">
            <el-icon class="mr-1"><UploadFilled /></el-icon>
            选择图片
          </el-button>
          <input
            ref="fileInput"
            type="file"
            accept="image/*"
            class="hidden-input"
            @change="handleFileChange"
          />
        </div>
      </template>

      <el-alert
        v-if="!canUpload"
        title="当前账号没有上传权限"
        type="warning"
        description="请联系管理员开通上传或分析权限"
        show-icon
        class="mb-4"
      />

      <div v-if="uploading" class="upload-state">
        <el-skeleton animated />
        <p class="mt-3 text-slate-500">正在上传并分析，请稍候…</p>
      </div>

      <div v-else-if="uploadError" class="upload-error">
        <el-result icon="error" title="上传失败" :sub-title="uploadError">
          <template #extra>
            <el-button type="primary" @click="resetError">重新上传</el-button>
          </template>
        </el-result>
      </div>

      <div v-else-if="lastResult" class="upload-result">
        <div class="result-header">
          <div>
            <h3>最近一次分析</h3>
            <p class="meta">
              任务 ID：{{ lastResult.upload_id }} · 上传时间：{{ formatDate(lastResult.created_at) }}
            </p>
          </div>
          <el-tag type="success">{{ lastResult.status }}</el-tag>
        </div>

        <el-row :gutter="12" class="result-summary">
          <el-col :span="6">
            <el-statistic title="识别总数" :value="lastResult.total_fruits" />
          </el-col>
          <el-col :span="6">
            <el-statistic title="A级" :value="lastResult.grade_a" />
          </el-col>
          <el-col :span="6">
            <el-statistic title="B级" :value="lastResult.grade_b" />
          </el-col>
          <el-col :span="6">
            <el-statistic title="C级" :value="lastResult.grade_c" />
          </el-col>
        </el-row>

        <el-table :data="lastResult.detections" border stripe>
          <el-table-column prop="class_name" label="类别" />
          <el-table-column label="置信度">
            <template #default="{ row }">
              <el-progress :percentage="Math.round(row.confidence * 100)" />
            </template>
          </el-table-column>
          <el-table-column prop="quality_grade" label="质量等级" width="110">
            <template #default="{ row }">
              <el-tag :type="gradeTag(row.quality_grade)" size="small">
                {{ row.quality_grade }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <div v-else class="upload-placeholder">
        <el-empty description="选择或拖入图片以开始分析">
          <el-button type="primary" @click="openFileDialog">立即上传</el-button>
        </el-empty>
      </div>
    </el-card>

    <el-card v-if="recentResults.length" class="recent-card" shadow="never">
      <template #header>
        <div class="card-header">
          <div>
            <h3 class="card-title">最近任务</h3>
            <p class="card-subtitle">展示最近 5 条检测记录</p>
          </div>
          <el-button type="text" @click="$router.push('/results')">查看全部</el-button>
        </div>
      </template>

      <el-table :data="recentResults" size="small">
        <el-table-column prop="filename" label="文件名" min-width="140" />
        <el-table-column prop="status" label="状态" width="90">
          <template #default="{ row }">
            <el-tag :type="statusTag(row.status)" size="small">{{ row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="total_fruits" label="识别数量" width="110" />
        <el-table-column label="上传时间" width="160">
          <template #default="{ row }">
            {{ formatDate(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="110">
          <template #default="{ row }">
            <el-button size="small" type="primary" text @click="viewDetail(row.upload_id)">
              查看详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog
      v-model="detailVisible"
      width="600px"
      :title="dialogResult?.filename ?? '检测详情'"
    >
      <template v-if="dialogResult">
        <p class="meta">
          任务 ID：{{ dialogResult.upload_id }} · {{ formatDate(dialogResult.created_at) }}
        </p>
        <el-table :data="dialogResult.detections" stripe style="margin-top: 1rem">
          <el-table-column prop="class_name" label="类别" />
          <el-table-column label="置信度">
            <template #default="{ row }">
              {{ (row.confidence * 100).toFixed(1) }}%
            </template>
          </el-table-column>
          <el-table-column prop="quality_grade" label="质量等级" width="100" />
        </el-table>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { UploadFilled } from '@element-plus/icons-vue'

import type { DetailedResult, ResultSummary } from '@/api/result'
import { resultApi } from '@/api/result'
import type { UploadResult } from '@/api/upload'
import { uploadApi } from '@/api/upload'
import { useUserStore } from '@/stores/user'

const uploading = ref(false)
const uploadError = ref<string | null>(null)
const lastResult = ref<UploadResult | null>(null)
const recentResults = ref<ResultSummary[]>([])
const detailVisible = ref(false)
const dialogResult = ref<DetailedResult | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const userStore = useUserStore()
const canUpload = computed(() => userStore.canUpload.value)

const openFileDialog = () => {
  if (!canUpload.value) {
    ElMessage.warning('当前账号无权上传图片')
    return
  }
  fileInput.value?.click()
}

const resetError = () => {
  uploadError.value = null
}

const handleFileChange = async (event: Event) => {
  const input = event.target as HTMLInputElement
  if (!input.files || input.files.length === 0) return

  const file = input.files[0]
  if (!file.type.startsWith('image/')) {
    ElMessage.error('仅支持图片文件')
    return
  }

  if (!canUpload.value) {
    ElMessage.warning('当前账号无权上传图片')
    return
  }

  uploadError.value = null
  uploading.value = true
  try {
    const response = await uploadApi.uploadFile(file)
    lastResult.value = response
    ElMessage.success('上传并分析完成')
    await loadRecentResults()
  } catch (error) {
    console.error(error)
    uploadError.value = '上传或分析失败，请稍后重试'
  } finally {
    uploading.value = false
    if (input) input.value = ''
  }
}

const loadRecentResults = async () => {
  try {
    const payload = await resultApi.getResults({ page: 1, limit: 5 })
    recentResults.value = payload.data
  } catch (error) {
    console.error(error)
  }
}

const viewDetail = async (uploadId: string) => {
  try {
    dialogResult.value = await resultApi.getResult(uploadId)
    detailVisible.value = true
  } catch (error) {
    console.error(error)
    ElMessage.error('加载详情失败')
  }
}

const formatDate = (value: string) => {
  return new Date(value).toLocaleString()
}

const gradeTag = (grade: string) => {
  switch (grade.toUpperCase()) {
    case 'A':
      return 'success'
    case 'B':
      return 'warning'
    case 'C':
      return 'danger'
    default:
      return 'info'
  }
}

const statusTag = (status: string) => {
  switch (status) {
    case 'completed':
      return 'success'
    case 'processing':
      return 'warning'
    case 'failed':
      return 'danger'
    default:
      return 'info'
  }
}

onMounted(async () => {
  await loadRecentResults()
})
</script>

<style scoped>
.upload-wrapper {
  display: grid;
  gap: 1.5rem;
}

.upload-card {
  border-radius: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.5rem;
}

.card-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.card-subtitle {
  margin: 0.25rem 0 0;
  color: #738192;
  font-size: 0.95rem;
}

.hidden-input {
  display: none;
}

.upload-state {
  text-align: center;
  padding: 2rem 0;
}

.upload-result {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.result-summary {
  margin: 0.5rem 0 1rem;
}

.meta {
  color: #738192;
  font-size: 0.9rem;
}

.recent-card {
  border-radius: 12px;
}

.upload-placeholder {
  padding: 2rem 0;
}
</style>
