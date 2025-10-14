<template>
  <div ref="windowRef" class="chat-window" :style="windowStyle">
    <!-- Header (draggable) -->
    <div
      class="chat-header"
      @mousedown="handleMouseDown"
      @touchstart="handleTouchStart"
    >
      <div class="header-title">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="header-icon"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z"
          />
        </svg>
        <span>果研助手</span>
        <span v-if="chatStore.currentSession?.title" class="session-title">
          - {{ chatStore.currentSession.title }}
        </span>
      </div>
      <div class="header-actions">
        <el-button text @click="handleMinimize" class="minimize-button">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
            style="width: 20px; height: 20px"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M19.5 12h-15"
            />
          </svg>
        </el-button>
      </div>
    </div>

    <!-- Messages Area -->
    <div class="chat-body">
      <el-scrollbar ref="scrollbarRef" class="message-scroll">
        <div class="message-list" ref="messageListRef">
          <!-- Empty state -->
          <div v-if="messages.length === 0" class="empty-state">
            <div class="empty-icon">
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
                  d="M20.25 8.511c.884.284 1.5 1.128 1.5 2.097v4.286c0 1.136-.847 2.1-1.98 2.193-.34.027-.68.052-1.02.072v3.091l-3-3c-1.354 0-2.694-.055-4.02-.163a2.115 2.115 0 01-.825-.242m9.345-8.334a2.126 2.126 0 00-.476-.095 48.64 48.64 0 00-8.048 0c-1.131.094-1.976 1.057-1.976 2.192v4.286c0 .837.46 1.58 1.155 1.951m9.345-8.334V6.637c0-1.621-1.152-3.026-2.76-3.235A48.455 48.455 0 0011.25 3c-2.115 0-4.198.137-6.24.402-1.608.209-2.76 1.614-2.76 3.235v6.226c0 1.621 1.152 3.026 2.76 3.235.577.075 1.157.14 1.74.194V21l4.155-4.155"
                />
              </svg>
            </div>
            <p class="empty-text">开始新对话</p>
            <p class="empty-hint">向果研助手提问，查询识别记录或获取统计信息</p>
          </div>

          <!-- Messages -->
          <ChatMessage
            v-for="message in messages"
            :key="message.id"
            :message="message"
          />

          <!-- Loading indicator -->
          <div v-if="chatStore.isLoading" class="loading-indicator">
            <div class="typing-animation">
              <span></span>
              <span></span>
              <span></span>
            </div>
            <span class="loading-text">小果正在思考...</span>
          </div>
        </div>
      </el-scrollbar>
    </div>

    <!-- Input Area -->
    <ChatInput
      :disabled="chatStore.isLoading"
      :sending="sending"
      @send="handleSendMessage"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useChatStore } from "../stores/chatStore";
import { useChat } from "../composables/useChat";
import { useChatPolling } from "../composables/useChatPolling";
import ChatMessage from "./ChatMessage.vue";
import ChatInput from "./ChatInput.vue";

const chatStore = useChatStore();
const { sendMessage, sending } = useChat();
const { startPolling } = useChatPolling();

const scrollbarRef = ref();
const messageListRef = ref();
const windowRef = ref<HTMLElement | null>(null);

// Dragging state
const position = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0, windowX: 0, windowY: 0 });

const messages = computed(() => chatStore.messages);

// Window style for positioning
const windowStyle = computed(() => {
  if (position.value.x === 0 && position.value.y === 0) {
    return {}; // Use default CSS positioning
  }
  return {
    right: 'auto',
    bottom: 'auto',
    left: `${position.value.x}px`,
    top: `${position.value.y}px`,
  };
});

// Mouse drag handlers
const handleMouseDown = (e: MouseEvent) => {
  // Don't drag if clicking on buttons
  if ((e.target as HTMLElement).closest('.minimize-button')) {
    return;
  }

  isDragging.value = false;
  const rect = windowRef.value?.getBoundingClientRect();
  if (rect) {
    dragStart.value = {
      x: e.clientX,
      y: e.clientY,
      windowX: rect.left,
      windowY: rect.top
    };
  }

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
  e.preventDefault();
};

const handleMouseMove = (e: MouseEvent) => {
  const deltaX = e.clientX - dragStart.value.x;
  const deltaY = e.clientY - dragStart.value.y;

  // Start dragging if moved more than 5px
  if (!isDragging.value && (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5)) {
    isDragging.value = true;
  }

  if (isDragging.value && windowRef.value) {
    const newX = dragStart.value.windowX + deltaX;
    const newY = dragStart.value.windowY + deltaY;

    // Get window dimensions
    const rect = windowRef.value.getBoundingClientRect();
    const maxX = window.innerWidth - rect.width;
    const maxY = window.innerHeight - rect.height;

    // Constrain to viewport
    position.value = {
      x: Math.max(0, Math.min(newX, maxX)),
      y: Math.max(0, Math.min(newY, maxY))
    };

    // Save position
    savePosition();
  }
};

const handleMouseUp = () => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);

  setTimeout(() => {
    isDragging.value = false;
  }, 100);
};

// Touch drag handlers
const handleTouchStart = (e: TouchEvent) => {
  if ((e.target as HTMLElement).closest('.minimize-button')) {
    return;
  }

  if (e.touches.length !== 1) return;

  isDragging.value = false;
  const touch = e.touches[0];
  const rect = windowRef.value?.getBoundingClientRect();
  if (rect) {
    dragStart.value = {
      x: touch.clientX,
      y: touch.clientY,
      windowX: rect.left,
      windowY: rect.top
    };
  }

  document.addEventListener('touchmove', handleTouchMove, { passive: false });
  document.addEventListener('touchend', handleTouchEnd);
  e.preventDefault();
};

const handleTouchMove = (e: TouchEvent) => {
  if (e.touches.length !== 1) return;

  const touch = e.touches[0];
  const deltaX = touch.clientX - dragStart.value.x;
  const deltaY = touch.clientY - dragStart.value.y;

  if (!isDragging.value && (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5)) {
    isDragging.value = true;
  }

  if (isDragging.value && windowRef.value) {
    const newX = dragStart.value.windowX + deltaX;
    const newY = dragStart.value.windowY + deltaY;

    const rect = windowRef.value.getBoundingClientRect();
    const maxX = window.innerWidth - rect.width;
    const maxY = window.innerHeight - rect.height;

    position.value = {
      x: Math.max(0, Math.min(newX, maxX)),
      y: Math.max(0, Math.min(newY, maxY))
    };

    savePosition();
  }

  e.preventDefault();
};

const handleTouchEnd = () => {
  document.removeEventListener('touchmove', handleTouchMove);
  document.removeEventListener('touchend', handleTouchEnd);

  setTimeout(() => {
    isDragging.value = false;
  }, 100);
};

// Save and restore position
const savePosition = () => {
  localStorage.setItem('chatWindowPosition', JSON.stringify(position.value));
};

const restorePosition = () => {
  const saved = localStorage.getItem('chatWindowPosition');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      // Validate position is within viewport
      if (windowRef.value) {
        const rect = windowRef.value.getBoundingClientRect();
        const maxX = window.innerWidth - rect.width;
        const maxY = window.innerHeight - rect.height;

        position.value = {
          x: Math.max(0, Math.min(parsed.x, maxX)),
          y: Math.max(0, Math.min(parsed.y, maxY))
        };
      }
    } catch (e) {
      // Invalid saved position
    }
  }
};

const handleMinimize = () => {
  chatStore.closeChat();
};

const handleSendMessage = async (content: string) => {
  try {
    await sendMessage(content);
    await scrollToBottom();
  } catch (error) {
    console.error("Failed to send message:", error);
  }
};

const scrollToBottom = async () => {
  await nextTick();
  if (scrollbarRef.value) {
    scrollbarRef.value.setScrollTop(messageListRef.value?.scrollHeight || 0);
  }
};

// Auto-scroll when new messages arrive
watch(
  () => messages.value.length,
  async () => {
    await scrollToBottom();
  }
);

// Start polling when component mounts
onMounted(() => {
  startPolling();
  scrollToBottom();
  nextTick(() => restorePosition());
});

// Cleanup
onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  document.removeEventListener('touchmove', handleTouchMove);
  document.removeEventListener('touchend', handleTouchEnd);
});
</script>

<style scoped>
.chat-window {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 400px;
  height: 600px;
  max-height: calc(100vh - 40px);
  background-color: #ffffff;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 1000;
  user-select: none;
}

.chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: linear-gradient(135deg, #67c23a 0%, #529b2e 100%);
  color: white;
  border-radius: 16px 16px 0 0;
  cursor: move;
  cursor: grab;
}

.chat-header:active {
  cursor: grabbing;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 16px;
}

.header-icon {
  width: 24px;
  height: 24px;
}

.session-title {
  font-weight: 400;
  font-size: 14px;
  opacity: 0.9;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  gap: 4px;
}

.minimize-button {
  color: white;
  padding: 4px;
}

.minimize-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.chat-body {
  flex: 1;
  overflow: hidden;
  background-color: #f5f7fa;
}

.message-scroll {
  height: 100%;
}

.message-list {
  padding: 16px;
  min-height: 100%;
  display: flex;
  flex-direction: column;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 40px 20px;
  text-align: center;
  color: #909399;
}

.empty-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-icon svg {
  width: 100%;
  height: 100%;
}

.empty-text {
  font-size: 18px;
  font-weight: 500;
  color: #606266;
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 14px;
  color: #909399;
  line-height: 1.5;
}

.loading-indicator {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  margin-top: 8px;
}

.typing-animation {
  display: flex;
  gap: 4px;
}

.typing-animation span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #67c23a;
  animation: typing 1.4s infinite;
}

.typing-animation span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-animation span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing {
  0%,
  60%,
  100% {
    transform: translateY(0);
    opacity: 0.7;
  }
  30% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

.loading-text {
  font-size: 14px;
  color: #909399;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .chat-window {
    width: calc(100vw - 40px);
    height: calc(100vh - 40px);
    max-width: 400px;
  }

  .header-title {
    font-size: 14px;
  }

  .session-title {
    max-width: 100px;
    font-size: 12px;
  }
}

/* Smaller screens */
@media (max-width: 480px) {
  .chat-window {
    bottom: 10px;
    right: 10px;
    width: calc(100vw - 20px);
    height: calc(100vh - 20px);
  }
}
</style>
