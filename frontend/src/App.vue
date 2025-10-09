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
          <!-- Live Status Bar (Redesigned) -->
          <div v-if="showLiveView" class="live-status-bar mb-4 p-3 rounded-lg flex items-center justify-between">
            <div class="flex items-center">
              <el-icon class="live-status-icon mr-2 animate-pulse"><VideoCamera /></el-icon>
              <span class="live-status-text font-medium">实时监控已启动</span>
            </div>
            <div class="text-sm text-gray-600">
              连接状态: <span :class="wsConnected ? 'text-green-600' : 'text-red-600'">
                {{ wsConnected ? '已连接' : '已断开' }}
              </span>
              <el-spinner v-if="showLiveView && !wsConnected" size="small" class="ml-2" />
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
          <el-input v-model="settings.apiEndpoint" placeholder="http://localhost:8000/api/v1" />
        </el-form-item>
        <el-form-item label="WebSocket端点">
          <el-input v-model="settings.wsEndpoint" placeholder="ws://localhost:8000/api/v1/ws/live" />
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
  apiEndpoint: 'http://localhost:8000/api/v1',
  wsEndpoint: 'ws://localhost:8000/api/v1/ws/live',
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
      }
    })
    wsConnected.value = true // Assume connected immediately after calling connectWebSocket
  } else {
    disconnectWebSocket()
    wsConnected.value = false // Assume disconnected immediately after calling disconnectWebSocket
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
  background-color: var(--color-background-light); /* Use new light background color */
  position: relative;
  overflow: hidden;
}

/* Removed .app-background::before for a cleaner look */

.sidebar-background {
  background-color: var(--color-white); /* Solid white for sidebar */
  border-right: 1px solid var(--color-border); /* Use new border color */
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.05); /* Subtle shadow */
}

.el-menu-item {
  color: var(--color-text-secondary); /* Use secondary text color */
  font-weight: 500; /* Slightly less bold */
  font-size: 15px; /* Slightly smaller font */
  border-radius: 8px; /* Add border-radius for menu items */
  margin: 4px 8px; /* Add some margin */
  padding-left: 16px !important; /* Adjust padding */
}

.el-menu-item.is-active {
  background-color: var(--color-primary-light) !important; /* Use primary light for active background */
  color: var(--color-white) !important; /* White text for active item */
  border-left: none; /* Remove left border */
  box-shadow: 0 2px 8px rgba(46, 204, 113, 0.2); /* Subtle shadow for active item */
}

.el-menu-item:hover {
  background-color: var(--color-primary-light) !important; /* Use primary light for hover */
  color: var(--color-white) !important; /* White text on hover */
}

.el-menu-item .el-icon {
  color: var(--color-text-secondary); /* Secondary text color for icons */
  font-size: 18px; /* Adjust icon size */
}

.el-menu-item.is-active .el-icon {
  color: var(--color-white) !important; /* White icon for active item */
}

.main-content-background {
  background-color: var(--color-white); /* Solid white for main content */
  border-radius: 12px; /* Slightly larger border-radius */
  margin: 16px; /* Adjust margin */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08); /* More subtle shadow */
}

.live-status-bar {
  background: linear-gradient(90deg, var(--color-primary-light) 0%, var(--color-primary) 100%);
  color: var(--color-white);
  box-shadow: 0 4px 12px rgba(46, 204, 113, 0.3);
}

.live-status-icon {
  color: var(--color-white);
  font-size: 20px;
}

.live-status-text {
  color: var(--color-white);
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
