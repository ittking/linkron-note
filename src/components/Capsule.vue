<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { Maximize2 } from 'lucide-vue-next'

const emit = defineEmits(['expand'])

const currentDate = ref('')
const currentDay = ref('')
const currentTime = ref('')
const visible = ref(false)

let timer = null

function updateTime() {
  const now = new Date()
  
  // 格式化日期：27号
  const day = now.getDate()
  currentDate.value = `${day}号`
  
  // 格式化星期：星期日
  const days = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  currentDay.value = days[now.getDay()]
  
  // 格式化时间：HH:MM
  const hours = now.getHours().toString().padStart(2, '0')
  const minutes = now.getMinutes().toString().padStart(2, '0')
  currentTime.value = `${hours}:${minutes}`
}

function handleExpand() {
  emit('expand')
}

// 监听组件显示状态变化
const capsuleElement = ref(null)
let observer = null

function setupVisibilityObserver() {
  if (capsuleElement.value) {
    observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          // 组件变为可见，延迟触发动画
          setTimeout(() => {
            visible.value = true
          }, 50)
        } else {
          // 组件变为隐藏，重置动画状态
          visible.value = false
        }
      })
    }, { threshold: 0.1 })
    
    observer.observe(capsuleElement.value)
  }
}

function cleanupObserver() {
  if (observer) {
    observer.disconnect()
    observer = null
  }
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 1000)
  setupVisibilityObserver()
})

onUnmounted(() => {
  if (timer) {
    clearInterval(timer)
  }
  cleanupObserver()
})
</script>

<template>
  <div ref="capsuleElement" class="capsule-container">
    <div class="capsule" :class="{ 'visible': visible }">
      <div class="capsule-content">
        <span class="capsule-date">{{ currentDate }}</span>
        <span class="capsule-day">{{ currentDay }}</span>
        <span class="capsule-time">{{ currentTime }}</span>
      </div>
      <button class="expand-btn" @click="handleExpand" title="展开">
        <Maximize2 :size="14" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.capsule-container {
  position: absolute;
  top: 0;
  right: 0;
  z-index: 1000;
  overflow: hidden;
  pointer-events: none;
}

.capsule {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--daisyui-base-100);
  border: 1px solid var(--daisyui-base-300);
  border-radius: 9999px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(8px);
  opacity: 0;
  transform: translateX(100%);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.capsule.visible {
  opacity: 1;
  transform: translateX(0);
}

.capsule-content {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--daisyui-base-content);
}

.capsule-date {
  font-weight: 600;
  color: var(--daisyui-primary);
}

.capsule-day {
  color: var(--daisyui-base-content/70);
}

.capsule-time {
  font-family: monospace;
  font-weight: 500;
}

.expand-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  min-height: 0;
  padding: 0;
  border: none;
  background: var(--daisyui-primary);
  color: var(--daisyui-primary-content);
  border-radius: 9999px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.expand-btn:hover {
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(var(--daisyui-primary-rgb), 0.4);
}

.expand-btn:active {
  transform: scale(0.95);
}
</style>