<template>
  <div class="records-view">
    <!-- Header -->
    <div class="records-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><DataBoard /></el-icon>
          识别记录查看
        </h1>
        <p class="page-subtitle">查看和管理所有果蔬分拣识别记录</p>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="exportRecords">
          <el-icon><Download /></el-icon>
          导出记录
        </el-button>
        <el-button @click="refreshRecords">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Statistics Summary -->
    <div class="summary-cards">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="总记录数" :value="summary.total_records" />
            <div class="statistic-extra">
              <el-tag type="info">全部来源</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="图像上传" :value="summary.upload_records" />
            <div class="statistic-extra">
              <el-tag type="primary">Upload</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="实时识别" :value="summary.detection_records" />
            <div class="statistic-extra">
              <el-tag type="success">Live</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="已完成" :value="summary.completed" />
            <div class="statistic-extra">
              <el-tag type="success">Completed</el-tag>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- Search and Filters -->
    <el-card class="search-card" shadow="never">
      <el-form :model="searchForm" :inline="true" class="search-form">
        <el-row :gutter="20">
          <el-col :span="6">
            <el-form-item label="来源类型">
              <el-select
                v-model="searchForm.source"
                placeholder="选择来源"
                clearable
                style="width: 100%"
                @change="handleSearch"
              >
                <el-option label="全部" value="all" />
                <el-option label="图像上传" value="upload" />
                <el-option label="摄像头" value="webcam" />
                <el-option label="移动应用" value="mobile" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="处理状态">
              <el-select
                v-model="searchForm.status"
                placeholder="选择状态"
                clearable
                style="width: 100%"
                @change="handleSearch"
              >
                <el-option label="已完成" value="completed" />
                <el-option label="处理中" value="processing" />
                <el-option label="失败" value="failed" />
                <el-option label="待处理" value="pending" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="每页显示">
              <el-select
                v-model="searchForm.limit"
                style="width: 100%"
                @change="handleSearch"
              >
                <el-option label="10条" :value="10" />
                <el-option label="20条" :value="20" />
                <el-option label="50条" :value="50" />
                <el-option label="100条" :value="100" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item>
              <el-button type="primary" @click="handleSearch">
                <el-icon><Search /></el-icon>
                搜索
              </el-button>
              <el-button @click="resetSearch">
                <el-icon><RefreshRight /></el-icon>
                重置
              </el-button>
            </el-form-item>
          </el-col>
        </el-row>
      </el-form>
    </el-card>

    <!-- Records Table -->
    <el-card class="records-card" shadow="never">
      <template #header>
        <div class="table-header">
          <h3>识别记录列表</h3>
          <div class="table-actions">
            <el-button-group>
              <el-button
                :type="viewMode === 'table' ? 'primary' : ''"
                size="small"
                @click="viewMode = 'table'"
              >
                <el-icon><Grid /></el-icon>
                表格
              </el-button>
              <el-button
                :type="viewMode === 'card' ? 'primary' : ''"
                size="small"
                @click="viewMode = 'card'"
              >
                <el-icon><Menu /></el-icon>
                卡片
              </el-button>
            </el-button-group>
            <el-dropdown @command="handleBatchAction">
              <el-button size="small">
                批量操作
                <el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="export">导出选中</el-dropdown-item>
                  <el-dropdown-item command="delete">删除选中</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <!-- Table View -->
      <el-table
        v-if="viewMode === 'table'"
        :data="records"
        :loading="loading"
        @selection-change="handleSelectionChange"
        @row-click="handleRowClick"
        stripe
        style="width: 100%"
      >
        <el-table-column type="selection" width="50" />
        <el-table-column label="预览图" width="90">
          <template #default="{ row }">
            <el-image
              :src="row.thumbnail_url"
              :preview-src-list="[row.thumbnail_url]"
              class="record-thumbnail"
              fit="cover"
              loading="lazy"
            >
              <template #error>
                <div class="image-slot">
                  <el-icon><Picture /></el-icon>
                </div>
              </template>
            </el-image>
          </template>
        </el-table-column>
        <el-table-column prop="filename" label="文件名" min-width="150" show-overflow-tooltip />
        <el-table-column label="来源" width="100">
          <template #default="{ row }">
            <el-tag :type="getSourceTagType(row.source)" size="small">
              {{ getSourceDisplayName(row.source) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="90">
          <template #default="{ row }">
            <el-tag :type="getStatusTagType(row.status)" size="small">
              {{ getStatusDisplayName(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="识别统计" min-width="200">
          <template #default="{ row }">
            <div class="stats-inline">
              <span class="stat-item">总数: {{ row.statistics.total_detections }}</span>
              <span class="stat-item grade-a">A: {{ row.statistics.grade_a }}</span>
              <span class="stat-item grade-b">B: {{ row.statistics.grade_b }}</span>
              <span class="stat-item grade-c">C: {{ row.statistics.grade_c }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="平均置信度" width="110" sortable>
          <template #default="{ row }">
            <el-progress
              :percentage="Math.round(row.statistics.avg_confidence * 100)"
              :color="getConfidenceColor(row.statistics.avg_confidence)"
              :stroke-width="6"
            />
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="160" sortable>
          <template #default="{ row }">
            {{ formatDateTime(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" align="center" fixed="right">
          <template #default="{ row }">
            <div class="operation-buttons">
              <el-button size="small" @click.stop="viewDetail(row)" title="查看详情">
                <el-icon><View /></el-icon>
              </el-button>
              <el-button size="small" @click.stop="downloadImage(row)" title="下载图像">
                <el-icon><Download /></el-icon>
              </el-button>
              <el-button size="small" type="danger" @click.stop="deleteRecord(row)" title="删除">
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- Card View -->
      <div v-else class="records-grid">
        <el-card
          v-for="record in records"
          :key="record.id"
          class="record-card"
          shadow="hover"
          @click="handleRowClick(record)"
        >
          <div class="record-card-content">
            <div class="record-image">
              <el-image
                :src="record.thumbnail_url"
                :preview-src-list="[record.thumbnail_url]"
                fit="cover"
                class="w-full h-full"
              >
                <template #error>
                  <div class="image-slot">
                    <el-icon><Picture /></el-icon>
                  </div>
                </template>
              </el-image>
              <div class="record-overlay">
                <el-checkbox
                  :model-value="selectedRecords.includes(record)"
                  @change="toggleSelection(record)"
                  @click.stop
                />
              </div>
            </div>
            <div class="record-info">
              <div class="record-filename">{{ record.filename }}</div>
              <div class="record-tags">
                <el-tag :type="getSourceTagType(record.source)" size="small">
                  {{ getSourceDisplayName(record.source) }}
                </el-tag>
                <el-tag :type="getStatusTagType(record.status)" size="small">
                  {{ getStatusDisplayName(record.status) }}
                </el-tag>
              </div>
              <div class="record-stats">
                <div class="stats-row">
                  <span class="label">识别总数:</span>
                  <span class="value">{{ record.statistics.total_detections }}</span>
                </div>
                <div class="stats-row">
                  <span class="label">等级分布:</span>
                  <span class="grades">
                    <span class="grade-a">A:{{ record.statistics.grade_a }}</span>
                    <span class="grade-b">B:{{ record.statistics.grade_b }}</span>
                    <span class="grade-c">C:{{ record.statistics.grade_c }}</span>
                  </span>
                </div>
                <div class="stats-row">
                  <span class="label">平均置信度:</span>
                  <span class="value">{{ formatConfidence(record.statistics.avg_confidence) }}</span>
                </div>
              </div>
              <div class="record-time">{{ formatDateTime(record.created_at) }}</div>
              <div class="record-actions">
                <el-button size="small" @click.stop="viewDetail(record)">
                  <el-icon><View /></el-icon>
                  详情
                </el-button>
                <el-button size="small" @click.stop="downloadImage(record)">
                  <el-icon><Download /></el-icon>
                  下载
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- Pagination -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.limit"
          :page-sizes="[10, 20, 50, 100]"
          :disabled="loading"
          :background="true"
          layout="total, sizes, prev, pager, next, jumper"
          :total="pagination.total"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- Detail Dialog -->
    <el-dialog
      v-model="detailVisible"
      title="识别记录详情"
      width="900px"
      :close-on-click-modal="false"
    >
      <div v-if="currentRecord" class="record-detail">
        <div class="detail-grid">
          <div class="detail-image">
            <el-image
              :src="currentRecord.thumbnail_url"
              :preview-src-list="[currentRecord.thumbnail_url]"
              fit="contain"
              class="detail-img"
            />
          </div>
          <div class="detail-info">
            <el-descriptions :column="2" border size="default">
              <el-descriptions-item label="文件名" :span="2">
                {{ currentRecord.filename }}
              </el-descriptions-item>
              <el-descriptions-item label="来源">
                <el-tag :type="getSourceTagType(currentRecord.source)">
                  {{ getSourceDisplayName(currentRecord.source) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="状态">
                <el-tag :type="getStatusTagType(currentRecord.status)">
                  {{ getStatusDisplayName(currentRecord.status) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="总识别数">
                {{ currentRecord.statistics.total_detections }}
              </el-descriptions-item>
              <el-descriptions-item label="唯一类别">
                {{ currentRecord.statistics.unique_classes }}
              </el-descriptions-item>
              <el-descriptions-item label="A级">
                <span class="grade-a">{{ currentRecord.statistics.grade_a }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="B级">
                <span class="grade-b">{{ currentRecord.statistics.grade_b }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="C级" :span="2">
                <span class="grade-c">{{ currentRecord.statistics.grade_c }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="平均置信度">
                {{ formatConfidence(currentRecord.statistics.avg_confidence) }}
              </el-descriptions-item>
              <el-descriptions-item label="创建时间" :span="2">
                {{ formatDateTime(currentRecord.created_at) }}
              </el-descriptions-item>
            </el-descriptions>

            <div v-if="currentRecord.detections && currentRecord.detections.length > 0" class="detections-section">
              <h4>识别详情 ({{ currentRecord.detections.length }} 个检测结果)</h4>
              <el-table :data="currentRecord.detections" size="small" max-height="300">
                <el-table-column prop="label" label="标签" width="120" />
                <el-table-column prop="class" label="类别" width="100">
                  <template #default="{ row }">
                    <el-tag size="small">{{ row.class }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="置信度" width="120">
                  <template #default="{ row }">
                    <el-progress
                      :percentage="Math.round(row.confidence * 100)"
                      :stroke-width="6"
                      :show-text="true"
                      :format="() => formatConfidence(row.confidence)"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="边界框" min-width="200">
                  <template #default="{ row }">
                    <span v-if="row.bbox">
                      [{{ row.bbox.join(', ') }}]
                    </span>
                    <span v-else class="text-gray-400">无</span>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="detailVisible = false">关闭</el-button>
          <el-button type="primary" @click="downloadImage(currentRecord)">
            <el-icon><Download /></el-icon>
            下载图像
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  DataBoard,
  Download,
  Refresh,
  Search,
  RefreshRight,
  Grid,
  Menu,
  ArrowDown,
  Picture,
  View,
  Delete
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'
import {
  recordsApi,
  type UnifiedRecord,
  type RecordsQuery,
  type RecordsSummary,
  formatConfidence,
  getSourceDisplayName,
  getStatusDisplayName
} from '@/api/records'

interface SearchForm {
  source: string
  status: string
  limit: number
}

// Search form data
const searchForm = reactive<SearchForm>({
  source: 'all',
  status: '',
  limit: 20
})

const records = ref<UnifiedRecord[]>([])
const selectedRecords = ref<UnifiedRecord[]>([])
const currentRecord = ref<UnifiedRecord | null>(null)
const loading = ref(false)
const detailVisible = ref(false)
const viewMode = ref<'table' | 'card'>('table')

const pagination = reactive({
  page: 1,
  limit: 20,
  total: 0,
  pages: 0
})

const summary = reactive<RecordsSummary>({
  total_records: 0,
  upload_records: 0,
  detection_records: 0,
  completed: 0,
  processing: 0,
  failed: 0
})

// Load records data
const loadRecords = async (silent = false) => {
  loading.value = true
  try {
    const query: RecordsQuery = {
      page: pagination.page,
      limit: pagination.limit,
      source: searchForm.source as any,
      status: searchForm.status || undefined
    }

    const response = await recordsApi.list(query)
    records.value = response.data
    pagination.total = response.pagination.total
    pagination.pages = response.pagination.pages

    // Update summary
    Object.assign(summary, response.summary)

    if (!silent) {
      ElMessage.success('数据加载成功')
    }
  } catch (error) {
    console.error('Failed to load records:', error)
    ElMessage.error('加载记录失败')
  } finally {
    loading.value = false
  }
}

// Search handling
const handleSearch = async () => {
  pagination.page = 1
  pagination.limit = searchForm.limit
  await loadRecords()
}

// Reset search conditions
const resetSearch = () => {
  searchForm.source = 'all'
  searchForm.status = ''
  searchForm.limit = 20
  pagination.page = 1
  handleSearch()
}

// Refresh records
const refreshRecords = async () => {
  await loadRecords()
}

// Handle table selection change
const handleSelectionChange = (selection: UnifiedRecord[]) => {
  selectedRecords.value = selection
}

// Toggle card selection
const toggleSelection = (record: UnifiedRecord) => {
  const index = selectedRecords.value.findIndex(r => r.id === record.id)
  if (index > -1) {
    selectedRecords.value.splice(index, 1)
  } else {
    selectedRecords.value.push(record)
  }
}

// Row click event
const handleRowClick = (record: UnifiedRecord) => {
  viewDetail(record)
}

// View detail
const viewDetail = async (record: UnifiedRecord) => {
  try {
    loading.value = true
    // Fetch full record details
    const detailedRecord = await recordsApi.getDetail(record.id)
    currentRecord.value = detailedRecord
    detailVisible.value = true
  } catch (error) {
    console.error('Failed to load record detail:', error)
    ElMessage.error('加载记录详情失败')
  } finally {
    loading.value = false
  }
}

// Download image
const downloadImage = async (record: UnifiedRecord | null) => {
  if (!record) return

  try {
    const blobUrl = await recordsApi.getImage(record.id)
    const link = document.createElement('a')
    link.href = blobUrl
    link.download = `${record.filename}-${Date.now()}.jpg`
    link.click()
    URL.revokeObjectURL(blobUrl)
    ElMessage.success('图像下载成功')
  } catch (error) {
    console.error('Failed to download image:', error)
    ElMessage.error('下载图像失败')
  }
}

// Delete record
const deleteRecord = async (record: UnifiedRecord) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除记录 "${record.filename}" 吗？此操作不可恢复。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    // Optimistic UI update: remove from local list immediately
    const recordIndex = records.value.findIndex(r => r.id === record.id)

    try {
      await recordsApi.delete(record.id)

      // Remove from local state
      if (recordIndex > -1) {
        records.value.splice(recordIndex, 1)
        pagination.total--
      }

      ElMessage.success('删除成功')

      // Silently refresh to ensure consistency with backend
      await loadRecords(true)
    } catch (error: any) {
      console.error('Failed to delete record:', error)

      // Check if it's a 404 error (record already deleted)
      if (error?.response?.status === 404) {
        ElMessage.warning('记录不存在或已被删除')
        // Still refresh the list to sync with backend
        await loadRecords(true)
      } else {
        ElMessage.error('删除失败，请重试')
        // Reload to restore the correct state
        await loadRecords(true)
      }
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('Delete operation error:', error)
    }
  }
}

// Batch action handling
const handleBatchAction = async (command: string) => {
  if (selectedRecords.value.length === 0) {
    ElMessage.warning('请先选择要操作的记录')
    return
  }

  try {
    switch (command) {
      case 'export':
        await exportRecords(selectedRecords.value)
        break
      case 'delete':
        await ElMessageBox.confirm(
          `确定要删除选中的 ${selectedRecords.value.length} 条记录吗？`,
          '批量删除',
          {
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            type: 'warning'
          }
        )

        // Delete all selected records
        let successCount = 0
        let failCount = 0
        for (const record of selectedRecords.value) {
          try {
            await recordsApi.delete(record.id)
            successCount++
          } catch (error: any) {
            console.error(`Failed to delete record ${record.id}:`, error)
            if (error?.response?.status === 404) {
              // Record already deleted, count as success
              successCount++
            } else {
              failCount++
            }
          }
        }

        selectedRecords.value = []

        if (failCount === 0) {
          ElMessage.success(`成功删除 ${successCount} 条记录`)
        } else {
          ElMessage.warning(`成功删除 ${successCount} 条，失败 ${failCount} 条`)
        }

        // Refresh list to sync with backend
        await loadRecords(true)
        break
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('Batch action failed:', error)
      ElMessage.error('批量操作失败')
    }
  }
}

// Export records
const exportRecords = async (recordsToExport = records.value) => {
  try {
    const dataToExport = Array.isArray(recordsToExport) ? recordsToExport : records.value

    const csvContent = [
      'ID,文件名,来源,状态,总识别数,A级,B级,C级,平均置信度,创建时间',
      ...dataToExport.map(record => [
        record.id,
        record.filename,
        getSourceDisplayName(record.source),
        getStatusDisplayName(record.status),
        record.statistics.total_detections,
        record.statistics.grade_a,
        record.statistics.grade_b,
        record.statistics.grade_c,
        formatConfidence(record.statistics.avg_confidence),
        formatDateTime(record.created_at)
      ].join(','))
    ].join('\n')

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `records_${new Date().toISOString().slice(0, 10)}.csv`
    link.click()
    URL.revokeObjectURL(link.href)

    ElMessage.success('导出成功')
  } catch (error) {
    console.error('Export failed:', error)
    ElMessage.error('导出失败')
  }
}

// Pagination size change
const handleSizeChange = (newSize: number) => {
  pagination.limit = newSize
  pagination.page = 1
  searchForm.limit = newSize
  loadRecords()
}

// Page change
const handleCurrentChange = (newPage: number) => {
  pagination.page = newPage
  loadRecords()
}

// Helper functions
const getSourceTagType = (source: string): string => {
  const types: Record<string, string> = {
    upload: 'primary',
    webcam: 'success',
    rtsp: 'warning',
    webrtc: 'info'
  }
  return types[source] || ''
}

const getStatusTagType = (status: string): string => {
  const types: Record<string, string> = {
    completed: 'success',
    processing: 'warning',
    failed: 'danger',
    pending: 'info'
  }
  return types[status] || ''
}

const getConfidenceColor = (confidence: number): string => {
  if (confidence >= 0.9) return '#67c23a'
  if (confidence >= 0.8) return '#409eff'
  if (confidence >= 0.7) return '#e6a23c'
  return '#f56c6c'
}

const formatDateTime = (dateString: string): string => {
  return dayjs(dateString).format('YYYY-MM-DD HH:mm:ss')
}

// Auto-refresh functionality
let refreshInterval: number | null = null

const startAutoRefresh = () => {
  // Refresh every 30 seconds
  refreshInterval = window.setInterval(() => {
    loadRecords(true) // Silent refresh
  }, 30000)
}

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

// Handle window visibility change (refresh when user returns to tab)
const handleVisibilityChange = () => {
  if (!document.hidden) {
    // User returned to the tab, refresh data silently
    loadRecords(true)
  }
}

// Component mounted
onMounted(() => {
  loadRecords()
  startAutoRefresh()
  document.addEventListener('visibilitychange', handleVisibilityChange)
})

// Component unmounted - cleanup
onUnmounted(() => {
  stopAutoRefresh()
  document.removeEventListener('visibilitychange', handleVisibilityChange)
})
</script>

<style scoped>
.records-view {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

.records-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-content {
  flex: 1;
}

.page-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 4px 0;
}

.page-subtitle {
  color: #606266;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.summary-cards {
  margin-bottom: 20px;
}

.summary-card {
  border-radius: 8px;
  transition: all 0.3s ease;
}

.summary-card:hover {
  transform: translateY(-2px);
}

.statistic-extra {
  margin-top: 8px;
}

.search-card {
  margin-bottom: 20px;
  border-radius: 8px;
}

.search-form {
  width: 100%;
}

.records-card {
  border-radius: 8px;
}

.table-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.record-thumbnail {
  width: 70px;
  height: 70px;
  border-radius: 4px;
  object-fit: cover;
}

.image-slot {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
  background: #f5f7fa;
  color: #909399;
}

.stats-inline {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat-item {
  font-size: 13px;
  padding: 2px 8px;
  border-radius: 4px;
  background: #f0f2f5;
}

.grade-a {
  color: #52c41a;
  font-weight: 600;
}

.grade-b {
  color: #faad14;
  font-weight: 600;
}

.grade-c {
  color: #f5222d;
  font-weight: 600;
}

.operation-buttons {
  display: flex;
  justify-content: space-around;
  align-items: center;
  gap: 4px;
}

.records-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.record-card {
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 8px;
}

.record-card:hover {
  transform: translateY(-2px);
}

.record-card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.record-image {
  position: relative;
  width: 100%;
  height: 180px;
  border-radius: 6px;
  overflow: hidden;
}

.record-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
}

.record-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.record-filename {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.record-tags {
  display: flex;
  gap: 8px;
}

.record-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.stats-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stats-row .label {
  color: #606266;
}

.stats-row .value {
  color: #303133;
  font-weight: 500;
}

.grades {
  display: flex;
  gap: 8px;
}

.record-time {
  font-size: 12px;
  color: #909399;
}

.record-actions {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.pagination-wrapper {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}

.record-detail {
  width: 100%;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.detail-image {
  display: flex;
  justify-content: center;
  align-items: start;
}

.detail-img {
  max-width: 100%;
  max-height: 400px;
  border-radius: 8px;
}

.detail-info {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.detections-section {
  margin-top: 16px;
}

.detections-section h4 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #303133;
}

/* Responsive design */
@media (max-width: 768px) {
  .records-view {
    padding: 12px;
  }

  .records-header {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .search-form :deep(.el-row) {
    flex-direction: column;
    gap: 0;
  }

  .search-form :deep(.el-col) {
    width: 100%;
    max-width: 100%;
  }

  .records-grid {
    grid-template-columns: 1fr;
  }

  .detail-grid {
    grid-template-columns: 1fr;
  }

  .table-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }
}
</style>
