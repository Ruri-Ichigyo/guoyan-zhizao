<template>
  <div class="detections-view">
    <el-card class="main-card" shadow="hover">
      <template #header>
        <div class="flex justify-between items-center">
          <h3 class="text-lg font-semibold">实时识别记录</h3>
          <el-button type="primary" size="small" @click="refresh" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </template>

      <!-- Filters -->
      <div class="filters mb-4">
        <el-form :inline="true">
          <el-form-item label="每页显示">
            <el-select v-model="pageSize" size="small" @change="loadDetections">
              <el-option label="10条" :value="10" />
              <el-option label="20条" :value="20" />
              <el-option label="50条" :value="50" />
            </el-select>
          </el-form-item>
        </el-form>
      </div>

      <!-- Detections Table -->
      <el-table :data="detections" v-loading="loading" stripe style="width: 100%">
        <el-table-column prop="id" label="ID" width="100" show-overflow-tooltip>
          <template #default="scope">
            {{ scope.row.id.slice(0, 8) }}...
          </template>
        </el-table-column>

        <el-table-column label="截图" width="120">
          <template #default="scope">
            <el-image
              :src="getSnapshotUrl(scope.row.id)"
              :preview-src-list="[getSnapshotUrl(scope.row.id)]"
              fit="cover"
              style="width: 80px; height: 60px; border-radius: 4px; cursor: pointer;"
              lazy
            >
              <template #error>
                <div class="image-slot">
                  <el-icon><Picture /></el-icon>
                </div>
              </template>
            </el-image>
          </template>
        </el-table-column>

        <el-table-column label="识别结果" min-width="200">
          <template #default="scope">
            <div class="detection-results">
              <el-tag
                v-for="(label, index) in scope.row.labels"
                :key="index"
                type="success"
                size="small"
                class="mr-1 mb-1"
              >
                {{ label }} ({{ (scope.row.confidence_scores[index] * 100).toFixed(1) }}%)
              </el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="检测类别" min-width="150">
          <template #default="scope">
            <el-tag
              v-for="(cls, index) in scope.row.detected_classes"
              :key="index"
              type="info"
              size="small"
              class="mr-1"
            >
              {{ cls }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="created_at" label="时间" width="180">
          <template #default="scope">
            {{ formatDate(scope.row.created_at) }}
          </template>
        </el-table-column>

        <el-table-column label="来源" width="120">
          <template #default="scope">
            <el-tag size="small">{{ scope.row.source }}</el-tag>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="150" fixed="right">
          <template #default="scope">
            <el-button
              type="primary"
              size="small"
              link
              @click="viewDetails(scope.row)"
            >
              查看详情
            </el-button>
            <el-button
              type="primary"
              size="small"
              link
              @click="downloadSnapshot(scope.row.id)"
            >
              下载
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- Pagination -->
      <div class="pagination mt-4 flex justify-end">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- Details Dialog -->
    <el-dialog
      v-model="detailsVisible"
      title="识别详情"
      width="700px"
      destroy-on-close
    >
      <div v-if="selectedDetection" class="details-content">
        <div class="detail-section">
          <h4>截图预览</h4>
          <el-image
            :src="getSnapshotUrl(selectedDetection.id)"
            fit="contain"
            style="width: 100%; max-height: 400px; border-radius: 8px;"
          >
            <template #error>
              <div class="image-slot">
                <el-icon :size="50"><Picture /></el-icon>
                <div>加载失败</div>
              </div>
            </template>
          </el-image>
        </div>

        <div class="detail-section">
          <h4>识别信息</h4>
          <el-descriptions :column="2" border>
            <el-descriptions-item label="记录ID">
              {{ selectedDetection.id }}
            </el-descriptions-item>
            <el-descriptions-item label="识别时间">
              {{ formatDate(selectedDetection.created_at) }}
            </el-descriptions-item>
            <el-descriptions-item label="来源">
              {{ selectedDetection.source }}
            </el-descriptions-item>
            <el-descriptions-item label="识别结果" :span="2">
              <div class="flex flex-wrap gap-2">
                <el-tag
                  v-for="(label, index) in selectedDetection.labels"
                  :key="index"
                  type="success"
                >
                  {{ label }} - {{ (selectedDetection.confidence_scores[index] * 100).toFixed(2) }}%
                </el-tag>
              </div>
            </el-descriptions-item>
            <el-descriptions-item label="检测类别" :span="2">
              <div class="flex flex-wrap gap-2">
                <el-tag
                  v-for="(cls, index) in selectedDetection.detected_classes"
                  :key="index"
                  type="info"
                >
                  {{ cls }}
                </el-tag>
              </div>
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, Picture } from '@element-plus/icons-vue'
import { detectionApi, type DetectionRecord } from '@/api/detection'

// Data
const detections = ref<DetectionRecord[]>([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)
const detailsVisible = ref(false)
const selectedDetection = ref<DetectionRecord | null>(null)

// Methods
const loadDetections = async () => {
  loading.value = true
  try {
    const response = await detectionApi.listDetections(currentPage.value, pageSize.value)
    detections.value = response.data
    total.value = response.pagination.total
  } catch (error) {
    console.error('Failed to load detections:', error)
    ElMessage.error('加载识别记录失败')
  } finally {
    loading.value = false
  }
}

const refresh = () => {
  loadDetections()
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  loadDetections()
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  loadDetections()
}

const getSnapshotUrl = (id: string): string => {
  return `/api/v1/detections/${id}/snapshot`
}

const viewDetails = (detection: DetectionRecord) => {
  selectedDetection.value = detection
  detailsVisible.value = true
}

const downloadSnapshot = async (id: string) => {
  try {
    const blob = await detectionApi.getDetectionSnapshot(id)
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `detection-${id}-${Date.now()}.png`
    a.click()
    URL.revokeObjectURL(url)
    ElMessage.success('截图下载成功')
  } catch (error) {
    console.error('Failed to download snapshot:', error)
    ElMessage.error('下载截图失败')
  }
}

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// Lifecycle
onMounted(() => {
  loadDetections()
})
</script>

<style scoped>
.detections-view {
  width: 100%;
  padding: 20px;
}

.main-card {
  border-radius: 12px;
  border: 1px solid var(--color-border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.filters {
  padding: 16px;
  background: var(--color-background-light);
  border-radius: 8px;
}

.detection-results {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.image-slot {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--color-background-medium);
  color: var(--color-text-placeholder);
  font-size: 14px;
}

.details-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-section h4 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.pagination {
  margin-top: 20px;
}
</style>
