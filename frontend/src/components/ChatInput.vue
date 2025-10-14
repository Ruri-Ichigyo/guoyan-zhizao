<template>
  <div class="chat-input">
    <el-input
      v-model="inputText"
      type="textarea"
      :rows="1"
      :autosize="{ minRows: 1, maxRows: 4 }"
      placeholder="输入消息... (Enter发送，Shift+Enter换行)"
      @keydown.enter="handleKeyDown"
      :disabled="disabled"
      class="input-field"
    />
    <div class="input-actions">
      <span class="char-count" :class="{ 'char-limit': isNearLimit }">
        {{ inputText.length }}/10000
      </span>
      <el-button
        type="primary"
        @click="handleSend"
        :loading="sending"
        :disabled="disabled || !canSend"
        size="small"
        class="send-button"
      >
        <template #icon>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            style="width: 16px; height: 16px"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5"
            />
          </svg>
        </template>
        发送
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  disabled?: boolean
  sending?: boolean
}

interface Emits {
  (e: 'send', message: string): void
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  sending: false,
})

const emit = defineEmits<Emits>()

const inputText = ref('')

const canSend = computed(() => {
  const trimmed = inputText.value.trim()
  return trimmed.length > 0 && trimmed.length <= 10000
})

const isNearLimit = computed(() => {
  return inputText.value.length > 9000
})

const handleSend = () => {
  if (!canSend.value || props.sending) {
    return
  }

  const message = inputText.value.trim()
  if (message) {
    emit('send', message)
    inputText.value = ''
  }
}

const handleKeyDown = (event: KeyboardEvent) => {
  // Shift+Enter: new line (default behavior)
  if (event.shiftKey) {
    return
  }

  // Enter alone: send message
  event.preventDefault()
  handleSend()
}
</script>

<style scoped>
.chat-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background-color: #ffffff;
  border-top: 1px solid #e4e7ed;
}

.input-field {
  width: 100%;
}

.input-field :deep(.el-textarea__inner) {
  resize: none;
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 14px;
  line-height: 1.5;
  border: 1px solid #dcdfe6;
  transition: border-color 0.3s;
}

.input-field :deep(.el-textarea__inner):focus {
  border-color: #67c23a;
  box-shadow: 0 0 0 2px rgba(103, 194, 58, 0.1);
}

.input-field :deep(.el-textarea__inner:disabled) {
  background-color: #f5f7fa;
  cursor: not-allowed;
}

.input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.char-count {
  font-size: 12px;
  color: #909399;
  transition: color 0.3s;
}

.char-count.char-limit {
  color: #f56c6c;
  font-weight: bold;
}

.send-button {
  flex-shrink: 0;
}

.send-button :deep(.el-button__text) {
  display: flex;
  align-items: center;
  gap: 4px;
}

@media (max-width: 768px) {
  .chat-input {
    padding: 12px;
  }

  .input-field :deep(.el-textarea__inner) {
    font-size: 13px;
  }

  .send-button {
    min-width: 60px;
  }
}
</style>
