import { ref, watch, onUnmounted } from 'vue'
import { useChatStore } from '../stores/chatStore'
import * as chatApi from '../api/chat'

const POLLING_INTERVAL = parseInt(import.meta.env.VITE_CHAT_POLLING_INTERVAL || '2000')

export function useChatPolling() {
  const chatStore = useChatStore()
  const isPolling = ref(false)
  let pollingTimer: number | null = null
  let lastMessageId = ref<string | undefined>(undefined)

  /**
   * Start polling for new messages
   */
  const startPolling = () => {
    if (isPolling.value || !chatStore.currentSession) {
      return
    }

    isPolling.value = true

    // Set initial lastMessageId to the most recent message
    if (chatStore.messages.length > 0) {
      lastMessageId.value = chatStore.messages[chatStore.messages.length - 1].id
    }

    poll()
  }

  /**
   * Stop polling
   */
  const stopPolling = () => {
    isPolling.value = false
    if (pollingTimer) {
      clearTimeout(pollingTimer)
      pollingTimer = null
    }
  }

  /**
   * Poll for new messages
   */
  const poll = async () => {
    if (!isPolling.value || !chatStore.currentSession) {
      return
    }

    try {
      const response = await chatApi.pollNewMessages(
        chatStore.currentSession.id,
        lastMessageId.value
      )

      // Add new messages to store
      if (response.messages && response.messages.length > 0) {
        response.messages.forEach((message) => {
          // Only add if message doesn't already exist
          const exists = chatStore.messages.some(m => m.id === message.id)
          if (!exists) {
            chatStore.addMessage(message)
          }
        })

        // Update last message ID
        lastMessageId.value = response.messages[response.messages.length - 1].id
      }
    } catch (error: any) {
      // Silently fail for polling errors to avoid spamming notifications
      // Only log to console
      if (error.response?.status !== 404) {
        console.error('Polling error:', error)
      }
    } finally {
      // Schedule next poll if still polling
      if (isPolling.value) {
        pollingTimer = window.setTimeout(poll, POLLING_INTERVAL)
      }
    }
  }

  /**
   * Reset polling state when session changes
   */
  watch(
    () => chatStore.currentSession,
    (newSession, oldSession) => {
      if (newSession?.id !== oldSession?.id) {
        stopPolling()
        lastMessageId.value = undefined

        // Auto-start polling if there's a new session and chat is open
        if (newSession && chatStore.isOpen) {
          startPolling()
        }
      }
    }
  )

  /**
   * Start/stop polling based on chat window state
   */
  watch(
    () => chatStore.isOpen,
    (isOpen) => {
      if (isOpen && chatStore.currentSession) {
        startPolling()
      } else {
        stopPolling()
      }
    }
  )

  /**
   * Clean up on component unmount
   */
  onUnmounted(() => {
    stopPolling()
  })

  return {
    isPolling,
    startPolling,
    stopPolling,
  }
}
