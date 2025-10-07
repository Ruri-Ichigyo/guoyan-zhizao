<template>
  <div class="live-camera">
    <el-card class="camera-card" shadow="hover">
      <template #header>
        <div class="flex justify-between items-center">
          <h3 class="text-lg font-semibold">实时摄像头监控</h3>
          <div class="flex gap-2">
            <el-button
              type="primary"
              size="small"
              :icon="isStreaming ? VideoPause : VideoPlay"
              @click="toggleStream"
              :loading="loading"
            >
              {{ isStreaming ? '停止' : '开始' }}
            </el-button>
            <el-dropdown @command="switchStream">
              <el-button size="small" type="info" style=" margin-left: 16px;">
                切换摄像头
                <el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item
                    v-for="camera in availableCameras"
                    :key="camera.id"
                    :command="camera.id"
                    :disabled="camera.id === currentCameraId"
                  >
                    {{ camera.name }}
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <div class="camera-container">
        <!-- WebRTC/RTSP Video Stream -->
        <div class="video-wrapper" :class="{ 'streaming': isStreaming }">
          <video
            ref="videoRef"
            autoplay
            muted
            playsinline
            class="video-stream"
            :class="{ 'hidden': !isStreaming || hasError }"
            @loadstart="onVideoLoadStart"
            @loadeddata="onVideoLoaded"
            @error="onVideoError"
            @canplay="onVideoCanPlay"
          ></video>

          <!-- Loading State -->
          <div v-if="loading" class="video-placeholder">
            <el-icon class="loading-icon" size="48"><Loading /></el-icon>
            <p class="mt-2">正在连接摄像头...</p>
          </div>

          <!-- Error State -->
          <div v-if="hasError && !loading" class="video-placeholder error">
            <el-icon class="error-icon" size="48"><VideoCameraFilled /></el-icon>
            <p class="mt-2 text-red-500">{{ errorMessage }}</p>
            <el-button type="primary" size="small" @click="retryConnection" class="mt-2">
              重新连接
            </el-button>
          </div>

          <!-- No Stream State -->
          <div v-if="!isStreaming && !loading && !hasError" class="video-placeholder">
            <el-icon class="placeholder-icon" size="48"><VideoCameraFilled /></el-icon>
            <p class="mt-2 text-gray-500">点击开始按钮启动摄像头</p>
          </div>

          <!-- Stream Info Overlay -->
          <div v-if="isStreaming && !hasError" class="stream-info">
            <div class="info-item">
              <el-tag type="success" size="small">
                <el-icon><VideoPlay /></el-icon>
                实时流
              </el-tag>
            </div>
            <div class="info-item">
              <el-tag type="info" size="small">
                {{ streamResolution }}
              </el-tag>
            </div>
            <div class="info-item">
              <el-tag type="warning" size="small">
                FPS: {{ currentFps }}
            </el-tag>
            </div>
          </div>

          <!-- Recording Indicator -->
          <div v-if="isRecording" class="recording-indicator">
            <el-icon class="recording-dot"><CircleFilled /></el-icon>
            <span>录制中</span>
          </div>
        </div>

        <!-- Control Panel -->
        <div class="control-panel mt-4">
          <div class="flex justify-between items-center">
            <div class="flex gap-2">
              <el-button
                size="small"
                @click="captureSnapshot"
                :disabled="!isStreaming || hasError"
              >
                <el-icon><Camera /></el-icon>
                截图
              </el-button>
              <el-button
                size="small"
                :type="isRecording ? 'danger' : 'warning'"
                @click="toggleRecording"
                :disabled="!isStreaming || hasError"
              >
                <el-icon><VideoCamera /></el-icon>
                {{ isRecording ? '停止录制' : '开始录制' }}
              </el-button>
              <el-button
                size="small"
                @click="toggleFullscreen"
                :disabled="!isStreaming || hasError"
              >
                <el-icon><FullScreen /></el-icon>
                全屏
              </el-button>
            </div>

            <div class="stream-stats text-sm text-gray-500">
              <span>延迟: {{ latency }}ms</span>
              <span class="ml-2">质量: {{ streamQuality }}</span>
            </div>
          </div>

          <!-- Stream Settings -->
          <el-collapse v-model="activeSettings" class="mt-3">
            <el-collapse-item name="settings" title="流媒体设置">
              <div class="settings-grid grid grid-cols-2 gap-4">
                <div>
                  <el-form-item label="分辨率">
                    <el-select v-model="selectedResolution" size="small" @change="changeResolution">
                      <el-option
                        v-for="res in resolutionOptions"
                        :key="res.value"
                        :label="res.label"
                        :value="res.value"
                      />
                    </el-select>
                  </el-form-item>
                </div>
                <div>
                  <el-form-item label="帧率">
                    <el-select v-model="selectedFps" size="small" @change="changeFps">
                      <el-option
                        v-for="fps in fpsOptions"
                        :key="fps"
                        :label="`${fps} FPS`"
                        :value="fps"
                      />
                    </el-select>
                  </el-form-item>
                </div>
                <div>
                  <el-form-item label="编码格式">
                    <el-select v-model="selectedCodec" size="small" @change="changeCodec">
                      <el-option label="H.264" value="h264" />
                      <el-option label="H.265" value="h265" />
                      <el-option label="VP8" value="vp8" />
                      <el-option label="VP9" value="vp9" />
                    </el-select>
                  </el-form-item>
                </div>
                <div>
                  <el-form-item label="比特率">
                    <el-slider
                      v-model="bitrate"
                      :min="500"
                      :max="5000"
                      :step="100"
                      @change="changeBitrate"
                    />
                    <span class="text-xs text-gray-500">{{ bitrate }}kbps</span>
                  </el-form-item>
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  VideoPlay,
  VideoPause,
  VideoCameraFilled,
  Camera,
  VideoCamera,
  FullScreen,
  Loading,
  ArrowDown,
  CirclePlusFilled /* Corrected icon name */
} from '@element-plus/icons-vue'

interface CameraDevice {
  id: string
  name: string
  type: 'webcam' | 'rtsp' | 'webrtc'
}

interface StreamStats {
  fps: number
  bitrate: number
  resolution: string
  latency: number
  quality: 'excellent' | 'good' | 'fair' | 'poor'
}

// Refs
const videoRef = ref<HTMLVideoElement>()
const isStreaming = ref(false)
const loading = ref(false)
const hasError = ref(false)
const errorMessage = ref('')
const isRecording = ref(false)
const currentCameraId = ref('default')
const activeSettings = ref<string[]>([])

// Stream info
const streamResolution = ref('1920x1080')
const currentFps = ref(30)
const latency = ref(0)
const streamQuality = ref<StreamStats['quality']>('excellent')

// Settings
const selectedResolution = ref('1920x1080')
const selectedFps = ref(30)
const selectedCodec = ref('h264')
const bitrate = ref(2000)

// Options
const resolutionOptions = [
  { label: '4K (3840x2160)', value: '3840x2160' },
  { label: '1080p (1920x1080)', value: '1920x1080' },
  { label: '720p (1280x720)', value: '1280x720' },
  { label: '480p (640x480)', value: '640x480' }
]

const fpsOptions = [60, 30, 25, 15]

const availableCameras = ref<CameraDevice[]>([
  { id: 'default', name: '默认摄像头', type: 'webcam' },
  { id: 'rtsp_1', name: 'RTSP摄像头 1', type: 'rtsp' },
  { id: 'rtsp_2', name: 'RTSP摄像头 2', type: 'rtsp' },
  { id: 'webrtc_1', name: 'WebRTC摄像头 1', type: 'webrtc' }
])

// Stream references
let mediaStream: MediaStream | null = null
let mediaRecorder: MediaRecorder | null = null
let recordedChunks: Blob[] = []
let statsInterval: number | null = null

// Methods
const toggleStream = async () => {
  if (isStreaming.value) {
    await stopStream()
  } else {
    await startStream()
  }
}

const startStream = async () => {
  loading.value = true
  hasError.value = false
  errorMessage.value = ''

  try {
    const currentCamera = availableCameras.value.find(cam => cam.id === currentCameraId.value)

    if (!currentCamera) {
      throw new Error('未找到指定的摄像头')
    }

    switch (currentCamera.type) {
      case 'webcam':
        await startWebcamStream()
        break
      case 'rtsp':
        await startRtspStream(currentCamera.id)
        break
      case 'webrtc':
        await startWebRtcStream(currentCamera.id)
        break
    }

    isStreaming.value = true
    startStatsMonitoring()
    ElMessage.success('摄像头已连接')
  } catch (error) {
    console.error('Failed to start stream:', error)
    hasError.value = true
    errorMessage.value = error instanceof Error ? error.message : '启动摄像头失败'
    ElMessage.error('启动摄像头失败')
  } finally {
    loading.value = false
  }
}

const startWebcamStream = async () => {
  const constraints = {
    video: {
      width: { ideal: parseInt(selectedResolution.value.split('x')[0]) },
      height: { ideal: parseInt(selectedResolution.value.split('x')[1]) },
      frameRate: { ideal: selectedFps.value }
    },
    audio: false
  }

  mediaStream = await navigator.mediaDevices.getUserMedia(constraints)

  if (videoRef.value) {
    videoRef.value.srcObject = mediaStream
  }
}

const startRtspStream = async (cameraId: string) => {
  // For RTSP streams, we would typically use a WebSocket connection
  // to receive the stream from a backend service that converts RTSP to WebRTC
  const rtspUrl = `rtsp://example.com/stream/${cameraId}`

  // Simulate WebRTC connection for RTSP stream
  // In a real implementation, this would connect to a WebRTC signaling server
  const pc = new RTCPeerConnection({
    iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
  })

  // This is a placeholder - actual RTSP to WebRTC conversion would happen server-side
  pc.ontrack = (event) => {
    if (videoRef.value && event.streams[0]) {
      videoRef.value.srcObject = event.streams[0]
      mediaStream = event.streams[0]
    }
  }

  // For demo purposes, fall back to webcam if RTSP not available
  await startWebcamStream()
}

const startWebRtcStream = async (cameraId: string) => {
  // WebRTC peer connection setup
  const pc = new RTCPeerConnection({
    iceServers: [
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' }
    ]
  })

  pc.ontrack = (event) => {
    if (videoRef.value && event.streams[0]) {
      videoRef.value.srcObject = event.streams[0]
      mediaStream = event.streams[0]
    }
  }

  // For demo purposes, fall back to webcam
  await startWebcamStream()
}

const stopStream = async () => {
  if (mediaStream) {
    mediaStream.getTracks().forEach(track => track.stop())
    mediaStream = null
  }

  if (videoRef.value) {
    videoRef.value.srcObject = null
  }

  if (isRecording.value) {
    stopRecording()
  }

  stopStatsMonitoring()
  isStreaming.value = false
  ElMessage.info('摄像头已断开')
}

const switchStream = async (cameraId: string) => {
  if (isStreaming.value) {
    await stopStream()
  }
  currentCameraId.value = cameraId
  await nextTick()
  await startStream()
}

const retryConnection = async () => {
  hasError.value = false
  await startStream()
}

const captureSnapshot = () => {
  if (!videoRef.value || !isStreaming.value) return

  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')

  if (!ctx) return

  canvas.width = videoRef.value.videoWidth
  canvas.height = videoRef.value.videoHeight
  ctx.drawImage(videoRef.value, 0, 0)

  canvas.toBlob((blob) => {
    if (blob) {
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `snapshot-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.png`
      a.click()
      URL.revokeObjectURL(url)
      ElMessage.success('截图已保存')
    }
  }, 'image/png')
}

const toggleRecording = () => {
  if (isRecording.value) {
    stopRecording()
  } else {
    startRecording()
  }
}

const startRecording = () => {
  if (!mediaStream) return

  recordedChunks = []
  mediaRecorder = new MediaRecorder(mediaStream, {
    mimeType: 'video/webm'
  })

  mediaRecorder.ondataavailable = (event) => {
    if (event.data.size > 0) {
      recordedChunks.push(event.data)
    }
  }

  mediaRecorder.onstop = () => {
    const blob = new Blob(recordedChunks, { type: 'video/webm' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `recording-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.webm`
    a.click()
    URL.revokeObjectURL(url)
    ElMessage.success('录制文件已保存')
  }

  mediaRecorder.start()
  isRecording.value = true
  ElMessage.success('开始录制')
}

const stopRecording = () => {
  if (mediaRecorder && mediaRecorder.state === 'recording') {
    mediaRecorder.stop()
  }
  isRecording.value = false
  ElMessage.info('录制已停止')
}

const toggleFullscreen = () => {
  if (!videoRef.value) return

  if (document.fullscreenElement) {
    document.exitFullscreen()
  } else {
    videoRef.value.requestFullscreen()
  }
}

const changeResolution = async (resolution: string) => {
  if (isStreaming.value) {
    await stopStream()
    await nextTick()
    await startStream()
  }
}

const changeFps = async (fps: number) => {
  if (isStreaming.value) {
    await stopStream()
    await nextTick()
    await startStream()
  }
}

const changeCodec = (codec: string) => {
  // Codec change would require stream restart in real implementation
  ElMessage.info(`编码格式已切换到 ${codec.toUpperCase()}`)
}

const changeBitrate = (rate: number) => {
  // Bitrate adjustment would be applied to the stream
  ElMessage.info(`比特率已调整到 ${rate}kbps`)
}

const startStatsMonitoring = () => {
  statsInterval = window.setInterval(() => {
    if (videoRef.value && mediaStream) {
      // Update stream statistics
      currentFps.value = selectedFps.value
      streamResolution.value = selectedResolution.value
      latency.value = Math.floor(Math.random() * 50) + 10 // Mock latency

      // Mock quality assessment
      const qualities: StreamStats['quality'][] = ['excellent', 'good', 'fair', 'poor']
      streamQuality.value = qualities[Math.floor(latency.value / 15)] || 'excellent'
    }
  }, 1000)
}

const stopStatsMonitoring = () => {
  if (statsInterval) {
    clearInterval(statsInterval)
    statsInterval = null
  }
}

// Video event handlers
const onVideoLoadStart = () => {
  console.log('Video load started')
}

const onVideoLoaded = () => {
  console.log('Video loaded')
  if (videoRef.value) {
    streamResolution.value = `${videoRef.value.videoWidth}x${videoRef.value.videoHeight}`
  }
}

const onVideoError = (error: Event) => {
  console.error('Video error:', error)
  hasError.value = true
  errorMessage.value = '视频流加载失败'
  isStreaming.value = false
  loading.value = false
}

const onVideoCanPlay = () => {
  console.log('Video can play')
  loading.value = false
}

// Lifecycle
onMounted(async () => {
  // Check for camera permissions
  try {
    const devices = await navigator.mediaDevices.enumerateDevices()
    const videoDevices = devices.filter(device => device.kind === 'videoinput')

    // Update available cameras with actual devices
    const webcams = videoDevices.map((device, index) => ({
      id: device.deviceId || `webcam_${index}`,
      name: device.label || `摄像头 ${index + 1}`,
      type: 'webcam' as const
    }))

    availableCameras.value = [
      ...webcams,
      ...availableCameras.value.filter(cam => cam.type !== 'webcam')
    ]
  } catch (error) {
    console.error('Failed to enumerate devices:', error)
  }
})

onUnmounted(() => {
  stopStream()
})
</script>

<style scoped>
.live-camera {
  width: 100%;
  height: 100%;
}

.camera-card {
  width: 100%;
  /* 移除固定高度，让内容自适应 */
  border-radius: 12px; /* Consistent border-radius */
  border: 1px solid var(--color-border); /* Consistent border */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08); /* Consistent shadow */
  background-color: var(--color-white); /* Consistent background */
  padding: 20px; /* 增加卡片内边距 */
}

.camera-container {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px; /* 增加视频区域和控制面板之间的间距 */
}

.video-wrapper {
  position: relative;
  width: 100%;
  /* 调整视频高度，使其更具响应性 */
  padding-bottom: 56.25%; /* 16:9 比例 */
  height: 0;
  background: var(--color-black); /* Use black for video background */
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-stream {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.video-stream {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.video-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #666;
  text-align: center;
}

.video-placeholder.error {
  color: #f56c6c;
}

.loading-icon {
  animation: spin 1s linear infinite;
  color: var(--color-accent); /* Use accent color for loading */
}

.error-icon {
  color: var(--color-error); /* Use error color */
}

.placeholder-icon {
  color: var(--color-text-placeholder); /* Use placeholder text color */
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.stream-info {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.stream-info .el-tag {
  background-color: rgba(0, 0, 0, 0.6); /* Darker background for tags */
  color: var(--color-white);
  border-color: transparent;
}

.recording-indicator {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  align-items: center;
  gap: 5px;
  background: rgba(0, 0, 0, 0.7);
  color: var(--color-white);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.recording-dot {
  color: var(--color-error); /* Use error color for recording dot */
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.control-panel {
  background: var(--color-background-medium); /* Use medium background color */
  padding: 16px; /* 增加控制面板内边距 */
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 15px; /* 增加控制面板内部组件的间距 */
}

.stream-stats span {
  display: inline-block;
  margin-left: 10px; /* 增加统计数据项之间的间距 */
}

.settings-grid {
  padding: 10px 0;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); /* 调整设置网格布局，使其更灵活 */
}

:deep(.el-collapse-item__header) {
  font-size: 14px;
}

:deep(.el-form-item) {
  margin-bottom: 12px;
}

:deep(.el-form-item__label) {
  font-size: 12px;
  line-height: 20px;
}
</style>
