import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface ChatMessage {
  id: string
  session_id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  metadata?: Record<string, any>
  created_at: string
}

export interface ChatSession {
  id: string
  user_id: string
  title: string | null
  status: 'active' | 'inactive' | 'archived'
  created_at: string
  updated_at: string
}

export const useChatStore = defineStore('chat', () => {
  // State
  const currentSession = ref<ChatSession | null>(null)
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const unreadCount = ref(0)
  const isOpen = ref(false)
  const sessions = ref<ChatSession[]>([])

  // Getters
  const hasActiveSession = computed(() => currentSession.value !== null)
  const lastMessage = computed(() => {
    return messages.value.length > 0
      ? messages.value[messages.value.length - 1]
      : null
  })

  // Actions
  function setCurrentSession(session: ChatSession | null) {
    currentSession.value = session
    if (session) {
      // Clear messages when switching sessions
      messages.value = []
    }
  }

  function addMessage(message: ChatMessage) {
    messages.value.push(message)

    // Increment unread count if chat is closed and message is from assistant
    if (!isOpen.value && message.role === 'assistant') {
      unreadCount.value++
    }
  }

  function setMessages(newMessages: ChatMessage[]) {
    messages.value = newMessages
  }

  function clearMessages() {
    messages.value = []
  }

  function markAsRead() {
    unreadCount.value = 0
  }

  function toggleChat() {
    isOpen.value = !isOpen.value
    if (isOpen.value) {
      markAsRead()
    }
  }

  function openChat() {
    isOpen.value = true
    markAsRead()
  }

  function closeChat() {
    isOpen.value = false
  }

  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  function addSession(session: ChatSession) {
    // Add session to the beginning of the list
    sessions.value.unshift(session)
  }

  function setSessions(newSessions: ChatSession[]) {
    sessions.value = newSessions
  }

  function removeSession(sessionId: string) {
    sessions.value = sessions.value.filter(s => s.id !== sessionId)

    // If the removed session was the current one, clear it
    if (currentSession.value?.id === sessionId) {
      currentSession.value = null
      messages.value = []
    }
  }

  function updateSessionTitle(sessionId: string, title: string) {
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) {
      session.title = title
    }
    if (currentSession.value?.id === sessionId) {
      currentSession.value.title = title
    }
  }

  // Reset store (useful for logout)
  function $reset() {
    currentSession.value = null
    messages.value = []
    isLoading.value = false
    unreadCount.value = 0
    isOpen.value = false
    sessions.value = []
  }

  return {
    // State
    currentSession,
    messages,
    isLoading,
    unreadCount,
    isOpen,
    sessions,

    // Getters
    hasActiveSession,
    lastMessage,

    // Actions
    setCurrentSession,
    addMessage,
    setMessages,
    clearMessages,
    markAsRead,
    toggleChat,
    openChat,
    closeChat,
    setLoading,
    addSession,
    setSessions,
    removeSession,
    updateSessionTitle,
    $reset,
  }
})
