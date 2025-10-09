<template>
  <div class="sort-view">
    <!-- Header -->
    <div class="sort-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><Setting /></el-icon>
          智能分拣控制
        </h1>
        <p class="page-subtitle">实时控制果蔬分拣系统的运行状态</p>
      </div>
      <div class="header-status">
        <el-tag
          :type="systemStatus === 'running' ? 'success' : systemStatus === 'idle' ? 'info' : 'danger'"
          size="large"
          effect="dark"
        >
          <el-icon>
            <VideoPlay v-if="systemStatus === 'running'" />
            <VideoPause v-else-if="systemStatus === 'idle'" />
            <Warning v-else />
          </el-icon>
          {{ getStatusText(systemStatus) }}
        </el-tag>
      </div>
    </div>

    <!-- Main Control Panel -->
    <div class="control-grid">
      <!-- System Control -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>系统控制</h3>
            <el-switch
              v-model="autoMode"
              active-text="自动"
              inactive-text="手动"
              @change="toggleAutoMode"
            />
          </div>
        </template>

        <div class="system-controls">
          <div class="control-row">
            <el-button
              type="success"
              size="large"
              :disabled="systemStatus === 'running'"
              @click="startSystem"
              :loading="isStarting"
            >
              <el-icon><VideoPlay /></el-icon>
              启动系统
            </el-button>
            <el-button
              type="warning"
              size="large"
              :disabled="systemStatus === 'stopped'"
              @click="pauseSystem"
              :loading="isPausing"
            >
              <el-icon><VideoPause /></el-icon>
              暂停系统
            </el-button>
            <el-button
              type="danger"
              size="large"
              :disabled="systemStatus === 'stopped'"
              @click="stopSystem"
              :loading="isStopping"
            >
              <el-icon><SwitchButton /></el-icon>
              停止系统
            </el-button>
          </div>

          <div class="emergency-control mt-4">
            <el-button
              type="danger"
              size="large"
              plain
              @click="emergencyStop"
              class="emergency-btn"
              style="margin-top:7px ;"
            >
              <el-icon><Close /></el-icon>
              紧急停止
            </el-button>
          </div>

          <div class="system-info mt-4">
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">运行时间:</span>
                <span class="info-value">{{ formatRuntime(runtime) }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">处理速度:</span>
                <span class="info-value">{{ processingSpeed }}/分钟</span>
              </div>
              <div class="info-item">
                <span class="info-label">系统温度:</span>
                <span class="info-value" :class="getTemperatureClass(systemTemperature)">
                  {{ systemTemperature }}°C
                </span>
              </div>
              <div class="info-item">
                <span class="info-label">错误计数:</span>
                <span class="info-value" :class="errorCount > 0 ? 'text-red-500' : ''">
                  {{ errorCount }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Conveyor Control -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <h3>传送带控制</h3>
        </template>

        <div class="conveyor-controls">
          <div class="speed-control mb-4">
            <div class="control-label mb-2">传送带速度</div>
            <el-slider
              v-model="conveyorSpeed"
              :min="0"
              :max="100"
              :disabled="!autoMode || systemStatus !== 'running'"
              @change="updateConveyorSpeed"
              show-input
            />
            <div class="speed-presets mt-2">
              <el-button-group size="small">
                <el-button @click="setConveyorSpeed(25)" :disabled="!autoMode">低速</el-button>
                <el-button @click="setConveyorSpeed(50)" :disabled="!autoMode">中速</el-button>
                <el-button @click="setConveyorSpeed(75)" :disabled="!autoMode">高速</el-button>
              </el-button-group>
            </div>
          </div>

          <div class="conveyor-status">
            <div class="status-grid">
              <div class="status-item">
                <div class="status-icon" :class="conveyorStatus">
                  <el-icon><Right /></el-icon>
                </div>
                <div class="status-info">
                  <div class="status-label">传送带 #1</div>
                  <div class="status-value">{{ conveyorSpeed }}%</div>
                </div>
              </div>
              <div class="status-item">
                <div class="status-icon" :class="conveyor2Status">
                  <el-icon><Right /></el-icon>
                </div>
                <div class="status-info">
                  <div class="status-label">传送带 #2</div>
                  <div class="status-value">{{ conveyor2Speed }}%</div>
                </div>
              </div>
            </div>
          </div>

          <div class="conveyor-actions mt-4">
            <el-button
              size="small"
              @click="calibrateConveyor"
              :disabled="systemStatus === 'running'"
            >
              <el-icon><Tools /></el-icon>
              校准传送带
            </el-button>
            <el-button
              size="small"
              type="warning"
              @click="testConveyor"
              :disabled="systemStatus === 'running'"
            >
              <el-icon><Monitor /></el-icon>
              测试运行
            </el-button>
          </div>
        </div>
      </el-card>

      <!-- Sorting Parameters -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>分拣参数</h3>
            <el-button size="small" @click="resetParameters">
              <el-icon><RefreshRight /></el-icon>
              重置
            </el-button>
          </div>
        </template>

        <div class="sorting-parameters">
          <el-form :model="sortingParams" label-width="100px" size="small">
            <el-form-item label="置信度阈值">
              <el-slider
                v-model="sortingParams.confidenceThreshold"
                :min="50"
                :max="99"
                show-input
                input-size="small"
                @change="updateSortingParams"
              />
            </el-form-item>

            <el-form-item label="质量标准">
              <el-select v-model="sortingParams.qualityStandard" @change="updateSortingParams">
                <el-option label="严格" value="strict" />
                <el-option label="标准" value="standard" />
                <el-option label="宽松" value="loose" />
              </el-select>
            </el-form-item>

            <el-form-item label="分拣模式">
              <el-radio-group v-model="sortingParams.sortMode" @change="updateSortingParams">
                <el-radio-button label="category">按类别</el-radio-button>
                <el-radio-button label="quality">按质量</el-radio-button>
                <el-radio-button label="mixed">混合</el-radio-button>
              </el-radio-group>
            </el-form-item>

            <el-form-item label="启用分类">
              <div class="category-checkboxes">
                <el-checkbox-group v-model="sortingParams.enabledCategories" @change="updateSortingParams">
                  <el-checkbox label="apple">苹果</el-checkbox>
                  <el-checkbox label="orange">橙子</el-checkbox>
                  <el-checkbox label="banana">香蕉</el-checkbox>
                  <el-checkbox label="pear">梨</el-checkbox>
                  <el-checkbox label="grape">葡萄</el-checkbox>
                  <el-checkbox label="watermelon">西瓜</el-checkbox>
                </el-checkbox-group>
              </div>
            </el-form-item>

            <el-form-item label="缺陷检测">
              <el-switch
                v-model="sortingParams.defectDetection"
                active-text="开启"
                inactive-text="关闭"
                @change="updateSortingParams"
              />
            </el-form-item>

            <el-form-item label="敏感度" v-if="sortingParams.defectDetection">
              <el-slider
                v-model="sortingParams.defectSensitivity"
                :min="1"
                :max="10"
                :marks="{ 1: '低', 5: '中', 10: '高' }"
                @change="updateSortingParams"
              />
            </el-form-item>
          </el-form>
        </div>
      </el-card>

      <!-- Sorting Recipe Management -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>分拣方案管理</h3>
          </div>
        </template>

        <div class="sorting-recipes">
          <el-form label-width="80px" size="small">
            <el-form-item label="选择方案">
              <el-select v-model="selectedRecipeId" placeholder="选择分拣方案" @change="loadRecipe">
                <el-option
                  v-for="recipe in sortingRecipes"
                  :key="recipe.id"
                  :label="recipe.name"
                  :value="recipe.id"
                />
              </el-select>
            </el-form-item>

            <el-form-item label="新方案名">
              <el-input v-model="newRecipeName" placeholder="输入新方案名称" />
            </el-form-item>

            <el-form-item>
              <el-button type="primary" size="small" @click="saveRecipe">
                <el-icon><Plus /></el-icon>
                保存当前参数为新方案
              </el-button>
              <el-button
                type="info"
                size="small"
                @click="updateRecipe"
                :disabled="selectedRecipeId === 'default'"
              >
                <el-icon><Edit /></el-icon>
                更新当前方案
              </el-button>
              <el-button
                type="danger"
                size="small"
                @click="deleteRecipe"
                :disabled="selectedRecipeId === 'default'"
              >
                <el-icon><Delete /></el-icon>
                删除选定方案
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-card>

      <!-- Device Status -->
      <el-card class="control-card" shadow="hover">
        <template #header>
          <h3>设备状态</h3>
        </template>

        <div class="device-status">
          <div class="devices-grid">
            <div
              v-for="device in devices"
              :key="device.id"
              class="device-item"
              :class="device.status"
            >
              <div class="device-header">
                <el-icon class="device-icon">
                  <Camera v-if="device.type === 'camera'" />
                  <Monitor v-else-if="device.type === 'sorter'" />
                  <Cpu v-else-if="device.type === 'controller'" />
                  <Operation v-else />
                </el-icon>
                <div class="device-info">
                  <div class="device-name">{{ device.name }}</div>
                  <div class="device-type">{{ getDeviceTypeLabel(device.type) }}</div>
                </div>
                <div class="device-status-indicator" :class="device.status">
                  <el-icon>
                    <Check v-if="device.status === 'online'" />
                    <Close v-else-if="device.status === 'offline'" />
                    <Warning v-else />
                  </el-icon>
                </div>
              </div>

              <div class="device-metrics" v-if="device.status === 'online'">
                <div class="metric">
                  <span class="metric-label">负载:</span>
                  <el-progress
                    :percentage="device.load"
                    :stroke-width="4"
                    :color="getLoadColor(device.load)"
                  />
                </div>
                <div class="metric" v-if="device.temperature">
                  <span class="metric-label">温度:</span>
                  <span class="metric-value" :class="getTemperatureClass(device.temperature)">
                    {{ device.temperature }}°C
                  </span>
                </div>
              </div>

              <div class="device-actions">
                <el-button
                  size="small"
                  @click="testDevice(device)"
                  :disabled="systemStatus === 'running'"
                >
                  测试
                </el-button>
                <el-button
                  size="small"
                  type="warning"
                  @click="restartDevice(device)"
                  :disabled="systemStatus === 'running'"
                >
                  重启
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Real-time Monitoring -->
      <el-card class="control-card lg:col-span-2" shadow="hover">
        <template #header>
          <h3>实时监控</h3>
        </template>

        <div class="monitoring-panel">
          <div class="monitoring-grid">
            <!-- Live Stats -->
            <div class="stats-section">
              <h4>实时统计</h4>
              <div class="live-stats">
                <div class="stat-item">
                  <div class="stat-icon success">
                    <el-icon><Check /></el-icon>
                  </div>
                  <div class="stat-content">
                    <div class="stat-value">{{ liveStats.processed }}</div>
                    <div class="stat-label">已处理</div>
                  </div>
                </div>
                <div class="stat-item">
                  <div class="stat-icon info">
                    <el-icon><Histogram /></el-icon>
                  </div>
                  <div class="stat-content">
                    <div class="stat-value">{{ liveStats.sorted }}</div>
                    <div class="stat-label">已分拣</div>
                  </div>
                </div>
                <div class="stat-item">
                  <div class="stat-icon warning">
                    <el-icon><Warning /></el-icon>
                  </div>
                  <div class="stat-content">
                    <div class="stat-value">{{ liveStats.rejected }}</div>
                    <div class="stat-label">已拒绝</div>
                  </div>
                </div>
                <div class="stat-item">
                  <div class="stat-icon danger">
                    <el-icon><Close /></el-icon>
                  </div>
                  <div class="stat-content">
                    <div class="stat-value">{{ liveStats.errors }}</div>
                    <div class="stat-label">错误数</div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Recent Events -->
            <div class="events-section">
              <h4>最近事件</h4>
              <div class="events-list">
                <div
                  v-for="event in recentEvents"
                  :key="event.id"
                  class="event-item"
                  :class="event.type"
                >
                  <div class="event-icon">
                    <el-icon>
                      <Check v-if="event.type === 'success'" />
                      <Warning v-else-if="event.type === 'warning'" />
                      <Close v-else-if="event.type === 'error'" />
                      <InfoFilled v-else />
                    </el-icon>
                  </div>
                  <div class="event-content">
                    <div class="event-message">{{ event.message }}</div>
                    <div class="event-time">{{ formatEventTime(event.timestamp) }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Control Actions -->
          <div class="monitoring-actions mt-4">
            <el-button size="small" @click="clearEvents">
              <el-icon><Delete /></el-icon>
              清除事件
            </el-button>
            <el-button size="small" @click="exportLogs">
              <el-icon><Download /></el-icon>
              导出日志
            </el-button>
            <el-button size="small" @click="viewFullLogs">
              <el-icon><View /></el-icon>
              查看完整日志
            </el-button>
          </div>
        </div>
      </el-card>
    </div>

    <!-- Alert Notifications -->
    <div class="alert-panel" v-if="alerts.length > 0">
      <el-alert
        v-for="alert in alerts"
        :key="alert.id"
        :title="alert.title"
        :type="alert.type"
        :description="alert.description"
        show-icon
        closable
        @close="dismissAlert(alert.id)"
        class="mb-2"
      >
        <template #default>
          <div class="alert-actions">
            <el-button
              v-if="alert.action"
              size="small"
              :type="alert.type"
              @click="handleAlertAction(alert)"
            >
              {{ alert.actionText }}
            </el-button>
          </div>
        </template>
      </el-alert>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch } from 'vue'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import {
  Setting,
  VideoPlay,
  VideoPause,
  Warning,
  SwitchButton,
  Close,
  Right,
  Tools,
  Monitor,
  RefreshRight,
  Camera,
  Cpu,
  Operation,
  Check,
  Histogram,
  InfoFilled,
  Delete,
  Download,
  View,
  Plus,
  Edit
} from '@element-plus/icons-vue'
import { useJobStore } from '@/stores/job'
import { useWebSocket } from '@/websocket'

interface Device {
  id: string
  name: string
  type: 'camera' | 'sorter' | 'controller' | 'sensor'
  status: 'online' | 'offline' | 'error'
  load: number
  temperature?: number
}

interface Event {
  id: string
  type: 'success' | 'warning' | 'error' | 'info'
  message: string
  timestamp: number
}

interface Alert {
  id: string
  type: 'success' | 'warning' | 'error' | 'info'
  title: string
  description: string
  action?: string
  actionText?: string
}

interface LiveStats {
  processed: number
  sorted: number
  rejected: number
  errors: number
}

interface SortingParams {
  confidenceThreshold: number
  qualityStandard: 'strict' | 'standard' | 'loose'
  sortMode: 'category' | 'quality' | 'mixed'
  enabledCategories: string[]
  defectDetection: boolean
  defectSensitivity: number
}

interface SortingRecipe {
  id: string
  name: string
  params: SortingParams
}

// Store and WebSocket
const jobStore = useJobStore()
const { connect, disconnect, send, onMessage } = useWebSocket()

// Reactive data
const systemStatus = ref<'running' | 'idle' | 'stopped' | 'error'>('stopped')
const autoMode = ref(true)
const runtime = ref(0)
const processingSpeed = ref(0)
const systemTemperature = ref(42)
const errorCount = ref(0)

const conveyorSpeed = ref(50)
const conveyor2Speed = ref(45)
const conveyorStatus = ref('running')
const conveyor2Status = ref('running')

const isStarting = ref(false)
const isPausing = ref(false)
const isStopping = ref(false)

const sortingParams = reactive<SortingParams>({
  confidenceThreshold: 85,
  qualityStandard: 'standard',
  sortMode: 'mixed',
  enabledCategories: ['apple', 'orange', 'banana', 'pear'],
  defectDetection: true,
  defectSensitivity: 5
})

const sortingRecipes = ref<SortingRecipe[]>([
  {
    id: 'default',
    name: '默认方案',
    params: { ...sortingParams }
  },
  {
    id: 'apple_standard',
    name: '苹果标准分拣',
    params: {
      confidenceThreshold: 88,
      qualityStandard: 'standard',
      sortMode: 'category',
      enabledCategories: ['apple'],
      defectDetection: true,
      defectSensitivity: 6
    }
  },
  {
    id: 'orange_strict',
    name: '橙子严格分拣',
    params: {
      confidenceThreshold: 92,
      qualityStandard: 'strict',
      sortMode: 'category',
      enabledCategories: ['orange'],
      defectDetection: true,
      defectSensitivity: 8
    }
  }
])
const newRecipeName = ref('')
const selectedRecipeId = ref('default')

const devices = ref<Device[]>([
  {
    id: 'cam1',
    name: '摄像头 #1',
    type: 'camera',
    status: 'online',
    load: 65,
    temperature: 38
  },
  {
    id: 'cam2',
    name: '摄像头 #2',
    type: 'camera',
    status: 'online',
    load: 58,
    temperature: 41
  },
  {
    id: 'sort1',
    name: '分拣器 #1',
    type: 'sorter',
    status: 'online',
    load: 72,
    temperature: 45
  },
  {
    id: 'sort2',
    name: '分拣器 #2',
    type: 'sorter',
    status: 'offline',
    load: 0
  },
  {
    id: 'ctrl1',
    name: '控制器',
    type: 'controller',
    status: 'online',
    load: 34,
    temperature: 35
  },
  {
    id: 'sensor1',
    name: '传感器阵列',
    type: 'sensor',
    status: 'online',
    load: 28
  }
])

const liveStats = reactive<LiveStats>({
  processed: 0,
  sorted: 0,
  rejected: 0,
  errors: 0
})

const recentEvents = ref<Event[]>([
  {
    id: '1',
    type: 'success',
    message: '系统启动完成',
    timestamp: Date.now() - 60000
  },
  {
    id: '2',
    type: 'info',
    message: '分拣器 #1 校准完成',
    timestamp: Date.now() - 120000
  },
  {
    id: '3',
    type: 'warning',
    message: '分拣器 #2 离线',
    timestamp: Date.now() - 180000
  }
])

const alerts = ref<Alert[]>([
  {
    id: '1',
    type: 'warning',
    title: '设备离线警告',
    description: '分拣器 #2 已离线，请检查设备连接状态',
    action: 'reconnect',
    actionText: '重新连接'
  }
])

// Methods
const startSystem = async () => {
  isStarting.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 2000))
    systemStatus.value = 'running'
    addEvent('success', '系统启动成功')
    ElMessage.success('系统已启动')

    // Start monitoring
    startMonitoring()
  } catch (error) {
    systemStatus.value = 'error'
    addEvent('error', '系统启动失败')
    ElMessage.error('系统启动失败')
  } finally {
    isStarting.value = false
  }
}

const pauseSystem = async () => {
  isPausing.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 1000))
    systemStatus.value = 'idle'
    addEvent('info', '系统已暂停')
    ElMessage.info('系统已暂停')
  } catch (error) {
    ElMessage.error('暂停系统失败')
  } finally {
    isPausing.value = false
  }
}

const stopSystem = async () => {
  isStopping.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 1500))
    systemStatus.value = 'stopped'
    runtime.value = 0
    processingSpeed.value = 0
    addEvent('info', '系统已停止')
    ElMessage.info('系统已停止')

    // Stop monitoring
    stopMonitoring()
  } catch (error) {
    ElMessage.error('停止系统失败')
  } finally {
    isStopping.value = false
  }
}

const emergencyStop = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要执行紧急停止吗？这将立即停止所有设备运行。',
      '紧急停止',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'error'
      }
    )

    systemStatus.value = 'stopped'
    runtime.value = 0
    processingSpeed.value = 0
    conveyorSpeed.value = 0
    conveyor2Speed.value = 0

    addEvent('error', '执行紧急停止')
    ElNotification({
      title: '紧急停止',
      message: '系统已紧急停止，请检查设备状态',
      type: 'error',
      duration: 0
    })
  } catch (error) {
    // User cancelled
  }
}

const toggleAutoMode = (value: boolean) => {
  addEvent('info', `切换到${value ? '自动' : '手动'}模式`)
  ElMessage.info(`已切换到${value ? '自动' : '手动'}模式`)
}

const updateConveyorSpeed = (speed: number) => {
  send('conveyor_speed', { speed })
  addEvent('info', `传送带速度调整为 ${speed}%`)
}

const setConveyorSpeed = (speed: number) => {
  conveyorSpeed.value = speed
  updateConveyorSpeed(speed)
}

const calibrateConveyor = async () => {
  try {
    ElMessage.info('开始校准传送带...')
    await new Promise(resolve => setTimeout(resolve, 3000))
    addEvent('success', '传送带校准完成')
    ElMessage.success('传送带校准完成')
  } catch (error) {
    addEvent('error', '传送带校准失败')
    ElMessage.error('传送带校准失败')
  }
}

const testConveyor = async () => {
  try {
    ElMessage.info('开始测试传送带...')
    await new Promise(resolve => setTimeout(resolve, 2000))
    addEvent('success', '传送带测试通过')
    ElMessage.success('传送带测试通过')
  } catch (error) {
    addEvent('error', '传送带测试失败')
    ElMessage.error('传送带测试失败')
  }
}

const updateSortingParams = () => {
  send('sorting_params', sortingParams)
  addEvent('info', '分拣参数已更新')
}

const resetParameters = () => {
  Object.assign(sortingParams, {
    confidenceThreshold: 85,
    qualityStandard: 'standard',
    sortMode: 'mixed',
    enabledCategories: ['apple', 'orange', 'banana', 'pear'],
    defectDetection: true,
    defectSensitivity: 5
  })
  updateSortingParams()
  ElMessage.success('参数已重置为默认值')
  selectedRecipeId.value = 'default' // Reset selected recipe to default
}

const saveRecipe = () => {
  if (!newRecipeName.value) {
    ElMessage.warning('请输入方案名称')
    return
  }

  const existingRecipe = sortingRecipes.value.find(r => r.name === newRecipeName.value)
  if (existingRecipe) {
    ElMessage.warning('方案名称已存在，请尝试更新或使用其他名称')
    return
  }

  const newRecipe: SortingRecipe = {
    id: Date.now().toString(),
    name: newRecipeName.value,
    params: { ...sortingParams }
  }
  sortingRecipes.value.push(newRecipe)
  selectedRecipeId.value = newRecipe.id
  ElMessage.success(`分拣方案 "${newRecipe.name}" 已保存`)
  newRecipeName.value = ''
}

const updateRecipe = () => {
  if (selectedRecipeId.value === 'default') {
    ElMessage.warning('默认方案不能被更新')
    return
  }
  const recipeIndex = sortingRecipes.value.findIndex(r => r.id === selectedRecipeId.value)
  if (recipeIndex > -1) {
    sortingRecipes.value[recipeIndex].params = { ...sortingParams }
    ElMessage.success(`分拣方案 "${sortingRecipes.value[recipeIndex].name}" 已更新`)
  } else {
    ElMessage.error('未找到要更新的方案')
  }
}

const loadRecipe = () => {
  const recipe = sortingRecipes.value.find(r => r.id === selectedRecipeId.value)
  if (recipe) {
    Object.assign(sortingParams, recipe.params)
    ElMessage.success(`已加载分拣方案 "${recipe.name}"`)
  } else {
    ElMessage.error('未找到选定的分拣方案')
  }
}

const deleteRecipe = async () => {
  if (selectedRecipeId.value === 'default') {
    ElMessage.warning('默认方案不能被删除')
    return
  }
  try {
    await ElMessageBox.confirm(
      `确定要删除分拣方案 "${sortingRecipes.value.find(r => r.id === selectedRecipeId.value)?.name}" 吗？`,
      '删除方案',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    const index = sortingRecipes.value.findIndex(r => r.id === selectedRecipeId.value)
    if (index > -1) {
      const deletedRecipeName = sortingRecipes.value[index].name
      sortingRecipes.value.splice(index, 1)
      selectedRecipeId.value = 'default'
      loadRecipe() // Load default after deleting
      ElMessage.success(`分拣方案 "${deletedRecipeName}" 已删除`)
    }
  } catch (error) {
    // User cancelled
  }
}

const testDevice = async (device: Device) => {
  try {
    ElMessage.info(`开始测试 ${device.name}...`)
    await new Promise(resolve => setTimeout(resolve, 2000))
    addEvent('success', `${device.name} 测试通过`)
    ElMessage.success(`${device.name} 测试通过`)
  } catch (error) {
    addEvent('error', `${device.name} 测试失败`)
    ElMessage.error(`${device.name} 测试失败`)
  }
}

const restartDevice = async (device: Device) => {
  try {
    await ElMessageBox.confirm(
      `确定要重启 ${device.name} 吗？`,
      '设备重启',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    ElMessage.info(`正在重启 ${device.name}...`)
    device.status = 'offline'

    await new Promise(resolve => setTimeout(resolve, 3000))

    device.status = 'online'
    device.load = Math.floor(Math.random() * 50) + 30
    addEvent('success', `${device.name} 重启完成`)
    ElMessage.success(`${device.name} 重启完成`)
  } catch (error) {
    // User cancelled
  }
}

const addEvent = (type: Event['type'], message: string) => {
  const event: Event = {
    id: Date.now().toString(),
    type,
    message,
    timestamp: Date.now()
  }

  recentEvents.value.unshift(event)
  if (recentEvents.value.length > 10) {
    recentEvents.value.pop()
  }
}

const clearEvents = () => {
  recentEvents.value = []
  ElMessage.success('事件列表已清空')
}

const exportLogs = () => {
  const logs = recentEvents.value.map(event => ({
    时间: new Date(event.timestamp).toLocaleString(),
    类型: event.type,
    消息: event.message
  }))

  const csvContent = [
    '时间,类型,消息',
    ...logs.map(log => `${log.时间},${log.类型},${log.消息}`)
  ].join('\n')

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = `system_logs_${new Date().toISOString().slice(0, 10)}.csv`
  link.click()
  URL.revokeObjectURL(link.href)

  ElMessage.success('日志已导出')
}

const viewFullLogs = () => {
  ElMessage.info('跳转到完整日志页面')
  // Navigate to full logs page
}

const dismissAlert = (alertId: string) => {
  const index = alerts.value.findIndex(alert => alert.id === alertId)
  if (index > -1) {
    alerts.value.splice(index, 1)
  }
}

const handleAlertAction = (alert: Alert) => {
  if (alert.action === 'reconnect') {
    // Handle reconnection logic
    ElMessage.info('正在重新连接设备...')
    setTimeout(() => {
      const device = devices.value.find(d => d.name === '分拣器 #2')
      if (device) {
        device.status = 'online'
        device.load = Math.floor(Math.random() * 50) + 30
      }
      dismissAlert(alert.id)
      ElMessage.success('设备重新连接成功')
    }, 2000)
  }
}

// Helper functions
const getStatusText = (status: string): string => {
  const texts: Record<string, string> = {
    running: '运行中',
    idle: '空闲',
    stopped: '已停止',
    error: '错误'
  }
  return texts[status] || status
}

const getDeviceTypeLabel = (type: string): string => {
  const labels: Record<string, string> = {
    camera: '摄像设备',
    sorter: '分拣设备',
    controller: '控制设备',
    sensor: '传感设备'
  }
  return labels[type] || type
}

const getTemperatureClass = (temperature: number): string => {
  if (temperature > 60) return 'text-red-500'
  if (temperature > 45) return 'text-orange-500'
  return 'text-green-500'
}

const getLoadColor = (load: number): string => {
  if (load > 80) return '#f56c6c'
  if (load > 60) return '#e6a23c'
  return '#67c23a'
}

const formatRuntime = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
}

const formatEventTime = (timestamp: number): string => {
  const now = Date.now()
  const diff = now - timestamp
  const minutes = Math.floor(diff / 60000)

  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  if (minutes < 1440) return `${Math.floor(minutes / 60)}小时前`
  return new Date(timestamp).toLocaleDateString()
}

// Monitoring
let monitoringInterval: number | null = null

const startMonitoring = () => {
  monitoringInterval = window.setInterval(() => {
    // Update runtime
    runtime.value++

    // Update processing speed (simulate fluctuation)
    processingSpeed.value = Math.floor(Math.random() * 50) + 200

    // Update live stats
    if (systemStatus.value === 'running') {
      liveStats.processed += Math.floor(Math.random() * 5) + 1
      liveStats.sorted += Math.floor(Math.random() * 4) + 1
      liveStats.rejected += Math.floor(Math.random() * 2)
      if (Math.random() < 0.1) {
        liveStats.errors++
        errorCount.value++
      }
    }

    // Update device loads
    devices.value.forEach(device => {
      if (device.status === 'online') {
        device.load = Math.max(10, Math.min(95, device.load + (Math.random() - 0.5) * 10))
        if (device.temperature) {
          device.temperature = Math.max(25, Math.min(70, device.temperature + (Math.random() - 0.5) * 2))
        }
      }
    })

    // Update system temperature
    systemTemperature.value = Math.max(30, Math.min(60, systemTemperature.value + (Math.random() - 0.5) * 2))
  }, 1000)
}

const stopMonitoring = () => {
  if (monitoringInterval) {
    clearInterval(monitoringInterval)
    monitoringInterval = null
  }
}

// Watch for changes in sortingParams to update the 'default' recipe
watch(sortingParams, (newParams) => {
  const defaultRecipe = sortingRecipes.value.find(r => r.id === 'default')
  if (defaultRecipe) {
    defaultRecipe.params = { ...newParams }
  }
}, { deep: true })

// Lifecycle
onMounted(async () => {
  // Connect to WebSocket
  await connect('ws://localhost:8000/api/v1/ws/sort-control')

  // Setup message handlers
  onMessage('device_status', (data: any) => { // Explicitly type data as any
    const device = devices.value.find(d => d.id === data.deviceId)
    if (device) {
      Object.assign(device, data)
    }
  })

  onMessage('system_status', (data: any) => { // Explicitly type data as any
    systemStatus.value = data.status
    if (data.alert) {
      alerts.value.push(data.alert)
    }
  })

  // Load initial state from store
  if (jobStore.currentJob) {
    systemStatus.value = 'running'
    startMonitoring()
  }
})

onUnmounted(() => {
  stopMonitoring()
  disconnect()
})
</script>

<style scoped>
.sort-view {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

.sort-header {
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

.header-status {
  flex-shrink: 0;
}

.control-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 20px;
}

.control-card {
  border-radius: 12px;
  border: none;
  transition: all 0.3s ease;
}

.control-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.control-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.control-row .el-button {
  flex: 1;
  min-width: 120px;
}

.emergency-control {
  text-align: center;
}

.emergency-btn {
  font-size: 16px;
  padding: 12px 24px;
  border: 2px dashed #f56c6c;
  background: rgba(245, 108, 108, 0.1);
}

.emergency-btn:hover {
  background: rgba(245, 108, 108, 0.2);
  border-color: #f78989;
}

.system-info {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-label {
  font-size: 13px;
  color: #606266;
}

.info-value {
  font-weight: 500;
  color: #303133;
}

.speed-control .control-label {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.speed-presets {
  display: flex;
  justify-content: center;
}

.conveyor-status {
  margin: 20px 0;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  background: #f8f9fa;
}

.status-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.status-icon.running {
  background: #67c23a;
}

.status-icon.stopped {
  background: #909399;
}

.status-icon.error {
  background: #f56c6c;
}

.status-info {
  flex: 1;
}

.status-label {
  font-size: 13px;
  color: #606266;
}

.status-value {
  font-weight: 500;
  color: #303133;
}

.conveyor-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.category-checkboxes {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.devices-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
}

.device-item {
  border-radius: 8px;
  padding: 16px;
  transition: all 0.3s ease;
}

.device-item.online {
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
  border: 1px solid #bae6fd;
}

.device-item.offline {
  background: linear-gradient(135deg, #fafafa 0%, #f5f5f5 100%);
  border: 1px solid #e0e0e0;
}

.device-item.error {
  background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
  border: 1px solid #fca5a5;
}

.device-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.device-icon {
  font-size: 20px;
  color: #409eff;
}

.device-info {
  flex: 1;
}

.device-name {
  font-weight: 500;
  color: #303133;
}

.device-type {
  font-size: 12px;
  color: #909399;
}

.device-status-indicator {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
}

.device-status-indicator.online {
  background: #67c23a;
}

.device-status-indicator.offline {
  background: #909399;
}

.device-status-indicator.error {
  background: #f56c6c;
}

.device-metrics {
  margin-bottom: 12px;
}

.metric {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.metric-label {
  width: 40px;
  font-size: 12px;
  color: #606266;
}

.metric-value {
  font-size: 12px;
  font-weight: 500;
}

.device-actions {
  display: flex;
  gap: 6px;
}

.monitoring-panel {
  width: 100%;
}

.monitoring-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.stats-section h4,
.events-section h4 {
  font-size: 16px;
  font-weight: 500;
  color: #303133;
  margin: 0 0 12px 0;
}

.live-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  background: #fafafa;
}

.stat-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.success {
  background: #67c23a;
}

.stat-icon.info {
  background: #409eff;
}

.stat-icon.warning {
  background: #e6a23c;
}

.stat-icon.danger {
  background: #f56c6c;
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 12px;
  color: #606266;
}

.events-list {
  max-height: 300px;
  overflow-y: auto;
}

.event-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 6px;
  margin-bottom: 8px;
  transition: background-color 0.2s;
}

.event-item:hover {
  background: rgba(0, 0, 0, 0.02);
}

.event-item.success .event-icon {
  color: #67c23a;
}

.event-item.warning .event-icon {
  color: #e6a23c;
}

.event-item.error .event-icon {
  color: #f56c6c;
}

.event-item.info .event-icon {
  color: #409eff;
}

.event-content {
  flex: 1;
}

.event-message {
  font-size: 13px;
  color: #303133;
}

.event-time {
  font-size: 11px;
  color: #909399;
}

.monitoring-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.alert-panel {
  position: fixed;
  top: 80px;
  right: 20px;
  max-width: 400px;
  z-index: 1000;
}

.alert-actions {
  margin-top: 8px;
}

/* Responsive design */
@media (max-width: 768px) {
  .sort-view {
    padding: 12px;
  }

  .sort-header {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }

  .control-grid {
    grid-template-columns: 1fr;
  }

  .control-row {
    flex-direction: column;
  }

  .monitoring-grid {
    grid-template-columns: 1fr;
  }

  .live-stats {
    grid-template-columns: 1fr;
  }

  .info-grid {
    grid-template-columns: 1fr;
  }

  .devices-grid {
    grid-template-columns: 1fr;
  }

  .alert-panel {
    position: relative;
    top: auto;
    right: auto;
    max-width: none;
    margin-top: 20px;
  }
}

/* Animation classes */
.text-red-500 {
  color: #f56c6c;
}

.text-orange-500 {
  color: #e6a23c;
}

.text-green-500 {
  color: #67c23a;
}
</style>
