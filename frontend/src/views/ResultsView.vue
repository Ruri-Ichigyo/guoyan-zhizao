<template>
  <div class="results-view">
    <!-- Header -->
    <div class="results-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><DataBoard /></el-icon>
          检测结果查看
        </h1>
        <p class="page-subtitle">查看和管理果蔬智能分拣检测结果</p>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="exportResults">
          <el-icon><Download /></el-icon>
          导出结果
        </el-button>
        <el-button @click="refreshResults">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- Search and Filters -->
    <el-card class="search-card" shadow="never">
      <el-form
        :model="searchForm"
        :inline="true"
        class="search-form"
        @submit.prevent="handleSearch"
      >
        <el-row :gutter="20">
          <el-col :span="6">
            <el-form-item label="检测批次">
              <el-input
                v-model="searchForm.batchId"
                placeholder="输入批次ID"
                clearable
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="分类结果" class="category-form-item">
              <el-select
                v-model="searchForm.category"
                placeholder="选择分类结果"
                clearable
                style="width: 100%"
              >
                <el-option
                  v-for="item in categoryOptions"
                  :key="item.value"
                  :label="item.label"
                  :value="item.value"
                />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="质量等级" class="quality-form-item">
              <el-select
                v-model="searchForm.quality"
                placeholder="选择质量等级"
                clearable
                style="width: 100%"
              >
                <el-option
                  v-for="item in qualityOptions"
                  :key="item.value"
                  :label="item.label"
                  :value="item.value"
                />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="置信度" class="confidence-form-item">
              <el-slider
                v-model="searchForm.confidenceRange"
                range
                :min="0"
                :max="100"
                :format-tooltip="formatConfidence"
                class="confidence-slider"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-row :gutter="16">
          <el-col :span="8">
            <el-form-item label="检测时间">
              <el-date-picker
                v-model="searchForm.dateRange"
                type="datetimerange"
                range-separator="至"
                start-placeholder="开始时间"
                end-placeholder="结束时间"
                format="YYYY-MM-DD HH:mm"
                value-format="YYYY-MM-DD HH:mm:ss"
                clearable
              />
            </el-form-item>
          </el-col>
          <el-col :span="4">
            <el-form-item label="包含缺陷">
              <el-switch
                v-model="searchForm.hasDefects"
                active-text="是"
                inactive-text="否"
              />
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item>
              <el-button type="primary" @click="handleSearch" native-type="submit">
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

    <!-- Statistics Summary -->
    <div class="summary-cards">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="总检测数" :value="statistics.total" suffix="个" />
            <div class="statistic-extra">
              <el-tag type="info">当前筛选</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="合格数" :value="statistics.qualified" suffix="个">
              <template #suffix>
                <span>个</span>
              </template>
            </el-statistic>
            <div class="statistic-extra">
              <el-tag type="success">{{ ((statistics.qualified / statistics.total) * 100).toFixed(1) }}%</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="不合格数" :value="statistics.unqualified" suffix="个">
              <template #suffix>
                <span>个</span>
              </template>
            </el-statistic>
            <div class="statistic-extra">
              <el-tag type="danger">{{ ((statistics.unqualified / statistics.total) * 100).toFixed(1) }}%</el-tag>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="summary-card" shadow="hover">
            <el-statistic title="平均置信度" :value="statistics.avgConfidence" suffix="%" :precision="1" />
            <div class="statistic-extra">
              <el-tag :type="statistics.avgConfidence > 90 ? 'success' : statistics.avgConfidence > 80 ? 'warning' : 'danger'">
                {{ statistics.avgConfidence > 90 ? '优秀' : statistics.avgConfidence > 80 ? '良好' : '需改进' }}
              </el-tag>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- Results Table -->
    <el-card class="results-card" shadow="never">
      <template #header>
        <div class="table-header">
          <h3>检测结果列表</h3>
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
                  <el-dropdown-item command="reprocess">重新处理</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <!-- Table View -->
      <el-table
        v-if="viewMode === 'table'"
        ref="resultsTable"
        :data="results"
        :loading="loading"
        @selection-change="handleSelectionChange"
        @row-click="handleRowClick"
        stripe
        style="width: 100%"
      >
        <el-table-column type="selection" width="50" />
        <el-table-column label="预览图" width="70">
          <template #default="{ row }">
            <el-image
              :src="row.imageUrl"
              :preview-src-list="[row.imageUrl]"
              class="result-thumbnail"
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
        <el-table-column prop="batchId" label="批次ID" width="100" sortable />
        <el-table-column prop="itemId" label="物品ID" width="100" />
        <el-table-column label="分类结果" width="100">
          <template #default="{ row }">
            <el-tag :type="getCategoryTagType(row.category)" size="small">
              {{ getCategoryLabel(row.category) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="质量等级" width="90">
          <template #default="{ row }">
            <el-tag :type="getQualityTagType(row.quality)" size="small">
              {{ getQualityLabel(row.quality) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="confidence" label="置信度" width="90" sortable>
          <template #default="{ row }">
            <el-progress
              :percentage="row.confidence"
              :color="getConfidenceColor(row.confidence)"
              :stroke-width="6"
            />
          </template>
        </el-table-column>
        <el-table-column label="缺陷信息" width="130">
          <template #default="{ row }">
            <div v-if="row.defects && row.defects.length > 0">
              <el-tag
                v-for="defect in row.defects.slice(0, 2)"
                :key="defect"
                type="warning"
                size="small"
                class="defect-tag"
              >
                {{ defect }}
              </el-tag>
              <el-tag
                v-if="row.defects.length > 2"
                size="small"
                class="more-defects"
              >
                +{{ row.defects.length - 2 }}
              </el-tag>
            </div>
            <span v-else class="text-gray-400">无</span>
          </template>
        </el-table-column>
        <el-table-column prop="processingTime" label="处理时间" width="90" sortable>
          <template #default="{ row }">
            {{ row.processingTime }}ms
          </template>
        </el-table-column>
        <el-table-column prop="detectedAt" label="检测时间" width="140" sortable>
          <template #default="{ row }">
            {{ formatDateTime(row.detectedAt) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" min-width="120" align="center" fixed="right">
          <template #default="{ row }">
            <div class="operation-buttons">
              <el-button size="small" @click.stop="viewDetail(row)" title="查看详情">
                <el-icon><View /></el-icon>
              </el-button>
              <el-button size="small" @click.stop="reprocess(row)" title="重新处理">
                <el-icon><RefreshRight /></el-icon>
              </el-button>
              <el-button size="small" @click.stop="deleteResult(row)" type="danger" title="删除">
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- Card View -->
      <div v-else class="results-grid">
        <el-card
          v-for="result in results"
          :key="result.id"
          class="result-card"
          shadow="hover"
          @click="handleRowClick(result)"
        >
          <div class="result-card-content">
            <div class="result-image">
              <el-image
                :src="result.imageUrl"
                :preview-src-list="[result.imageUrl]"
                fit="cover"
                class="w-full h-full"
              >
                <template #error>
                  <div class="image-slot">
                    <el-icon><Picture /></el-icon>
                  </div>
                </template>
              </el-image>
              <div class="result-overlay">
                <el-checkbox
                  :model-value="selectedResults.includes(result)"
                  @change="toggleSelection(result)"
                  @click.stop
                />
              </div>
            </div>
            <div class="result-info">
              <div class="result-meta">
                <span class="batch-id">{{ result.batchId }}</span>
                <span class="item-id">{{ result.itemId }}</span>
              </div>
              <div class="result-classification">
                <el-tag :type="getCategoryTagType(result.category)" size="small">
                  {{ getCategoryLabel(result.category) }}
                </el-tag>
                <el-tag :type="getQualityTagType(result.quality)" size="small">
                  {{ getQualityLabel(result.quality) }}
                </el-tag>
              </div>
              <div class="result-stats">
                <div class="confidence-info">
                  <span class="label">置信度:</span>
                  <el-progress
                    :percentage="result.confidence"
                    :color="getConfidenceColor(result.confidence)"
                    :stroke-width="4"
                    :show-text="false"
                  />
                  <span class="value">{{ result.confidence }}%</span>
                </div>
                <div class="time-info">
                  <span class="label">处理:</span>
                  <span class="value">{{ result.processingTime }}ms</span>
                </div>
              </div>
              <div class="result-defects" v-if="result.defects && result.defects.length > 0">
                <el-tag
                  v-for="defect in result.defects.slice(0, 3)"
                  :key="defect"
                  type="warning"
                  size="small"
                  class="defect-tag"
                >
                  {{ defect }}
                </el-tag>
              </div>
              <div class="result-actions">
                <el-button size="small" @click.stop="viewDetail(result)">
                  <el-icon><View /></el-icon>
                  详情
                </el-button>
                <el-button size="small" @click.stop="reprocess(result)">
                  <el-icon><RefreshRight /></el-icon>
                  重处理
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- Pagination -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.currentPage"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :small="false"
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
      title="检测结果详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <div v-if="currentResult" class="result-detail">
        <div class="detail-grid">
          <div class="detail-image">
            <el-image
              :src="currentResult.imageUrl"
              :preview-src-list="[currentResult.imageUrl]"
              fit="contain"
              class="detail-img"
            />
          </div>
          <div class="detail-info">
            <el-descriptions :column="2" border>
              <el-descriptions-item label="批次ID">{{ currentResult.batchId }}</el-descriptions-item>
              <el-descriptions-item label="物品ID">{{ currentResult.itemId }}</el-descriptions-item>
              <el-descriptions-item label="分类结果">
                <el-tag :type="getCategoryTagType(currentResult.category)">
                  {{ getCategoryLabel(currentResult.category) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="质量等级">
                <el-tag :type="getQualityTagType(currentResult.quality)">
                  {{ getQualityLabel(currentResult.quality) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="置信度">{{ currentResult.confidence }}%</el-descriptions-item>
              <el-descriptions-item label="处理时间">{{ currentResult.processingTime }}ms</el-descriptions-item>
              <el-descriptions-item label="检测时间" :span="2">
                {{ formatDateTime(currentResult.detectedAt) }}
              </el-descriptions-item>
            </el-descriptions>

            <div v-if="currentResult.defects && currentResult.defects.length > 0" class="mt-4">
              <h4 class="text-sm font-semibold mb-2">检测到的缺陷:</h4>
              <div class="defects-list">
                <el-tag
                  v-for="defect in currentResult.defects"
                  :key="defect"
                  type="warning"
                  class="mr-1 mb-1"
                >
                  {{ defect }}
                </el-tag>
              </div>
            </div>

            <div v-if="currentResult.metadata" class="mt-4">
              <h4 class="text-sm font-semibold mb-2">其他信息:</h4>
              <el-table :data="Object.entries(currentResult.metadata)" size="small">
                <el-table-column prop="0" label="属性" width="120" />
                <el-table-column prop="1" label="值" />
              </el-table>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="detailVisible = false">关闭</el-button>
          <el-button type="primary" @click="reprocess(currentResult)">重新处理</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
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
const categoryOptions = ref([
  { label: '苹果', value: 'apple' },
  { label: '香蕉', value: 'banana' },
  { label: '橙子', value: 'orange' },
  { label: '葡萄', value: 'grape' },
  { label: '草莓', value: 'strawberry' },
  { label: '西瓜', value: 'watermelon' },
  { label: '菠萝', value: 'pineapple' },
  { label: '芒果', value: 'mango' },
  { label: '猕猴桃', value: 'kiwi' },
  { label: '梨', value: 'pear' }
]);

// 质量等级选项
const qualityOptions = ref([
  { label: '优质', value: 'excellent' },
  { label: '良好', value: 'good' },
  { label: '一般', value: 'fair' },
  { label: '不合格', value: 'poor' }
]);

interface DetectionResult {
  id: string
  batchId: string
  itemId: string
  category: string
  quality: 'excellent' | 'good' | 'fair' | 'poor'
  confidence: number
  defects?: string[]
  processingTime: number
  imageUrl: string
  detectedAt: string
  metadata?: Record<string, any>
}

interface SearchForm {
  batchId: string
  category: string
  quality: string
  confidenceRange: [number, number]
  dateRange: [string, string] | null
  hasDefects: boolean
}

interface Statistics {
  total: number
  qualified: number
  unqualified: number
  avgConfidence: number
}

interface Pagination {
  currentPage: number
  pageSize: number
  total: number
}

// 搜索表单数据
const searchForm = reactive<SearchForm>({
  batchId: '',
  category: '',
  quality: '',
  confidenceRange: [0, 100],
  dateRange: null,
  hasDefects: false
})

const results = ref<DetectionResult[]>([])
const selectedResults = ref<DetectionResult[]>([])
const currentResult = ref<DetectionResult | null>(null)
const loading = ref(false)
const detailVisible = ref(false)
const viewMode = ref<'table' | 'card'>('table')

const pagination = reactive<Pagination>({
  currentPage: 1,
  pageSize: 20,
  total: 0
})

const statistics = computed<Statistics>(() => ({
  total: results.value.length,
  qualified: results.value.filter(r => ['excellent', 'good'].includes(r.quality)).length,
  unqualified: results.value.filter(r => ['fair', 'poor'].includes(r.quality)).length,
  avgConfidence: results.value.length > 0
    ? results.value.reduce((sum, r) => sum + r.confidence, 0) / results.value.length
    : 0
}))

const confidenceMarks = {
  0: '0%',
  25: '25%',
  50: '50%',
  75: '75%',
  100: '100%'
}

// 格式化置信度显示
const formatConfidence = (value: number) => {
  return `${value}%`;
}

// 搜索处理
const handleSearch = async () => {
  loading.value = true
  try {
    // 模拟API调用延迟
    await new Promise(resolve => setTimeout(resolve, 1000))
    await loadResults()
    ElMessage.success('搜索完成')
  } catch (error) {
    ElMessage.error('搜索失败')
  } finally {
    loading.value = false
  }
}

// 重置搜索条件
const resetSearch = () => {
  Object.assign(searchForm, {
    batchId: '',
    category: '',
    quality: '',
    confidenceRange: [0, 100],
    dateRange: null,
    hasDefects: false
  })
  pagination.currentPage = 1
  handleSearch()
}

// 刷新结果
const refreshResults = async () => {
  loading.value = true
  try {
    await loadResults()
    ElMessage.success('数据已刷新')
  } catch (error) {
    ElMessage.error('刷新失败')
  } finally {
    loading.value = false
  }
}

// 加载结果数据
const loadResults = async () => {
  // 基于搜索条件生成模拟数据
  const mockResults: DetectionResult[] = []
  const categories = ['apple', 'orange', 'banana', 'pear', 'grape', 'watermelon', 'other']
  const qualities: DetectionResult['quality'][] = ['excellent', 'good', 'fair', 'poor']
  const defectTypes = ['表面缺陷', '形状异常', '颜色异常', '尺寸不符', '碰伤', '腐烂']

  for (let i = 1; i <= 100; i++) {
    const category = categories[Math.floor(Math.random() * categories.length)]
    const quality = qualities[Math.floor(Math.random() * qualities.length)]
    const confidence = Math.floor(Math.random() * 40) + 60
    const hasDefects = Math.random() > 0.7
    const defects = hasDefects
      ? defectTypes.slice(0, Math.floor(Math.random() * 3) + 1)
      : []

    // 应用搜索筛选条件
    if (searchForm.category && category !== searchForm.category) continue
    if (searchForm.quality && quality !== searchForm.quality) continue
    if (confidence < searchForm.confidenceRange[0] || confidence > searchForm.confidenceRange[1]) continue
    if (searchForm.hasDefects && !hasDefects) continue
    if (searchForm.batchId && !`BATCH${Math.floor(i / 10) + 1}`.includes(searchForm.batchId)) continue

    const detectedAt = new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000)
    if (searchForm.dateRange) {
      const [start, end] = searchForm.dateRange
      if (detectedAt < new Date(start) || detectedAt > new Date(end)) continue
    }

    mockResults.push({
      id: `result_${i}`,
      batchId: `BATCH${Math.floor(i / 10) + 1}`,
      itemId: `ITEM${String(i).padStart(4, '0')}`,
      category,
      quality,
      confidence,
      defects,
      processingTime: Math.floor(Math.random() * 200) + 50,
      imageUrl: `/api/images/result_${i}.jpg`,
      detectedAt: detectedAt.toISOString(),
      metadata: {
        '尺寸': `${Math.floor(Math.random() * 50 + 30)}mm`,
        '重量': `${Math.floor(Math.random() * 200 + 50)}g`,
        '成熟度': `${Math.floor(Math.random() * 100)}%`
      }
    })
  }

  // 应用分页
  const startIndex = (pagination.currentPage - 1) * pagination.pageSize
  const endIndex = startIndex + pagination.pageSize

  results.value = mockResults.slice(startIndex, endIndex)
  pagination.total = mockResults.length
}

// 处理表格选择变化
const handleSelectionChange = (selection: DetectionResult[]) => {
  selectedResults.value = selection
}

// 切换卡片选择状态
const toggleSelection = (result: DetectionResult) => {
  const index = selectedResults.value.findIndex(r => r.id === result.id)
  if (index > -1) {
    selectedResults.value.splice(index, 1)
  } else {
    selectedResults.value.push(result)
  }
}

// 行点击事件
const handleRowClick = (result: DetectionResult) => {
  viewDetail(result)
}

// 查看详情
const viewDetail = (result: DetectionResult) => {
  currentResult.value = result
  detailVisible.value = true
}

// 重新处理
const reprocess = async (result: DetectionResult | null) => {
  if (!result) return;
  
  try {
    await ElMessageBox.confirm(
      `确定要重新处理 ${result.itemId} 吗？`,
      '确认操作',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    // 模拟重新处理
    ElMessage.info('正在重新处理...')
    await new Promise(resolve => setTimeout(resolve, 2000))

    // 更新结果数据
    const index = results.value.findIndex(r => r.id === result.id)
    if (index > -1) {
      results.value[index] = {
        ...result,
        confidence: Math.floor(Math.random() * 40) + 60,
        processingTime: Math.floor(Math.random() * 200) + 50,
        detectedAt: new Date().toISOString()
      }
    }

    ElMessage.success('重新处理完成')
    detailVisible.value = false
  } catch (error) {
    // 用户取消操作
  }
}

// 删除结果
const deleteResult = async (result: DetectionResult) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除 ${result.itemId} 的检测结果吗？`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    const index = results.value.findIndex(r => r.id === result.id)
    if (index > -1) {
      results.value.splice(index, 1)
      pagination.total--
    }

    ElMessage.success('删除成功')
  } catch (error) {
    // 用户取消操作
  }
}

// 批量操作处理
const handleBatchAction = async (command: string) => {
  if (selectedResults.value.length === 0) {
    ElMessage.warning('请先选择要操作的结果')
    return
  }

  try {
    switch (command) {
      case 'export':
        await exportResults(selectedResults.value)
        break
      case 'delete':
        await ElMessageBox.confirm(
          `确定要删除选中的 ${selectedResults.value.length} 个结果吗？`,
          '批量删除',
          {
            confirmButtonText: '删除',
            cancelButtonText: '取消',
            type: 'warning'
          }
        )
        selectedResults.value.forEach(result => {
          const index = results.value.findIndex(r => r.id === result.id)
          if (index > -1) {
            results.value.splice(index, 1)
          }
        })
        pagination.total -= selectedResults.value.length
        selectedResults.value = []
        ElMessage.success('批量删除成功')
        break
      case 'reprocess':
        await ElMessageBox.confirm(
          `确定要重新处理选中的 ${selectedResults.value.length} 个结果吗？`,
          '批量重处理',
          {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
          }
        )
        ElMessage.info('正在批量重新处理...')
        // 模拟批量处理
        await new Promise(resolve => setTimeout(resolve, 3000))
        selectedResults.value = []
        ElMessage.success('批量重处理完成')
        await loadResults()
        break
    }
  } catch (error) {
    // 用户取消操作
  }
}

// 导出结果
const exportResults = async (resultsToExport = results.value) => {
  try {
    // 确保 resultsToExport 是一个数组
    const dataToExport = Array.isArray(resultsToExport) ? resultsToExport : results.value;
    
    const csvContent = [
      '批次ID,物品ID,分类结果,质量等级,置信度,处理时间,检测时间,缺陷信息',
      ...dataToExport.map(result => [
        result.batchId || '',
        result.itemId || '',
        getCategoryLabel(result.category || ''),
        getQualityLabel(result.quality || ''),
        `${result.confidence || 0}%`,
        `${result.processingTime || 0}ms`,
        formatDateTime(result.detectedAt || new Date().toISOString()),
        result.defects && Array.isArray(result.defects) ? result.defects.join(';') : '无'
      ].join(','))
    ].join('\n')

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `detection_results_${new Date().toISOString().slice(0, 10)}.csv`
    link.click()
    URL.revokeObjectURL(link.href)

    ElMessage.success('导出成功')
  } catch (error) {
    ElMessage.error('导出失败')
  }
}

// 分页大小变化
const handleSizeChange = (newSize: number) => {
  pagination.pageSize = newSize
  pagination.currentPage = 1
  loadResults()
}

// 页码变化
const handleCurrentChange = (newPage: number) => {
  pagination.currentPage = newPage
  loadResults()
}

// 辅助函数：获取分类标签文本
const getCategoryLabel = (category: string): string => {
  const labels: Record<string, string> = {
    apple: '苹果',
    orange: '橙子',
    banana: '香蕉',
    pear: '梨',
    grape: '葡萄',
    watermelon: '西瓜',
    other: '其他'
  }
  return labels[category] || category
}

// 辅助函数：获取分类标签类型
const getCategoryTagType = (category: string): string => {
  const types: Record<string, string> = {
    apple: 'success',
    orange: 'warning',
    banana: 'info',
    pear: 'success',
    grape: 'danger',
    watermelon: 'warning',
    other: ''
  }
  return types[category] || ''
}

// 辅助函数：获取质量等级标签文本
const getQualityLabel = (quality: string): string => {
  const labels: Record<string, string> = {
    excellent: '优质',
    good: '良好',
    fair: '一般',
    poor: '不合格'
  }
  return labels[quality] || quality
}

// 辅助函数：获取质量等级标签类型
const getQualityTagType = (quality: string): string => {
  const types: Record<string, string> = {
    excellent: 'success',
    good: 'info',
    fair: 'warning',
    poor: 'danger'
  }
  return types[quality] || ''
}

// 辅助函数：获取置信度颜色
const getConfidenceColor = (confidence: number): string => {
  if (confidence >= 90) return '#67c23a'
  if (confidence >= 80) return '#409eff'
  if (confidence >= 70) return '#e6a23c'
  return '#f56c6c'
}

// 辅助函数：格式化日期时间
const formatDateTime = (dateString: string): string => {
  return dayjs(dateString).format('YYYY-MM-DD HH:mm:ss')
}

// 组件挂载时加载数据
onMounted(() => {
  loadResults()
})
</script>

<style scoped>
.results-view {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

.results-header {
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

.search-card {
  margin-bottom: 20px;
  border-radius: 8px;
}

.search-form {
  width: 100%;
}

.confidence-slider {
  width: 100%;
  margin-top: 8px;
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

.results-card {
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

.result-thumbnail {
  width: 60px;
  height: 60px;
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

.defect-tag {
  margin-right: 4px;
  margin-bottom: 2px;
}

.more-defects {
  margin-left: 4px;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.result-card {
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 8px;
}

.result-card:hover {
  transform: translateY(-2px);
}

.result-card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-image {
  position: relative;
  width: 100%;
  height: 180px;
  border-radius: 6px;
  overflow: hidden;
}

.result-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
}

.result-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #909399;
}

.result-classification {
  display: flex;
  gap: 8px;
}

.result-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.confidence-info,
.time-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.confidence-info .el-progress {
  flex: 1;
}

.label {
  color: #606266;
  width: 50px;
}

.value {
  color: #303133;
  font-weight: 500;
}

.result-defects {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.result-actions {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.pagination-wrapper {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}

.result-detail {
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
}

.detail-img {
  max-width: 100%;
  max-height: 400px;
  border-radius: 8px;
}

.defects-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .results-view {
    padding: 12px;
  }

  .results-header {
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

  .results-grid {
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

:deep(.el-table__body-wrapper) {
  max-height: 600px;
}

:deep(.el-progress-bar) {
  flex: 1;
}

.operation-buttons {
  display: flex;
  justify-content: space-around;
  align-items: center;
  gap: 8px;
}

:deep(.el-table) {
  table-layout: fixed;
  width: 100% !important;
  margin: 0;
}

:deep(.el-table__cell) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 8px 0;
}

:deep(.el-table .cell) {
  padding: 0 8px;
}

.result-thumbnail {
  width: 50px;
  height: 50px;
  border-radius: 4px;
}

.defect-tag {
  margin: 2px;
}

:deep(.el-button--small) {
  padding: 6px;
  min-height: 28px;
}

/* 搜索表单样式优化 */
.search-form {
  padding: 10px 0;
}

.search-form :deep(.el-form-item) {
  margin-bottom: 18px;
  width: 100%;
}

.search-form :deep(.el-form-item__label) {
  padding-right: 12px;
  font-weight: 500;
}

.search-row {
  margin-bottom: 10px;
}

.confidence-form-item :deep(.el-form-item__content) {
  padding: 0 10px;
}

.date-form-item :deep(.el-date-editor) {
  width: 100%;
}

.defect-form-item :deep(.el-switch__label) {
  margin: 0 4px;
}

.search-buttons :deep(.el-form-item__content) {
  display: flex;
  justify-content: flex-start;
  gap: 10px;
}

:deep(.el-button--small) {
  min-height: 28px;
}
</style>