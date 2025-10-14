import { ref } from 'vue'
import { useChatStore } from '../stores/chatStore'
import * as chatApi from '../api/chat'
import { ElNotification } from 'element-plus'

export function useChat() {
  const chatStore = useChatStore()
  const sending = ref(false)
  const loading = ref(false)

  /**
   * Create a new chat session
   */
  const createSession = async (title?: string) => {
    try {
      loading.value = true
      const response = await chatApi.createSession({ title })

      // Create session object
      const newSession = {
        id: response.session_id,
        user_id: '', // Will be filled by backend
        title: title || null,
        status: 'active' as const,
        created_at: response.created_at,
        updated_at: response.created_at,
      }

      chatStore.setCurrentSession(newSession)
      chatStore.addSession(newSession)

      return newSession
    } catch (error: any) {
      ElNotification({
        title: '创建会话失败',
        message: error.response?.data?.message || '无法创建新的聊天会话',
        type: 'error',
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * Send a message to the AI assistant
   */
  const sendMessage = async (content: string) => {
    if (!content.trim()) {
      ElNotification({
        title: '提示',
        message: '消息内容不能为空',
        type: 'warning',
      })
      return
    }

    // Create session if none exists
    if (!chatStore.currentSession) {
      await createSession()
    }

    if (!chatStore.currentSession) {
      throw new Error('Failed to create session')
    }

    try {
      sending.value = true
      chatStore.setLoading(true)

      // Add user message to store immediately (optimistic update)
      const userMessage = {
        id: `temp-${Date.now()}`,
        session_id: chatStore.currentSession.id,
        role: 'user' as const,
        content,
        created_at: new Date().toISOString(),
      }
      chatStore.addMessage(userMessage)

      // Send message to backend
      const response = await chatApi.sendMessage({
        session_id: chatStore.currentSession.id,
        content,
      })

      // Update user message with real ID
      const messages = chatStore.messages
      const userMsgIndex = messages.findIndex(m => m.id === userMessage.id)
      if (userMsgIndex !== -1) {
        // Remove the temp message (it was already sent and has a real ID now)
        messages.splice(userMsgIndex, 1)
      }

      // Add assistant response
      chatStore.addMessage({
        id: response.message_id,
        session_id: response.session_id,
        role: response.role as any,
        content: response.content,
        metadata: response.metadata,
        created_at: response.created_at,
      })

      return response
    } catch (error: any) {
      // Remove optimistic user message on error
      const messages = chatStore.messages
      const tempMsgIndex = messages.findIndex(m => m.id.startsWith('temp-'))
      if (tempMsgIndex !== -1) {
        messages.splice(tempMsgIndex, 1)
      }

      const errorMessage = error.response?.data?.message || '发送消息失败'

      if (error.response?.status === 429) {
        ElNotification({
          title: '请求过于频繁',
          message: '请稍后再试',
          type: 'warning',
        })
      } else if (error.code === 'ECONNABORTED' || error.message.includes('timeout')) {
        ElNotification({
          title: 'AI助手响应超时',
          message: '请稍后重试',
          type: 'error',
        })
      } else {
        ElNotification({
          title: '发送失败',
          message: errorMessage,
          type: 'error',
        })
      }

      throw error
    } finally {
      sending.value = false
      chatStore.setLoading(false)
    }
  }

  /**
   * Load chat history for a session
   */
  const loadHistory = async (sessionId: string, limit = 20, offset = 0) => {
    try {
      loading.value = true
      const response = await chatApi.getChatHistory(sessionId, limit, offset)

      chatStore.setCurrentSession(response.session)
      chatStore.setMessages(response.messages)

      return response
    } catch (error: any) {
      ElNotification({
        title: '加载历史记录失败',
        message: error.response?.data?.message || '无法加载聊天历史',
        type: 'error',
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * Load user's session list
   */
  const loadSessions = async (
    status?: 'active' | 'inactive' | 'archived',
    limit = 10,
    offset = 0
  ) => {
    try {
      loading.value = true
      const response = await chatApi.listSessions(status, limit, offset)

      chatStore.setSessions(response.sessions)

      return response
    } catch (error: any) {
      ElNotification({
        title: '加载会话列表失败',
        message: error.response?.data?.message || '无法加载会话列表',
        type: 'error',
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * Delete a session
   */
  const deleteSession = async (sessionId: string) => {
    try {
      await chatApi.deleteSession(sessionId)
      chatStore.removeSession(sessionId)

      ElNotification({
        title: '删除成功',
        message: '会话已删除',
        type: 'success',
      })
    } catch (error: any) {
      ElNotification({
        title: '删除失败',
        message: error.response?.data?.message || '无法删除会话',
        type: 'error',
      })
      throw error
    }
  }

  return {
    sending,
    loading,
    createSession,
    sendMessage,
    loadHistory,
    loadSessions,
    deleteSession,
  }
}
