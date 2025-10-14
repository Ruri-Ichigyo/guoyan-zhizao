<template>
  <div
    ref="bubbleRef"
    class="chat-bubble"
    :style="bubbleStyle"
    @mousedown="handleMouseDown"
    @touchstart="handleTouchStart"
  >
    <div class="bubble-icon">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="icon"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M8.625 12a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375M21 12c0 4.556-4.03 8.25-9 8.25a9.764 9.764 0 01-2.555-.337A5.972 5.972 0 015.41 20.97a5.969 5.969 0 01-.474-.065 4.48 4.48 0 00.978-2.025c.09-.457-.133-.901-.467-1.226C3.93 16.178 3 14.189 3 12c0-4.556 4.03-8.25 9-8.25s9 3.694 9 8.25z"
        />
      </svg>
    </div>

    <!-- Unread badge -->
    <div v-if="unreadCount > 0" class="unread-badge">
      {{ unreadCount > 99 ? '99+' : unreadCount }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useChatStore } from '../stores/chatStore'

const chatStore = useChatStore()
const bubbleRef = ref<HTMLElement | null>(null)

// Position state
const position = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const clickThreshold = 5 // pixels - to distinguish click from drag

const unreadCount = computed(() => chatStore.unreadCount)

// Computed style for bubble position
const bubbleStyle = computed(() => ({
  right: position.value.x > 0 ? 'auto' : '20px',
  left: position.value.x > 0 ? `${position.value.x}px` : 'auto',
  bottom: position.value.y > 0 ? 'auto' : '20px',
  top: position.value.y > 0 ? `${position.value.y}px` : 'auto',
  cursor: isDragging.value ? 'grabbing' : 'grab'
}))

// Mouse drag handlers
const handleMouseDown = (e: MouseEvent) => {
  isDragging.value = false
  dragStart.value = {
    x: e.clientX - (position.value.x || (window.innerWidth - 20 - 60)),
    y: e.clientY - (position.value.y || (window.innerHeight - 20 - 60))
  }
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
  e.preventDefault()
}

const handleMouseMove = (e: MouseEvent) => {
  const deltaX = Math.abs(e.clientX - dragStart.value.x - (position.value.x || 0))
  const deltaY = Math.abs(e.clientY - dragStart.value.y - (position.value.y || 0))

  if (deltaX > clickThreshold || deltaY > clickThreshold) {
    isDragging.value = true
  }

  if (isDragging.value) {
    const newX = e.clientX - dragStart.value.x
    const newY = e.clientY - dragStart.value.y

    // Constrain to viewport
    const maxX = window.innerWidth - 60
    const maxY = window.innerHeight - 60

    position.value = {
      x: Math.max(0, Math.min(newX, maxX)),
      y: Math.max(0, Math.min(newY, maxY))
    }

    // Save position to localStorage
    savePosition()
  }
}

const handleMouseUp = () => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)

  // If it was a click (not a drag), open the chat
  if (!isDragging.value) {
    chatStore.openChat()
  }

  setTimeout(() => {
    isDragging.value = false
  }, 100)
}

// Touch drag handlers
const handleTouchStart = (e: TouchEvent) => {
  if (e.touches.length !== 1) return

  isDragging.value = false
  const touch = e.touches[0]
  dragStart.value = {
    x: touch.clientX - (position.value.x || (window.innerWidth - 20 - 60)),
    y: touch.clientY - (position.value.y || (window.innerHeight - 20 - 60))
  }
  document.addEventListener('touchmove', handleTouchMove)
  document.addEventListener('touchend', handleTouchEnd)
  e.preventDefault()
}

const handleTouchMove = (e: TouchEvent) => {
  if (e.touches.length !== 1) return

  const touch = e.touches[0]
  const deltaX = Math.abs(touch.clientX - dragStart.value.x - (position.value.x || 0))
  const deltaY = Math.abs(touch.clientY - dragStart.value.y - (position.value.y || 0))

  if (deltaX > clickThreshold || deltaY > clickThreshold) {
    isDragging.value = true
  }

  if (isDragging.value) {
    const newX = touch.clientX - dragStart.value.x
    const newY = touch.clientY - dragStart.value.y

    // Constrain to viewport
    const maxX = window.innerWidth - 60
    const maxY = window.innerHeight - 60

    position.value = {
      x: Math.max(0, Math.min(newX, maxX)),
      y: Math.max(0, Math.min(newY, maxY))
    }

    // Save position to localStorage
    savePosition()
  }
}

const handleTouchEnd = () => {
  document.removeEventListener('touchmove', handleTouchMove)
  document.removeEventListener('touchend', handleTouchEnd)

  // If it was a tap (not a drag), open the chat
  if (!isDragging.value) {
    chatStore.openChat()
  }

  setTimeout(() => {
    isDragging.value = false
  }, 100)
}

// Save and restore position
const savePosition = () => {
  localStorage.setItem('chatBubblePosition', JSON.stringify(position.value))
}

const restorePosition = () => {
  const saved = localStorage.getItem('chatBubblePosition')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      // Validate the position is within current viewport
      const maxX = window.innerWidth - 60
      const maxY = window.innerHeight - 60

      position.value = {
        x: Math.max(0, Math.min(parsed.x, maxX)),
        y: Math.max(0, Math.min(parsed.y, maxY))
      }
    } catch (e) {
      // Invalid saved position, use default
    }
  }
}

// Lifecycle
onMounted(() => {
  restorePosition()
})

onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
  document.removeEventListener('touchmove', handleTouchMove)
  document.removeEventListener('touchend', handleTouchEnd)
})
</script>

<style scoped>
.chat-bubble {
  position: fixed;
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: linear-gradient(135deg, #67c23a 0%, #529b2e 100%);
  box-shadow: 0 4px 12px rgba(103, 194, 58, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  z-index: 1000;
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

.chat-bubble:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 16px rgba(103, 194, 58, 0.5);
}

.chat-bubble:active {
  transform: scale(0.95);
}

.bubble-icon {
  width: 32px;
  height: 32px;
  color: white;
  pointer-events: none;
}

.icon {
  width: 100%;
  height: 100%;
}

.unread-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 20px;
  height: 20px;
  border-radius: 10px;
  background-color: #f56c6c;
  color: white;
  font-size: 12px;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 6px;
  box-shadow: 0 2px 4px rgba(245, 108, 108, 0.3);
  pointer-events: none;
}

@media (max-width: 768px) {
  .chat-bubble {
    width: 50px;
    height: 50px;
  }

  .bubble-icon {
    width: 28px;
    height: 28px;
  }
}
</style>
