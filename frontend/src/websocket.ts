import { ref, reactive, onUnmounted, readonly } from 'vue'
import type { Ref } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'

// Types
export interface WebSocketMessage {
  type: string
  data: any
  timestamp?: number
  id?: string
}

export interface WebSocketConfig {
  url: string
  protocols?: string[]
  reconnect?: boolean
  reconnectInterval?: number
  maxReconnectAttempts?: number
  heartbeat?: boolean
  heartbeatInterval?: number
  debug?: boolean
}

export interface WebSocketState {
  connected: boolean
  connecting: boolean
  reconnectCount: number
  lastConnectedAt: string | null
  lastDisconnectedAt: string | null
  error: string | null
}

// Default configuration
const DEFAULT_CONFIG: Required<Omit<WebSocketConfig, 'url'>> = {
  protocols: [], // Add protocols with an empty array as default
  reconnect: true,
  reconnectInterval: 5000,
  maxReconnectAttempts: 10,
  heartbeat: true,
  heartbeatInterval: 30000,
  debug: false
}

// Global WebSocket management
class WebSocketManager {
  private connections: Map<string, WebSocketConnection> = new Map()
  private static instance: WebSocketManager

  static getInstance(): WebSocketManager {
    if (!WebSocketManager.instance) {
      WebSocketManager.instance = new WebSocketManager()
    }
    return WebSocketManager.instance
  }

  getConnection(key: string): WebSocketConnection | undefined {
    return this.connections.get(key)
  }

  setConnection(key: string, connection: WebSocketConnection): void {
    this.connections.set(key, connection)
  }

  removeConnection(key: string): void {
    const connection = this.connections.get(key)
    if (connection) {
      connection.disconnect()
      this.connections.delete(key)
    }
  }

  disconnectAll(): void {
    this.connections.forEach((connection) => {
      connection.disconnect()
    })
    this.connections.clear()
  }
}

// WebSocket connection class
class WebSocketConnection {
  private ws: WebSocket | null = null
  private config: Required<WebSocketConfig>
  private state: WebSocketState
  private messageHandlers: Map<string, Function[]> = new Map()
  private reconnectTimer: number | null = null
  private heartbeatTimer: number | null = null
  private messageQueue: WebSocketMessage[] = []

  constructor(config: WebSocketConfig) {
    this.config = { ...DEFAULT_CONFIG, ...config }
    this.state = reactive({
      connected: false,
      connecting: false,
      reconnectCount: 0,
      lastConnectedAt: null,
      lastDisconnectedAt: null,
      error: null
    })
  }

  getState(): WebSocketState {
    return this.state
  }

  async connect(): Promise<void> {
    if (this.state.connected || this.state.connecting) {
      return
    }

    this.state.connecting = true
    this.state.error = null

    try {
      this.ws = new WebSocket(this.config.url, this.config.protocols)

      this.ws.onopen = this.handleOpen.bind(this)
      this.ws.onmessage = this.handleMessage.bind(this)
      this.ws.onclose = this.handleClose.bind(this)
      this.ws.onerror = this.handleError.bind(this)

      if (this.config.debug) {
        console.log(`[WebSocket] Connecting to ${this.config.url}`)
      }
    } catch (error) {
      this.state.connecting = false
      this.state.error = error instanceof Error ? error.message : '连接失败'
      throw error
    }
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.close(1000, 'Client disconnect')
    }

    this.state.connected = false
    this.state.connecting = false
    this.messageQueue = []
  }

  send(type: string, data: any): boolean {
    const message: WebSocketMessage = {
      type,
      data,
      timestamp: Date.now(),
      id: this.generateMessageId()
    }

    if (this.state.connected && this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message))
        if (this.config.debug) {
          console.log('[WebSocket] Sent message:', message)
        }
        return true
      } catch (error) {
        if (this.config.debug) {
          console.error('[WebSocket] Send error:', error)
        }
        return false
      }
    } else {
      // Queue message if not connected
      this.messageQueue.push(message)
      if (this.config.debug) {
        console.log('[WebSocket] Message queued:', message)
      }
      return false
    }
  }

  on(type: string, handler: Function): void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, [])
    }
    this.messageHandlers.get(type)!.push(handler)
  }

  off(type: string, handler?: Function): void {
    if (!handler) {
      this.messageHandlers.delete(type)
      return
    }

    const handlers = this.messageHandlers.get(type)
    if (handlers) {
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
      if (handlers.length === 0) {
        this.messageHandlers.delete(type)
      }
    }
  }

  private handleOpen(): void {
    this.state.connected = true
    this.state.connecting = false
    this.state.reconnectCount = 0
    this.state.lastConnectedAt = new Date().toISOString()
    this.state.error = null

    if (this.config.debug) {
      console.log('[WebSocket] Connected')
    }

    // Send queued messages
    if (this.messageQueue.length > 0) {
      this.messageQueue.forEach(message => {
        this.send(message.type, message.data)
      })
      this.messageQueue = []
    }

    // Start heartbeat
    if (this.config.heartbeat) {
      this.startHeartbeat()
    }

    // Emit connected event
    this.emit('connected', null)

    ElNotification({
      title: 'WebSocket连接成功',
      message: '实时通信已建立',
      type: 'success',
      duration: 3000
    })
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const message: WebSocketMessage = JSON.parse(event.data)

      if (this.config.debug) {
        console.log('[WebSocket] Received message:', message)
      }

      // Handle heartbeat response
      if (message.type === 'pong') {
        return
      }

      this.emit(message.type, message.data)
    } catch (error) {
      if (this.config.debug) {
        console.error('[WebSocket] Message parse error:', error)
      }
    }
  }

  private handleClose(event: CloseEvent): void {
    this.state.connected = false
    this.state.connecting = false
    this.state.lastDisconnectedAt = new Date().toISOString()

    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }

    if (this.config.debug) {
      console.log('[WebSocket] Disconnected:', event.code, event.reason)
    }

    // Emit disconnected event
    this.emit('disconnected', { code: event.code, reason: event.reason })

    // Handle reconnection
    if (this.config.reconnect && event.code !== 1000) {
      this.handleReconnect()
    } else {
      ElNotification({
        title: 'WebSocket连接断开',
        message: event.reason || '连接已关闭',
        type: 'info',
        duration: 3000
      })
    }
  }

  private handleError(event: Event): void {
    const error = 'WebSocket连接错误'
    this.state.error = error
    this.state.connecting = false

    if (this.config.debug) {
      console.error('[WebSocket] Error:', event)
    }

    this.emit('error', error)

    ElMessage.error(error)
  }

  private handleReconnect(): void {
    if (this.state.reconnectCount >= this.config.maxReconnectAttempts) {
      this.state.error = '重连次数超过限制'
      ElNotification({
        title: '连接失败',
        message: '无法重新建立WebSocket连接，请刷新页面重试',
        type: 'error',
        duration: 0
      })
      return
    }

    this.state.reconnectCount++

    if (this.config.debug) {
      console.log(`[WebSocket] Attempting reconnection ${this.state.reconnectCount}/${this.config.maxReconnectAttempts}`)
    }

    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch(error => {
        if (this.config.debug) {
          console.error('[WebSocket] Reconnection failed:', error)
        }
      })
    }, this.config.reconnectInterval)

    ElMessage.warning(`连接断开，正在重连... (${this.state.reconnectCount}/${this.config.maxReconnectAttempts})`)
  }

  private startHeartbeat(): void {
    this.heartbeatTimer = window.setInterval(() => {
      if (this.state.connected) {
        this.send('ping', { timestamp: Date.now() })
      }
    }, this.config.heartbeatInterval)
  }

  private emit(type: string, data: any): void {
    const handlers = this.messageHandlers.get(type)
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(data)
        } catch (error) {
          if (this.config.debug) {
            console.error('[WebSocket] Handler error:', error)
          }
        }
      })
    }
  }

  private generateMessageId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }
}

// Composable for WebSocket usage
export function useWebSocket(key = 'default') {
  const manager = WebSocketManager.getInstance()
  let connection = manager.getConnection(key)

  if (!connection) {
    // Create a placeholder connection that will be initialized when connect is called
    connection = new WebSocketConnection({ url: '' })
    manager.setConnection(key, connection)
  }

  const state: Ref<WebSocketState> = ref(connection.getState())

  // Methods
  const connect = async (url: string, config: Partial<WebSocketConfig> = {}): Promise<void> => {
    // Create new connection with proper URL
    connection = new WebSocketConnection({ url, ...config })
    manager.setConnection(key, connection)
    state.value = connection.getState()

    await connection.connect()
  }

  const disconnect = (): void => {
    connection?.disconnect()
  }

  const send = (type: string, data: any): boolean => {
    return connection?.send(type, data) || false
  }

  const onMessage = (type: string, handler: Function): void => {
    connection?.on(type, handler)
  }

  const offMessage = (type: string, handler?: Function): void => {
    connection?.off(type, handler)
  }

  // Cleanup on unmount
  onUnmounted(() => {
    // Don't automatically disconnect as other components might be using the same connection
    // manager.removeConnection(key)
  })

  return {
    state: readonly(state),
    connect,
    disconnect,
    send,
    onMessage,
    offMessage
  }
}

// Specialized WebSocket hooks for different purposes

// Job processing WebSocket
export function useJobWebSocket() {
  const { state, connect, disconnect, send, onMessage, offMessage } = useWebSocket('job')

  const connectToJobService = async (): Promise<void> => {
    await connect('ws://localhost:8080/ws/job', {
      heartbeat: true,
      heartbeatInterval: 30000,
      reconnect: true,
      reconnectInterval: 5000,
      debug: import.meta.env.DEV
    })
  }

  const subscribeToJob = (jobId: string): void => {
    send('subscribe_job', { jobId })
  }

  const unsubscribeFromJob = (jobId: string): void => {
    send('unsubscribe_job', { jobId })
  }

  const startJob = (jobId: string, settings: any): void => {
    send('start_job', { jobId, settings })
  }

  const pauseJob = (jobId: string): void => {
    send('pause_job', { jobId })
  }

  const stopJob = (jobId: string): void => {
    send('stop_job', { jobId })
  }

  return {
    state,
    connect: connectToJobService,
    disconnect,
    subscribeToJob,
    unsubscribeFromJob,
    startJob,
    pauseJob,
    stopJob,
    onMessage,
    offMessage
  }
}

// System monitoring WebSocket
export function useSystemWebSocket() {
  const { state, connect, disconnect, send, onMessage, offMessage } = useWebSocket('system')

  const connectToSystemService = async (): Promise<void> => {
    await connect('ws://localhost:8080/ws/system', {
      heartbeat: true,
      heartbeatInterval: 15000,
      reconnect: true,
      reconnectInterval: 3000,
      debug: import.meta.env.DEV
    })
  }

  const subscribeToMetrics = (): void => {
    send('subscribe_metrics', {})
  }

  const subscribeToEvents = (): void => {
    send('subscribe_events', {})
  }

  const subscribeToAlerts = (): void => {
    send('subscribe_alerts', {})
  }

  return {
    state,
    connect: connectToSystemService,
    disconnect,
    subscribeToMetrics,
    subscribeToEvents,
    subscribeToAlerts,
    onMessage,
    offMessage
  }
}

// Live camera WebSocket
export function useCameraWebSocket() {
  const { state, connect, disconnect, send, onMessage, offMessage } = useWebSocket('camera')

  const connectToCameraService = async (): Promise<void> => {
    await connect('ws://localhost:8080/ws/camera', {
      heartbeat: true,
      heartbeatInterval: 10000,
      reconnect: true,
      reconnectInterval: 2000,
      debug: import.meta.env.DEV
    })
  }

  const startStream = (cameraId: string, settings: any): void => {
    send('start_stream', { cameraId, settings })
  }

  const stopStream = (cameraId: string): void => {
    send('stop_stream', { cameraId })
  }

  const captureFrame = (cameraId: string): void => {
    send('capture_frame', { cameraId })
  }

  const adjustSettings = (cameraId: string, settings: any): void => {
    send('adjust_settings', { cameraId, settings })
  }

  return {
    state,
    connect: connectToCameraService,
    disconnect,
    startStream,
    stopStream,
    captureFrame,
    adjustSettings,
    onMessage,
    offMessage
  }
}

// Global cleanup function
export function disconnectAllWebSockets(): void {
  WebSocketManager.getInstance().disconnectAll()
}

// Export types and classes for advanced usage
export { WebSocketConnection, WebSocketManager }
