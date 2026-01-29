<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const emit = defineEmits(['click'])

const ballRef = ref(null)
const isDragging = ref(false)
let startX, startY, initialX, initialY

function handleMouseDown(e) {
  isDragging.value = true
  startX = e.clientX
  startY = e.clientY

  // Get current position
  const rect = ballRef.value.getBoundingClientRect()
  initialX = rect.left
  initialY = rect.top

  // Add dragging class to disable animations
  ballRef.value.classList.add('dragging')
}

function handleMouseMove(e) {
  if (!isDragging.value) return

  const dx = e.clientX - startX
  const dy = e.clientY - startY

  // Switch to fixed positioning on first drag
  ballRef.value.style.position = 'fixed'
  ballRef.value.style.left = (initialX + dx) + 'px'
  ballRef.value.style.top = (initialY + dy) + 'px'
  ballRef.value.style.transform = 'none'
}

function handleMouseUp() {
  if (isDragging.value) {
    ballRef.value.classList.remove('dragging')
  }
  isDragging.value = false
}

function handleClick() {
  // Only emit click if not dragging
  if (!isDragging.value) {
    emit('click')
  }
}

onMounted(() => {
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <div
    ref="ballRef"
    class="floating-ball"
    @mousedown="handleMouseDown"
    @click="handleClick"
  >
    <span class="ball-icon">&gt;_</span>
    <span class="ball-tooltip">点击展开终端</span>
  </div>
</template>

<style scoped>
/* CSS Variables */
.floating-ball {
  --bg-primary: #0a0a0b;
  --bg-secondary: #141417;
  --bg-tertiary: #1c1c21;
  --border-subtle: #2a2a32;
  --text-primary: #e8e8ed;
  --text-secondary: #6b6b76;
  --accent-primary: #00ff88;
  --accent-glow: rgba(0, 255, 136, 0.15);
}

.floating-ball {
  width: 48px;
  height: 48px;
  background: linear-gradient(145deg, var(--bg-secondary), var(--bg-tertiary));
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  position: relative;
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  transition: box-shadow 0.3s cubic-bezier(0.4, 0, 0.2, 1),
              transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  animation: float 3s ease-in-out infinite;
}

.floating-ball:hover {
  transform: scale(1.08);
  box-shadow:
    0 8px 32px rgba(0, 255, 136, 0.2),
    0 0 0 1px var(--accent-primary),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.floating-ball:active {
  cursor: grabbing;
  transform: scale(1);
  transition: none;
}

.floating-ball.dragging {
  animation: none;
  transition: none;
  will-change: transform;
  cursor: grabbing;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}

/* Terminal icon inside ball */
.ball-icon {
  font-family: 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
  font-size: 16px;
  font-weight: 600;
  color: var(--accent-primary);
  text-shadow: 0 0 20px var(--accent-glow);
  transition: all 0.3s ease;
}

.floating-ball:hover .ball-icon {
  text-shadow: 0 0 30px var(--accent-glow), 0 0 60px var(--accent-glow);
}

/* Pulse ring effect */
.floating-ball::before {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 1px solid var(--accent-primary);
  opacity: 0;
  animation: pulse-ring 2s ease-out infinite;
}

@keyframes pulse-ring {
  0% {
    transform: scale(0.8);
    opacity: 0.5;
  }
  100% {
    transform: scale(1.5);
    opacity: 0;
  }
}

/* Tooltip */
.ball-tooltip {
  position: absolute;
  left: 60px;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  padding: 8px 14px;
  border-radius: 8px;
  font-family: 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transform: translateX(-10px);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.4),
    0 0 0 1px rgba(255, 255, 255, 0.05);
}

.ball-tooltip::before {
  content: '';
  position: absolute;
  left: -5px;
  top: 50%;
  transform: translateY(-50%) rotate(45deg);
  width: 10px;
  height: 10px;
  background: var(--bg-tertiary);
  border-left: 1px solid rgba(255, 255, 255, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.floating-ball:hover .ball-tooltip {
  opacity: 1;
  transform: translateX(0);
}
</style>