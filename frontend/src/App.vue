<template>
  <div id="app">
    <el-container class="min-h-screen">
      <!-- Header -->
      <el-header class="bg-blue-600 text-white flex items-center justify-between px-6">
        <div class="flex items-center">
          <el-icon class="text-2xl mr-3"><Apple /></el-icon>
          <h1 class="text-xl font-bold">果蔬智能分拣系统</h1>
        </div>
        <div class="flex items-center space-x-4">
          <el-badge :value="liveJobsCount" :hidden="liveJobsCount === 0" class="mr-2">
            <el-button type="primary" :icon="Monitor" @click="toggleLiveView">
              {{ showLiveView ? '关闭实时' : '实时监控' }}
            </el-button>
          </el-badge>
          <el-button type="success" :icon="Setting" @click="showSettings = true">
            设置
          </el-button>
        </div>
      </el-header>

      <el-container>
        <!-- Sidebar -->
        <el-aside width="250px" class="bg-gray-100 border-r">
          <el-menu
            :default-active="$route.path"
            router
            class="h-full"
          >
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
        <el-main class="p-6">
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
              </div>
            </div>
          </div>

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
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useJobStore } from './stores/job'
import { connectWebSocket, disconnectWebSocket, isConnected } from './websocket'
import {
  Apple, Monitor, Setting, Upload, Camera, DataAnalysis, List,
  Operation, VideoCamera
} from '@element-plus/icons-vue'

const router = useRouter()
const jobStore = useJobStore()

const showLiveView = ref(false)
const showSettings = ref(false)
const wsConnected = ref(false)
const liveJobsCount = ref(0)

const settings = ref({
  apiEndpoint: 'http://localhost:8000',
  wsEndpoint: 'ws://localhost:8000/ws/live',
  refreshInterval: 10000
})

const toggleLiveView = () => {
  showLiveView.value = !showLiveView.value

  if (showLiveView.value) {
    connectWebSocket(settings.value.wsEndpoint, (message) => {
      // Handle WebSocket messages
      if (message.type === 'detection_result') {
        jobStore.updateJob(message.data)
        liveJobsCount.value = jobStore.activeJobs.length
      }
    })
  } else {
    disconnectWebSocket()
  }

  wsConnected.value = isConnected()
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
  router.push('/upload')

  // Check WebSocket connection status periodically
  setInterval(() => {
    wsConnected.value = isConnected()
  }, 1000)
})

onUnmounted(() => {
  disconnectWebSocket()
})
</script>

<style>
#app {
  font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
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