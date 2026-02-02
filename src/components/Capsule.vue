<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Maximize2 } from 'lucide-vue-next'

const emit = defineEmits<{
  expand: []
}>()

const currentDay = ref('')
const currentTime = ref('')

let timer: ReturnType<typeof setInterval> | null = null

function updateTime() {
  const now = new Date()

  const days = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  currentDay.value = days[now.getDay()]

  const hours = now.getHours().toString().padStart(2, '0')
  const minutes = now.getMinutes().toString().padStart(2, '0')
  const seconds = now.getSeconds().toString().padStart(2, '0')
  currentTime.value = `${hours}:${minutes}:${seconds}`
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 1000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div data-tauri-drag-region class="absolute top-0 right-1 z-[1000]">
    <div data-tauri-drag-region through-listener="true"
      class="flex items-center gap-2 p-1 bg-base-100 border border-base-300 bg-base-100/80 rounded-full shadow-lg backdrop-blur select-none">
      <div data-tauri-drag-region class="flex items-center gap-2 text-xs text-base-content pl-2 cursor-pointer">
        <div data-tauri-drag-region class="flex flex-col items-center text-base-content/70 scale-80">
          <span data-tauri-drag-region>{{ currentDay }}</span>
          <span data-tauri-drag-region>{{ currentTime }}</span>
        </div>
      </div>

      <button class="bg-primary text-primary-content rounded-full flex items-center justify-center w-8 h-8 min-h-0 p-0"
        @click="emit('expand')" title="展开">
        <Maximize2 :size="12" />
      </button>
    </div>
  </div>
</template>
