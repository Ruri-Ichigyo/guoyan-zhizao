import { http } from './http'
import type { ChatMessage, ChatSession } from '../stores/chatStore'

// API Request/Response Types
export interface CreateSessionRequest {
  title?: string
}

export interface CreateSessionResponse {
  session_id: string
  created_at: string
}

export interface SendMessageRequest {
  session_id: string
  content: string
}

export interface SendMessageResponse {
  message_id: string
  session_id: string
  role: string
  content: string
  metadata?: Record<string, any>
  created_at: string
}

export interface ChatHistoryResponse {
  messages: ChatMessage[]
  session: ChatSession
}

export interface ListSessionsResponse {
  sessions: ChatSession[]
  total: number
}

export interface PollResponse {
  messages: ChatMessage[]
  has_more: boolean
}

/**
 * Create a new chat session
 */
export const createSession = async (
  request?: CreateSessionRequest
): Promise<CreateSessionResponse> => {
  const response = await http.post<CreateSessionResponse>('/chat/sessions', request || {})
  return response.data
}

/**
 * Send a message to the AI assistant
 */
export const sendMessage = async (
  request: SendMessageRequest
): Promise<SendMessageResponse> => {
  const response = await http.post<SendMessageResponse>('/chat/message', request)
  return response.data
}

/**
 * Get conversation history
 */
export const getChatHistory = async (
  sessionId: string,
  limit = 20,
  offset = 0
): Promise<ChatHistoryResponse> => {
  const response = await http.get<ChatHistoryResponse>(`/chat/history/${sessionId}`, {
    params: { limit, offset },
  })
  return response.data
}

/**
 * List user's chat sessions
 */
export const listSessions = async (
  status?: 'active' | 'inactive' | 'archived',
  limit = 10,
  offset = 0
): Promise<ListSessionsResponse> => {
  const response = await http.get<ListSessionsResponse>('/chat/sessions', {
    params: { status, limit, offset },
  })
  return response.data
}

/**
 * Delete a chat session
 */
export const deleteSession = async (sessionId: string): Promise<void> => {
  await http.delete(`/chat/sessions/${sessionId}`)
}

/**
 * Poll for new messages (used by HTTP polling)
 */
export const pollNewMessages = async (
  sessionId: string,
  afterMessageId?: string
): Promise<PollResponse> => {
  const response = await http.get<PollResponse>('/chat/messages/new', {
    params: {
      session_id: sessionId,
      after: afterMessageId,
    },
  })
  return response.data
}

// Export all chat API functions
export default {
  createSession,
  sendMessage,
  getChatHistory,
  listSessions,
  deleteSession,
  pollNewMessages,
}
