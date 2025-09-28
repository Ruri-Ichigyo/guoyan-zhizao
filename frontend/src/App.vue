<template>
  <div id="app">
    <el-container class="min-h-screen">
      <el-container class="min-h-screen app-background">
        <!-- Sidebar -->
        <el-aside width="250px" class="sidebar-background border-r border-transparent">
          <el-menu
            :default-active="$route.path"
            router
            class="h-full bg-transparent border-r-0"
          >
            <el-menu-item index="/">
              <el-icon><HomeFilled /></el-icon>
              <span>首页</span>
            </el-menu-item>
            <el-menu-item index="/upload">
              <el-icon><Upload /></el-icon>
              <span>图像上传</span>
            </el-menu-item>
            <el-menu-item index="/camera">
              <el-icon><Camera /></el-icon>
              <span>实时摄像</span>
            </el-menu-item>
            <el-menu-item index="/dashboard">
              <el-icon><DataAnalysis /></el-icon>
              <span>数据大屏</span>
            </el-menu-item>
            <el-menu-item index="/results">
              <el-icon><List /></el-icon>
              <span>结果查看</span>
            </el-menu-item>
            <el-menu-item index="/sort">
              <el-icon><Operation /></el-icon>
              <span>分拣控制</span>
            </el-menu-item>
          </el-menu>
        </el-aside>

        <!-- Main Content -->
        <el-main class="p-6 main-content-background">
          <!-- Live Status Bar -->
          <div v-if="showLiveView" class="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg">
            <div class="flex items-center justify-between">
              <div class="flex items-center">
                <el-icon class="text-green-500 mr-2 animate-pulse"><VideoCamera /></el-icon>
                <span class="text-green-700 font-medium">实时监控已启动</span>
              </div>
              <div class="text-sm text-gray-600">
                连接状态: <span :class="wsConnected ? 'text-green-600' : 'text-red-600'">
                  {{ wsConnected ? '已连接' : '已断开' }}
                </span>
                <el-spinner v-if="showLiveView && !wsConnected" size="small" class="ml-2" />
              </div>
            </div>
          </div>

          <!-- WebSocket Error Alert -->
          <el-alert
            v-if="wsError"
            :title="wsError"
            type="error"
            show-icon
            closable
            class="mb-4"
            @close="wsError = null"
          />

          <!-- Router View -->
          <router-view />
        </el-main>
      </el-container>
    </el-container>

    <!-- Settings Dialog -->
    <el-dialog v-model="showSettings" title="系统设置" width="600px">
      <el-form label-width="120px">
        <el-form-item label="API端点">
          <el-input v-model="settings.apiEndpoint" placeholder="http://localhost:8000" />
        </el-form-item>
        <el-form-item label="WebSocket端点">
          <el-input v-model="settings.wsEndpoint" placeholder="ws://localhost:8000/ws/live" />
        </el-form-item>
        <el-form-item label="自动刷新间隔">
          <el-select v-model="settings.refreshInterval">
            <el-option label="5秒" :value="5000" />
            <el-option label="10秒" :value="10000" />
            <el-option label="30秒" :value="30000" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showSettings = false">取消</el-button>
        <el-button type="primary" @click="saveSettings">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useJobStore } from './stores/job'
// If './websocket' exports a default object with these methods:
// Import the required named exports from './websocket'
import { useWebSocket, disconnectAllWebSockets } from './websocket'
// If you need connectWebSocket, ensure it is exported from './websocket.ts'
import {
  Apple, Monitor, Setting, Upload, Camera, DataAnalysis, List,
  Operation, VideoCamera, HomeFilled
} from '@element-plus/icons-vue'

const router = useRouter()
const jobStore = useJobStore()
const { connect, disconnect, onMessage, state: wsState } = useWebSocket('live')

const showLiveView = ref(false)
const showSettings = ref<boolean>(false)
const wsConnected = ref(false) // This will be derived from wsState.connected
const wsError = ref<string | null>(null) // New: To store WebSocket connection errors
const liveJobsCount = ref(0)

const settings = ref({
  apiEndpoint: 'http://localhost:8000',
  wsEndpoint: 'ws://localhost:8000/ws/live',
  refreshInterval: 10000
})

const toggleLiveView = async () => {
  showLiveView.value = !showLiveView.value
  wsError.value = null // Clear previous errors

  if (showLiveView.value) {
    try {
      await connect(settings.value.wsEndpoint)
      onMessage('detection_result', (data: any) => {
        jobStore.updateJob(data)
        liveJobsCount.value = jobStore.activeJobs.length
      })
    } catch (error: any) {
      console.error('WebSocket connection error:', error)
      wsError.value = `WebSocket连接失败: ${error.message || '未知错误'}`
      showLiveView.value = false // Turn off live view if connection fails
    }
  } else {
    disconnect()
  }
}

const saveSettings = () => {
  localStorage.setItem('fruit-sort-settings', JSON.stringify(settings.value))
  showSettings.value = false
}

const loadSettings = () => {
  const saved = localStorage.getItem('fruit-sort-settings')
  if (saved) {
    settings.value = { ...settings.value, ...JSON.parse(saved) }
  }
}

onMounted(() => {
  loadSettings()
  // No longer push to /upload by default, as we now have a home page
})

onUnmounted(() => {
  disconnect()
})

// Watch wsState to update wsConnected
watch(() => wsState.value.connected, (newVal: boolean) => {
  wsConnected.value = newVal
})

</script>

<style>
#app {
  font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

.app-background {
  background: linear-gradient(135deg, #e6ffe6 0%, #c0ffc0 100%); /* Light green gradient */
  position: relative;
  overflow: hidden;
}

.app-background::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: url('/images/vine_top.png'), url('/images/blueberry_left.png'), url('/images/grape_right.png'), url('/images/kiwi_bottom.png'); /* Placeholder images */
  background-repeat: no-repeat;
  background-position: top center, 5% 80%, 95% 20%, 80% 90%;
  background-size: 100% auto, 150px, 200px, 180px;
  opacity: 0.8; /* Adjust as needed */
  z-index: -1;
}

.sidebar-background {
  background-color: rgba(255, 255, 255, 0.3); /* Semi-transparent white */
  backdrop-filter: blur(5px); /* Frosted glass effect */
  border-right: 1px solid rgba(255, 255, 255, 0.2);
}

.el-menu-item {
  color: #66bb6a; /* Light green text */
  font-weight: bold;
  font-size: 16px; /* Slightly larger font */
}

.el-menu-item.is-active {
  background-color: rgba(76, 175, 80, 0.3) !important; /* More vibrant green active background */
  color: #388e3c !important; /* Darker green text for active item */
  border-left: 4px solid #4CAF50; /* Green border on active item */
}

.el-menu-item:hover {
  background-color: rgba(76, 175, 80, 0.15) !important; /* Lighter vibrant green hover */
}

.el-menu-item .el-icon {
  color: #66bb6a; /* Light green icon */
}

.el-menu-item.is-active .el-icon {
  color: #388e3c !important; /* Darker green icon for active item */
}

.main-content-background {
  background-color: rgba(255, 255, 255, 0.4); /* Semi-transparent white */
  backdrop-filter: blur(5px);
  border-radius: 10px;
  margin: 20px; /* Add some margin around the main content */
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
