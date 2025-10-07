<template>
  <div class="data-dashboard">
    <!-- Header Stats Cards -->
    <div class="stats-cards grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
      <el-card class="stat-card" shadow="hover">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon size="24" color="#67c23a"><Checked /></el-icon>
          </div>
          <div class="stat-info">
            <h3 class="stat-value">{{ totalProcessed.toLocaleString() }}</h3>
            <p class="stat-label">总处理数量</p>
            <p class="stat-change positive">+{{ dailyIncrease.processed }}%</p>
          </div>
        </div>
      </el-card>

      <el-card class="stat-card" shadow="hover">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon size="24" color="#409eff"><DataAnalysis /></el-icon>
          </div>
          <div class="stat-info">
            <h3 class="stat-value">{{ accuracyRate }}%</h3>
            <p class="stat-label">识别准确率</p>
            <p class="stat-change positive">+{{ dailyIncrease.accuracy }}%</p>
          </div>
        </div>
      </el-card>

      <el-card class="stat-card" shadow="hover">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon size="24" color="#e6a23c"><Warning /></el-icon>
          </div>
          <div class="stat-info">
            <h3 class="stat-value">{{ defectiveCount.toLocaleString() }}</h3>
            <p class="stat-label">缺陷果蔬</p>
            <p class="stat-change negative">+{{ dailyIncrease.defective }}%</p>
          </div>
        </div>
      </el-card>

      <el-card class="stat-card" shadow="hover">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon size="24" color="#f56c6c"><Timer /></el-icon>
          </div>
          <div class="stat-info">
            <h3 class="stat-value">{{ avgProcessingTime }}ms</h3>
            <p class="stat-label">平均处理时间</p>
            <p class="stat-change positive">-{{ dailyIncrease.processingTime }}%</p>
          </div>
        </div>
      </el-card>
    </div>

    <!-- Charts Grid -->
    <div class="charts-grid grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
      <!-- Processing Volume Trend -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3 class="text-lg font-semibold">处理量趋势</h3>
            <el-radio-group v-model="volumeTimeRange" size="small" @change="updateVolumeChart">
              <el-radio-button label="24h">24小时</el-radio-button>
              <el-radio-button label="7d">7天</el-radio-button>
              <el-radio-button label="30d">30天</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        <div class="chart-container">
          <v-chart
            :option="volumeChartOption"
            :loading="volumeChartLoading"
            autoresize
            style="height: 300px"
          />
        </div>
      </el-card>

      <!-- Accuracy Rate Chart -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3 class="text-lg font-semibold">识别准确率</h3>
            <el-radio-group v-model="accuracyTimeRange" size="small" @change="updateAccuracyChart">
              <el-radio-button label="24h">24小时</el-radio-button>
              <el-radio-button label="7d">7天</el-radio-button>
              <el-radio-button label="30d">30天</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        <div class="chart-container">
          <v-chart
            :option="accuracyChartOption"
            :loading="accuracyChartLoading"
            autoresize
            style="height: 300px"
          />
        </div>
      </el-card>
    </div>

    <!-- Category Distribution and Real-time Stats -->
    <div class="charts-grid grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
      <!-- Category Distribution -->
      <el-card class="chart-card lg:col-span-2" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3 class="text-lg font-semibold">分类分布</h3>
            <el-select v-model="categoryTimeRange" size="small" @change="updateCategoryChart">
              <el-option label="今日" value="today" />
              <el-option label="本周" value="week" />
              <el-option label="本月" value="month" />
            </el-select>
          </div>
        </template>
        <div class="chart-container">
          <v-chart
            :option="categoryChartOption"
            :loading="categoryChartLoading"
            autoresize
            style="height: 350px"
          />
        </div>
      </el-card>

      <!-- Real-time Processing -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <h3 class="text-lg font-semibold">实时状态</h3>
        </template>
        <div class="realtime-stats">
          <div class="realtime-item">
            <div class="realtime-label">当前处理速度</div>
            <div class="realtime-value">{{ currentProcessingSpeed }}/分钟</div>
            <el-progress
              :percentage="processingSpeedPercentage"
              :color="progressColors"
              :stroke-width="8"
              class="mt-2"
            />
          </div>

          <div class="realtime-item mt-4">
            <div class="realtime-label">设备状态</div>
            <div class="device-status">
              <div
                v-for="device in deviceStatus"
                :key="device.id"
                class="device-item"
                :class="device.status"
              >
                <el-icon><Monitor /></el-icon>
                <span>{{ device.name }}</span>
                <el-tag
                  :type="device.status === 'online' ? 'success' : 'danger'"
                  size="small"
                >
                  {{ device.status === 'online' ? '在线' : '离线' }}
                </el-tag>
              </div>
            </div>
          </div>

          <div class="realtime-item mt-4">
            <div class="realtime-label">系统负载</div>
            <div class="system-load">
              <div class="load-item">
                <span>CPU</span>
                <el-progress :percentage="systemLoad.cpu" size="small" />
              </div>
              <div class="load-item">
                <span>内存</span>
                <el-progress :percentage="systemLoad.memory" size="small" />
              </div>
              <div class="load-item">
                <span>GPU</span>
                <el-progress :percentage="systemLoad.gpu" size="small" />
              </div>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- Quality Analysis and Defect Trends -->
    <div class="charts-grid grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Quality Distribution -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3 class="text-lg font-semibold">质量分析</h3>
            <el-button size="small" @click="refreshQualityData">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </template>
        <div class="chart-container">
          <v-chart
            :option="qualityChartOption"
            :loading="qualityChartLoading"
            autoresize
            style="height: 300px"
          />
        </div>
      </el-card>

      <!-- Defect Types -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3 class="text-lg font-semibold">缺陷类型统计</h3>
            <el-button size="small" @click="exportDefectData">
              <el-icon><Download /></el-icon>
              导出
            </el-button>
          </div>
        </template>
        <div class="chart-container">
          <v-chart
            :option="defectChartOption"
            :loading="defectChartLoading"
            autoresize
            style="height: 300px"
          />
        </div>
      </el-card>
    </div>

    <!-- Footer Info -->
    <div class="footer-info mt-6 text-center text-sm text-gray-500">
      <p>数据更新时间: {{ lastUpdateTime }}</p>
      <p>系统运行时间: {{ systemUptime }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart, PieChart, GaugeChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { ElMessage } from 'element-plus'
import {
  Checked,
  DataAnalysis,
  Warning,
  Timer,
  Monitor,
  Refresh,
  Download
} from '@element-plus/icons-vue'

// Register ECharts components
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  PieChart,
  GaugeChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent
])

interface DeviceStatus {
  id: string
  name: string
  status: 'online' | 'offline'
}

interface SystemLoad {
  cpu: number
  memory: number
  gpu: number
}

interface ChartData {
  time: string[]
  values: number[]
}

// Reactive data
const totalProcessed = ref(156789)
const accuracyRate = ref(98.5)
const defectiveCount = ref(2341)
const avgProcessingTime = ref(125)

const dailyIncrease = reactive({
  processed: 12.5,
  accuracy: 2.1,
  defective: 5.3,
  processingTime: 8.7
})

const currentProcessingSpeed = ref(240)
const processingSpeedPercentage = ref(75)

const deviceStatus = ref<DeviceStatus[]>([
  { id: '1', name: '分拣机 #1', status: 'online' },
  { id: '2', name: '分拣机 #2', status: 'online' },
  { id: '3', name: '相机 #1', status: 'online' },
  { id: '4', name: '相机 #2', status: 'offline' }
])

const systemLoad = reactive<SystemLoad>({
  cpu: 65,
  memory: 72,
  gpu: 58
})

const lastUpdateTime = ref(new Date().toLocaleString())
const systemUptime = ref('7天 12小时 34分钟')

// Chart time ranges
const volumeTimeRange = ref('24h')
const accuracyTimeRange = ref('24h')
const categoryTimeRange = ref('today')

// Chart loading states
const volumeChartLoading = ref(false)
const accuracyChartLoading = ref(false)
const categoryChartLoading = ref(false)
const qualityChartLoading = ref(false)
const defectChartLoading = ref(false)

// Progress colors
const progressColors = [
  { color: 'var(--color-error)', percentage: 20 },
  { color: 'var(--color-warning)', percentage: 40 },
  { color: 'var(--color-success)', percentage: 60 },
  { color: 'var(--color-accent)', percentage: 80 },
  { color: 'var(--color-primary)', percentage: 100 }
]

// Chart options
const volumeChartOption = ref({
  title: {
    text: ''
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross',
      label: {
        backgroundColor: 'var(--color-text-secondary)'
      }
    }
  },
  legend: {
    data: ['处理量', '合格数', '不合格数'],
    textStyle: {
      color: 'var(--color-text-secondary)'
    }
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: [] as string[],
    axisLabel: {
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    }
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    },
    splitLine: {
      lineStyle: {
        color: 'var(--color-background-medium)'
      }
    }
  },
  series: [
    {
      name: '处理量',
      type: 'line',
      stack: 'Total',
      data: [] as number[],
      smooth: true,
      lineStyle: { color: 'var(--color-accent)' },
      areaStyle: { color: 'rgba(0, 122, 255, 0.1)' } // Using rgba for accent color with opacity
    },
    {
      name: '合格数',
      type: 'line',
      stack: 'Total',
      data: [] as number[],
      smooth: true,
      lineStyle: { color: 'var(--color-success)' },
      areaStyle: { color: 'rgba(40, 167, 69, 0.1)' } // Using rgba for success color with opacity
    },
    {
      name: '不合格数',
      type: 'line',
      stack: 'Total',
      data: [] as number[],
      smooth: true,
      lineStyle: { color: 'var(--color-error)' },
      areaStyle: { color: 'rgba(220, 53, 69, 0.1)' } // Using rgba for error color with opacity
    }
  ]
})

const accuracyChartOption = ref({
  tooltip: {
    trigger: 'axis',
    formatter: function(params: any) {
      return `${params[0].name}<br/>准确率: ${params[0].value}%`
    }
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: [] as string[],
    axisLabel: {
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    }
  },
  yAxis: {
    type: 'value',
    min: 90,
    max: 100,
    axisLabel: {
      formatter: '{value}%',
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    },
    splitLine: {
      lineStyle: {
        color: 'var(--color-background-medium)'
      }
    }
  },
  series: [
    {
      name: '准确率',
      type: 'line',
      data: [] as number[],
      smooth: true,
      lineStyle: { color: 'var(--color-primary)', width: 3 },
      symbol: 'circle',
      symbolSize: 6,
      itemStyle: {
        color: 'var(--color-primary)'
      },
      markLine: {
        data: [
          { type: 'average', name: '平均值' },
          { yAxis: 95, name: '目标值', lineStyle: { color: 'var(--color-success)' } }
        ],
        label: {
          color: 'var(--color-text-secondary)'
        }
      }
    }
  ]
})

const categoryChartOption = ref({
  tooltip: {
    trigger: 'item',
    formatter: function(params: any) {
      return `${params.name}<br/>数量: ${params.value}<br/>占比: ${params.percent}%`
    }
  },
  legend: {
    orient: 'vertical',
    left: 'left',
    textStyle: {
      color: 'var(--color-text-primary)' // 将图例字体颜色改为深色系
    }
  },
  series: [
    {
      name: '分类分布',
      type: 'pie',
      radius: [0, '70%'],
      avoidLabelOverlap: false,
      label: {
        show: true,
        position: 'outside',
        formatter: '{b}: {d}%',
        color: 'var(--color-text-primary)'
      },
      labelLine: {
        show: true,
        length: 10,
        length2: 15,
        lineStyle: {
          color: 'var(--color-text-secondary)'
        }
      },
      data: [] as Array<{ name: string; value: number }>,
      itemStyle: {
        borderRadius: 5,
        borderColor: 'var(--color-white)',
        borderWidth: 2
      },
      color: [
        '#FF0000', // Red (苹果)
        '#FF8C00', // DarkOrange (替换 var(--color-accent) for 橙子)
        '#B8860B', // DarkGoldenRod (替换 #FFD700 Gold for 香蕉)
        '#8A2BE2', // BlueViolet
        '#FF6347', // Tomato
        '#20B2AA', // LightSeaGreen
        '#6A5ACD' // SlateBlue
      ]
    }
  ]
})

const qualityChartOption = ref({
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'shadow'
    }
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    data: ['优质', '良好', '一般', '不合格'],
    axisLabel: {
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    }
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      color: 'var(--color-text-placeholder)'
    },
    axisLine: {
      lineStyle: {
        color: 'var(--color-border)'
      }
    },
    splitLine: {
      lineStyle: {
        color: 'var(--color-background-medium)'
      }
    }
  },
  series: [
    {
      name: '数量',
      type: 'bar',
      data: [12500, 8900, 3400, 1200],
      itemStyle: {
        borderRadius: [5, 5, 0, 0], // Rounded corners for bars
        color: function(params: any) {
        const colors = [
          '#36B37E', // 优质 (深绿，更沉稳)
          '#2563EB', // 良好 (深蓝，降低饱和度)
          '#FFAB00', // 一般 (橙黄，减少刺眼感)
          '#E53E3E'  // 不合格 (深红，降低明度)
        ]
          return colors[params.dataIndex]
        }
      }
    }
  ]
})

const defectChartOption = ref({
  tooltip: {
    trigger: 'item',
    formatter: '{a} <br/>{b}: {c} ({d}%)'
  },
  legend: {
    bottom: 0,
    data: ['表面缺陷', '形状异常', '颜色异常', '尺寸不符', '其他'],
    textStyle: {
      color: 'var(--color-text-secondary)'
    }
  },
  series: [
    {
      name: '缺陷类型',
      type: 'pie',
      radius: [0, '70%'],
      data: [
        { value: 890, name: '表面缺陷' },
        { value: 680, name: '形状异常' },
        { value: 456, name: '颜色异常' },
        { value: 235, name: '尺寸不符' },
        { value: 80, name: '其他' }
      ],
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowOffsetX: 0,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      },
      itemStyle: {
        borderRadius: 5,
        borderColor: 'var(--color-white)',
        borderWidth: 2
      },
      color: [
        '#8B0000', // 表面缺陷 (DarkRed)
        '#483D8B', // 形状异常 (DarkSlateBlue)
        '#DAA520', // 颜色异常 (Goldenrod)
        '#2F4F4F', // 尺寸不符 (DarkSlateGray)
        '#696969'  // 其他 (DimGray)
      ],
      label: { // 添加标签配置，确保字体颜色不是白色
        show: true,
        position: 'outside',
        formatter: '{b}: {d}%',
        color: 'var(--color-text-primary)' // 使用深色字体
      }
    }
  ]
})

// Methods
const updateVolumeChart = async (timeRange: string) => {
  volumeChartLoading.value = true
  try {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000))

    const mockData = generateMockTimeSeriesData(timeRange)
    volumeChartOption.value.xAxis.data = mockData.time
    volumeChartOption.value.series[0].data = mockData.values.map(v => v * 1.2)
    volumeChartOption.value.series[1].data = mockData.values
    volumeChartOption.value.series[2].data = mockData.values.map(v => Math.floor(v * 0.1))
  } catch (error) {
    ElMessage.error('更新处理量图表失败')
  } finally {
    volumeChartLoading.value = false
  }
}

const updateAccuracyChart = async (timeRange: string) => {
  accuracyChartLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 800))

    const mockData = generateMockTimeSeriesData(timeRange)
    accuracyChartOption.value.xAxis.data = mockData.time
    accuracyChartOption.value.series[0].data = mockData.values.map(v =>
      Math.min(99.9, Math.max(92, 95 + Math.random() * 4))
    )
  } catch (error) {
    ElMessage.error('更新准确率图表失败')
  } finally {
    accuracyChartLoading.value = false
  }
}

const updateCategoryChart = async (timeRange: string) => {
  categoryChartLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 600))

    const categories = [
      { name: '苹果', value: 4500 },
      { name: '橙子', value: 3200 },
      { name: '香蕉', value: 2800 },
      { name: '梨', value: 2100 },
      { name: '葡萄', value: 1800 },
      { name: '西瓜', value: 1500 },
      { name: '其他', value: 1200 }
    ]

    categoryChartOption.value.series[0].data = categories
  } catch (error) {
    ElMessage.error('更新分类图表失败')
  } finally {
    categoryChartLoading.value = false
  }
}

const refreshQualityData = async () => {
  qualityChartLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 500))

    // Update quality data with random values
    const qualityData = [
      Math.floor(Math.random() * 5000) + 10000,
      Math.floor(Math.random() * 3000) + 6000,
      Math.floor(Math.random() * 2000) + 2000,
      Math.floor(Math.random() * 1000) + 500
    ]

    qualityChartOption.value.series[0].data = qualityData
    ElMessage.success('质量数据已刷新')
  } catch (error) {
    ElMessage.error('刷新质量数据失败')
  } finally {
    qualityChartLoading.value = false
  }
}

const exportDefectData = () => {
  try {
    const csvContent = [
      '缺陷类型,数量,占比',
      '表面缺陷,890,38.0%',
      '形状异常,680,29.1%',
      '颜色异常,456,19.5%',
      '尺寸不符,235,10.0%',
      '其他,80,3.4%'
    ].join('\n')

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `defect_data_${new Date().toISOString().slice(0, 10)}.csv`
    link.click()
    URL.revokeObjectURL(link.href)

    ElMessage.success('缺陷数据已导出')
  } catch (error) {
    ElMessage.error('导出数据失败')
  }
}

const generateMockTimeSeriesData = (timeRange: string) => {
  const now = new Date()
  const data: ChartData = { time: [], values: [] }

  let points = 24
  let interval = 60 * 60 * 1000 // 1 hour

  if (timeRange === '7d') {
    points = 7
    interval = 24 * 60 * 60 * 1000 // 1 day
  } else if (timeRange === '30d') {
    points = 30
    interval = 24 * 60 * 60 * 1000 // 1 day
  }

  for (let i = points - 1; i >= 0; i--) {
    const time = new Date(now.getTime() - i * interval)
    data.time.push(
      timeRange === '24h'
        ? time.getHours().toString().padStart(2, '0') + ':00'
        : time.toISOString().slice(5, 10)
    )
    data.values.push(Math.floor(Math.random() * 1000) + 500)
  }

  return data
}

const updateStats = () => {
  // Simulate real-time stat updates
  totalProcessed.value += Math.floor(Math.random() * 10)
  accuracyRate.value = Math.min(99.9, Math.max(95, accuracyRate.value + (Math.random() - 0.5) * 0.1))
  defectiveCount.value += Math.floor(Math.random() * 3)
  avgProcessingTime.value = Math.max(80, avgProcessingTime.value + (Math.random() - 0.5) * 5)

  currentProcessingSpeed.value = Math.floor(Math.random() * 50) + 200
  processingSpeedPercentage.value = Math.floor((currentProcessingSpeed.value / 300) * 100)

  // Update system load
  systemLoad.cpu = Math.max(10, Math.min(95, systemLoad.cpu + (Math.random() - 0.5) * 5))
  systemLoad.memory = Math.max(10, Math.min(95, systemLoad.memory + (Math.random() - 0.5) * 3))
  systemLoad.gpu = Math.max(10, Math.min(95, systemLoad.gpu + (Math.random() - 0.5) * 4))

  lastUpdateTime.value = new Date().toLocaleString()
}

// Lifecycle
let statsUpdateInterval: number | null = null

onMounted(async () => {
  // Initialize charts with default data
  await Promise.all([
    updateVolumeChart(volumeTimeRange.value),
    updateAccuracyChart(accuracyTimeRange.value),
    updateCategoryChart(categoryTimeRange.value)
  ])

  // Start real-time updates
  statsUpdateInterval = window.setInterval(updateStats, 5000)
})

onUnmounted(() => {
  if (statsUpdateInterval) {
    clearInterval(statsUpdateInterval)
  }
})
</script>

<style scoped>
.data-dashboard {
  width: 100%;
  padding: 20px;
  min-height: 100vh;
  background-color: var(--color-background-light); /* Use global background color */
}

.stats-cards {
  margin-bottom: 24px;
}

.stat-card {
  border: 1px solid var(--color-border); /* Consistent border */
  border-radius: 12px; /* Consistent border-radius */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05); /* Subtle shadow */
  transition: all 0.3s ease;
  background-color: var(--color-white); /* Consistent background */
}

.stat-card:hover {
  transform: translateY(-3px); /* Slightly more pronounced hover */
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-icon {
  flex-shrink: 0;
  padding: 12px;
  border-radius: 8px;
  background: var(--color-background-medium); /* Use medium background for icon area */
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  margin: 0 0 4px 0;
  color: var(--color-text-primary); /* Use primary text color */
}

.stat-label {
  font-size: 14px;
  margin: 0 0 4px 0;
  color: var(--color-text-secondary); /* Use secondary text color */
}

.stat-change {
  font-size: 12px;
  font-weight: 500;
}

.stat-change.positive {
  color: var(--color-success); /* Use success color */
}

.stat-change.negative {
  color: var(--color-error); /* Use error color */
}

.charts-grid {
  margin-bottom: 24px;
}

.chart-card {
  border: 1px solid var(--color-border); /* Consistent border */
  border-radius: 12px; /* Consistent border-radius */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05); /* Subtle shadow */
  background-color: var(--color-white); /* Consistent background */
}

.card-header {
  display: flex;
  justify-content: space-between; /* Changed to space-between for better alignment */
  align-items: center;
  gap: 12px;
  padding-bottom: 16px; /* Add padding to header bottom */
  border-bottom: 1px solid var(--color-border); /* Add subtle border */
  margin-bottom: 16px; /* Add margin below border */
}

.chart-container {
  width: 100%;
}

.realtime-stats {
  padding: 12px 0;
}

.realtime-item {
  margin-bottom: 20px;
}

.realtime-item:last-child {
  margin-bottom: 0;
}

.realtime-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary); /* Use secondary text color */
  margin-bottom: 8px;
}

.realtime-value {
  font-size: 20px;
  font-weight: bold;
  color: var(--color-text-primary); /* Use primary text color */
}

.device-status {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.device-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  background: var(--color-background-medium); /* Use medium background */
  font-size: 13px;
  color: var(--color-text-primary); /* Primary text color */
}

.device-item.online {
  border-left: 3px solid var(--color-success); /* Use success color */
}

.device-item.offline {
  border-left: 3px solid var(--color-error); /* Use error color */
}

.system-load {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.load-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.load-item span {
  width: 40px;
  font-size: 12px;
  color: var(--color-text-secondary); /* Secondary text color */
}

.footer-info {
  margin-top: 24px;
  padding: 16px;
  background: var(--color-white); /* Consistent background */
  border-radius: 8px;
  border: 1px solid var(--color-border); /* Consistent border */
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03); /* Lighter shadow */
  color: var(--color-text-secondary); /* Secondary text color */
}

/* Responsive design */
@media (max-width: 768px) {
  .data-dashboard {
    padding: 12px;
  }

  .card-header {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .charts-grid {
    grid-template-columns: 1fr;
  }
}

/* ECharts responsive */
:deep(.echarts) {
  width: 100% !important;
}
</style>
