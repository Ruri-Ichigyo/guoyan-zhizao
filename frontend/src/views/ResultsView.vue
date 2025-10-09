<template>
  <div class="results-page">
    <div class="page-header">
      <div>
        <h2>检测结果</h2>
        <p class="subtitle">查询历史上传记录，查看 AI 分析详情</p>
      </div>
      <el-button type="primary" @click="$router.push('/upload')">
        <el-icon class="mr-1"><UploadFilled /></el-icon>
        新建上传
      </el-button>
    </div>

    <el-row :gutter="16" class="stat-row">
      <el-col :span="6" v-for="card in statCards" :key="card.title">
        <el-card shadow="hover" class="stat-card">
          <p class="stat-title">{{ card.title }}</p>
          <p class="stat-value">{{ card.value }}</p>
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never">
      <template #header>
        <div class="table-header">
          <div>
            <h3>历史记录</h3>
            <p class="table-subtitle">共 {{ pagination.total }} 条</p>
          </div>
          <div class="filters">
            <el-select v-model="filters.status" placeholder="状态筛选" clearable @change="handleFilter">
              <el-option label="已完成" value="completed" />
              <el-option label="处理中" value="processing" />
              <el-option label="失败" value="failed" />
            </el-select>
          </div>
        </div>
      </template>

      <el-table :data="results" :loading="loading" stripe border>
        <el-table-column prop="filename" label="文件名" min-width="160" />
        <el-table-column prop="status" label="状态" width="110">
          <template #default="{ row }">
            <el-tag :type="statusTag(row.status)">{{ row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="total_fruits" label="识别数量" width="120" />
        <el-table-column label="各等级" min-width="160">
          <template #default="{ row }">
            A: {{ row.grade_a ?? 0 }} · B: {{ row.grade_b ?? 0 }} · C: {{ row.grade_c ?? 0 }}
          </template>
        </el-table-column>
        <el-table-column prop="avg_confidence" label="平均置信度" width="140">
          <template #default="{ row }">
            {{ row.avg_confidence ? (row.avg_confidence * 100).toFixed(1) + '%' : '-' }}
          </template>
        </el-table-column>
        <el-table-column label="上传时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" text @click="openDetail(row.upload_id)">
              查看详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="table-footer">
        <el-pagination
          background
          layout="total, prev, pager, next"
          :current-page="pagination.page"
          :page-size="pagination.limit"
          :total="pagination.total"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <el-drawer v-model="detailVisible" title="检测详情" size="40%">
      <template v-if="selectedResult">
        <p class="meta">
          ID：{{ selectedResult.upload_id }} · 文件：{{ selectedResult.filename }}
        </p>
        <p class="meta">
          上传时间：{{ formatDate(selectedResult.created_at) }}
        </p>
        <el-table :data="selectedResult.detections" style="margin-top: 1rem" border>
          <el-table-column prop="class_name" label="类别" min-width="150" />
          <el-table-column label="置信度" width="120">
            <template #default="{ row }">
              {{ (row.confidence * 100).toFixed(1) }}%
            </template>
          </el-table-column>
          <el-table-column prop="quality_grade" label="等级" width="100" />
        </el-table>
      </template>
      <el-empty v-else description="选择一个记录以查看详情" />
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { UploadFilled } from '@element-plus/icons-vue'

import type { DetailedResult, PaginatedResults, ResultSummary, StatisticsResponse } from '@/api/result'
import { resultApi } from '@/api/result'

const loading = ref(false)
const results = ref<ResultSummary[]>([])
const pagination = reactive({
  page: 1,
  limit: 10,
  total: 0,
  pages: 1
})

const statistics = ref<StatisticsResponse | null>(null)
const filters = reactive({
  status: ''
})

const detailVisible = ref(false)
const selectedResult = ref<DetailedResult | null>(null)

const statCards = computed(() => {
  if (!statistics.value) return []
  const totals = statistics.value.totals
  return [
    { title: '累计上传', value: totals.total ?? 0 },
    { title: '已完成', value: totals.completed ?? 0 },
    { title: '处理中', value: totals.processing ?? 0 },
    { title: '识别总数', value: totals.total_fruits ?? 0 }
  ]
})

const fetchStatistics = async () => {
  try {
    statistics.value = await resultApi.getStatistics()
  } catch (error) {
    console.error(error)
  }
}

const fetchResults = async () => {
  loading.value = true
  try {
    const payload: PaginatedResults = await resultApi.getResults({
      page: pagination.page,
      limit: pagination.limit,
      status: filters.status || undefined
    })
    results.value = payload.data
    Object.assign(pagination, payload.pagination)
  } catch (error) {
    console.error(error)
    ElMessage.error('加载记录失败')
  } finally {
    loading.value = false
  }
}

const handlePageChange = (page: number) => {
  pagination.page = page
  fetchResults()
}

const handleFilter = () => {
  pagination.page = 1
  fetchResults()
}

const openDetail = async (uploadId: string) => {
  try {
    selectedResult.value = await resultApi.getResult(uploadId)
    detailVisible.value = true
  } catch (error) {
    console.error(error)
    ElMessage.error('加载详情失败')
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

const formatDate = (value: string) => {
  return new Date(value).toLocaleString()
}

onMounted(async () => {
  await Promise.all([fetchStatistics(), fetchResults()])
})

watch(
  () => filters.status,
  () => {
    pagination.page = 1
  }
)
</script>

<style scoped>
.results-page {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1.5rem;
}

.page-header h2 {
  margin: 0;
  font-size: 1.8rem;
}

.subtitle {
  margin: 0.4rem 0 0;
  color: #64748b;
}

.stat-row {
  margin-bottom: 0.5rem;
}

.stat-card {
  border-radius: 14px;
  text-align: left;
}

.stat-title {
  margin: 0;
  font-size: 0.95rem;
  color: #748094;
}

.stat-value {
  margin: 0.4rem 0 0;
  font-size: 1.75rem;
  font-weight: 600;
  color: #0f172a;
}

.table-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1.5rem;
}

.table-subtitle {
  margin: 0.3rem 0 0;
  color: #94a3b8;
}

.filters {
  display: flex;
  gap: 1rem;
}

.table-footer {
  margin-top: 1rem;
  display: flex;
  justify-content: flex-end;
}

.meta {
  color: #64748b;
  font-size: 0.9rem;
  margin: 0.25rem 0;
}
</style>
