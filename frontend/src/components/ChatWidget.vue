<template>
  <div class="chat-widget">
    <!-- Chat Window (expanded state) -->
    <Transition name="chat-window">
      <ChatWindow v-if="chatStore.isOpen" />
    </Transition>

    <!-- Chat Bubble (minimized state) -->
    <Transition name="chat-bubble">
      <ChatBubble v-if="!chatStore.isOpen" />
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useChatStore } from '../stores/chatStore'
import ChatWindow from './ChatWindow.vue'
import ChatBubble from './ChatBubble.vue'

const chatStore = useChatStore()

// Initialize chat store on mount
onMounted(() => {
  // You can add initialization logic here, such as:
  // - Load user's last active session
  // - Check for unread messages
  // - Set up global keyboard shortcuts
})
</script>

<style scoped>
.chat-widget {
  position: fixed;
  bottom: 0;
  right: 0;
  z-index: 9999;
}

/* Chat Window Transitions */
.chat-window-enter-active {
  animation: slideInUp 0.3s ease-out;
}

.chat-window-leave-active {
  animation: slideOutDown 0.3s ease-in;
}

@keyframes slideInUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes slideOutDown {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
}

/* Chat Bubble Transitions */
.chat-bubble-enter-active {
  animation: bubbleIn 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.chat-bubble-leave-active {
  animation: bubbleOut 0.3s ease-in;
}

@keyframes bubbleIn {
  from {
    opacity: 0;
    transform: scale(0);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes bubbleOut {
  from {
    opacity: 1;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(0);
  }
}
</style>
