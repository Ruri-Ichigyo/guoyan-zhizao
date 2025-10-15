import { ref, onUnmounted, type Ref } from 'vue'
import { ElMessage } from 'element-plus'

export interface WebSocketState {
  connected: boolean
  error: string | null
}

export interface WebSocketInstance {
  connect: (url: string) => Promise<void>
  disconnect: () => void
  send: (event: string, data: any) => void
  onMessage: (event: string, callback: (data: any) => void) => void
  state: Ref<WebSocketState>
}

const connections = new Map<string, WebSocket>()
const messageHandlers = new Map<string, Map<string, Array<(data: any) => void>>>()

export function useWebSocket(namespace: string = 'default'): WebSocketInstance {
  const state = ref<WebSocketState>({
    connected: false,
    error: null
  })

  let ws: WebSocket | null = null
  let reconnectTimer: number | null = null
  let reconnectAttempts = 0
  const maxReconnectAttempts = 5

  const connect = async (url: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      try {
        ws = new WebSocket(url)
        connections.set(namespace, ws)

        ws.onopen = () => {
          state.value.connected = true
          state.value.error = null
          reconnectAttempts = 0
          console.log(`[WebSocket] Connected to ${url}`)
          resolve()
        }

        ws.onclose = () => {
          state.value.connected = false
          console.log(`[WebSocket] Disconnected from ${url}`)

          // Attempt to reconnect
          if (reconnectAttempts < maxReconnectAttempts) {
            reconnectAttempts++
            reconnectTimer = window.setTimeout(() => {
              console.log(`[WebSocket] Reconnecting... (attempt ${reconnectAttempts})`)
              connect(url)
            }, 3000)
          } else {
            state.value.error = 'WebSocket连接失败，已达到最大重连次数'
            ElMessage.error('WebSocket连接失败')
          }
        }

        ws.onerror = (error) => {
          state.value.error = 'WebSocket连接错误'
          console.error('[WebSocket] Error:', error)
          reject(error)
        }

        ws.onmessage = (event) => {
          try {
            const message = JSON.parse(event.data)
            const handlers = messageHandlers.get(namespace)?.get(message.event)
            if (handlers) {
              handlers.forEach(handler => handler(message.data))
            }
          } catch (error) {
            console.error('[WebSocket] Failed to parse message:', error)
          }
        }
      } catch (error) {
        state.value.error = 'Failed to create WebSocket connection'
        reject(error)
      }
    })
  }

  const disconnect = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }

    if (ws) {
      ws.close()
      ws = null
      connections.delete(namespace)
    }

    state.value.connected = false
  }

  const send = (event: string, data: any) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ event, data }))
    } else {
      console.warn('[WebSocket] Cannot send message, not connected')
    }
  }

  const onMessage = (event: string, callback: (data: any) => void) => {
    if (!messageHandlers.has(namespace)) {
      messageHandlers.set(namespace, new Map())
    }

    const namespaceHandlers = messageHandlers.get(namespace)!
    if (!namespaceHandlers.has(event)) {
      namespaceHandlers.set(event, [])
    }

    namespaceHandlers.get(event)!.push(callback)
  }

  // Cleanup on unmount
  onUnmounted(() => {
    disconnect()
  })

  return {
    connect,
    disconnect,
    send,
    onMessage,
    state
  }
}

export function disconnectAllWebSockets() {
  connections.forEach((ws) => {
    ws.close()
  })
  connections.clear()
  messageHandlers.clear()
}
