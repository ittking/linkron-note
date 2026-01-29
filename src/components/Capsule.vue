<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { Maximize2 } from 'lucide-vue-next'

const emit = defineEmits<{
  expand: []
}>()

const currentDate = ref('')
const currentDay = ref('')
const currentTime = ref('')
const visible = ref(false)
const capsuleElement = ref<HTMLElement | null>(null)

let timer: ReturnType<typeof setInterval> | null = null
let observer: IntersectionObserver | null = null

function updateTime() {
  const now = new Date()

  currentDate.value = `${now.getDate()}号`

  const days = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  currentDay.value = days[now.getDay()]

  const hours = now.getHours().toString().padStart(2, '0')
  const minutes = now.getMinutes().toString().padStart(2, '0')
  const seconds = now.getSeconds().toString().padStart(2, '0')
  currentTime.value = `${hours}:${minutes}:${seconds}`
}

function setupVisibilityObserver() {
  if (capsuleElement.value) {
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setTimeout(() => {
              visible.value = true
            }, 50)
          } else {
            visible.value = false
          }
        })
      },
      { threshold: 0.1 }
    )

    observer.observe(capsuleElement.value)
  }
}

function cleanupObserver() {
  observer?.disconnect()
  observer = null
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 1000)
  setupVisibilityObserver()
  // 确保组件在挂载后显示
  nextTick(() => {
    visible.value = true
  })
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
  cleanupObserver()
})
</script>

<template>
  <div ref="capsuleElement" class="absolute top-0 right-0 z-[1000]">
    <div
      class="flex items-center gap-2 p-1 bg-base-100 border border-base-300 bg-base-100/50 rounded-full shadow-lg backdrop-blur select-none">
      <div class="flex items-center gap-2 text-xs text-base-content pl-2">
        <span data-tauri-drag-region class="font-semibold text-primary">{{ currentDate }}</span>
        <div data-tauri-drag-region class="flex flex-col items-center text-base-content/70 scale-80">
          <span data-tauri-drag-region>{{ currentDay }}</span>
          <span data-tauri-drag-region>{{ currentTime }}</span>
        </div>
      </div>

      <button class="btn btn-primary btn-circle btn-xs flex items-center justify-center w-8 h-8 min-h-0 p-0"
        @click="emit('expand')" title="展开">
        <Maximize2 :size="12" />
      </button>
    </div>

    <!-- 其他内容 -->
    <div></div>
  </div>
</template>
