<template>
  <div :class="['chat-message', `message-${message.role}`]">
    <div class="message-avatar">
      <div v-if="message.role === 'assistant'" class="avatar-ai">AI</div>
      <div v-else class="avatar-user">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
          />
        </svg>
      </div>
    </div>

    <div class="message-content">
      <div class="message-bubble">
        <div class="message-text" v-html="formattedContent"></div>
      </div>
      <div class="message-time">{{ formattedTime }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChatMessage } from '../stores/chatStore'

interface Props {
  message: ChatMessage
}

const props = defineProps<Props>()

// Format timestamp
const formattedTime = computed(() => {
  const date = new Date(props.message.created_at)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  // Less than 1 minute
  if (diff < 60000) {
    return '刚刚'
  }

  // Less than 1 hour
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)}分钟前`
  }

  // Today
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }

  // This year
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  }

  // Other years
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
})

// Simple markdown-like formatting for content
const formattedContent = computed(() => {
  let content = props.message.content

  // Escape HTML
  content = content.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

  // Convert line breaks
  content = content.replace(/\n/g, '<br>')

  // Bold text **text**
  content = content.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')

  // Italic text *text*
  content = content.replace(/\*(.*?)\*/g, '<em>$1</em>')

  // Code blocks ```code```
  content = content.replace(/```(.*?)```/gs, '<pre><code>$1</code></pre>')

  // Inline code `code`
  content = content.replace(/`(.*?)`/g, '<code>$1</code>')

  return content
})
</script>

<style scoped>
.chat-message {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-assistant {
  flex-direction: row;
}

.message-user {
  flex-direction: row-reverse;
}

.message-avatar {
  flex-shrink: 0;
}

.avatar-ai,
.avatar-user {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: bold;
  color: white;
}

.avatar-ai {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.avatar-user {
  background: linear-gradient(135deg, #84fab0 0%, #8fd3f4 100%);
}

.avatar-user svg {
  width: 20px;
  height: 20px;
}

.message-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 70%;
}

.message-user .message-content {
  align-items: flex-end;
}

.message-assistant .message-content {
  align-items: flex-start;
}

.message-bubble {
  padding: 12px 16px;
  border-radius: 12px;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.message-assistant .message-bubble {
  background-color: #f5f7fa;
  color: #303133;
  border-bottom-left-radius: 4px;
}

.message-user .message-bubble {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-bottom-right-radius: 4px;
}

.message-text {
  line-height: 1.6;
  font-size: 14px;
}

.message-text :deep(pre) {
  background-color: rgba(0, 0, 0, 0.1);
  padding: 8px;
  border-radius: 4px;
  overflow-x: auto;
  margin: 8px 0;
}

.message-text :deep(code) {
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.message-text :deep(strong) {
  font-weight: bold;
}

.message-text :deep(em) {
  font-style: italic;
}

.message-time {
  font-size: 12px;
  color: #909399;
  padding: 0 4px;
}

@media (max-width: 768px) {
  .message-content {
    max-width: 80%;
  }

  .avatar-ai,
  .avatar-user {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }

  .message-bubble {
    padding: 10px 14px;
  }

  .message-text {
    font-size: 13px;
  }
}
</style>
