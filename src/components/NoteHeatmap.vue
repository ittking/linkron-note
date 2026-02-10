<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkDirectory } from '@/composables/useWorkDirectory'

const { getWorkDirectory } = useWorkDirectory('setting')

// 月份数据：12列，每列5周
const monthData = ref([]) // 格式: { year, month, weeks: [count1, count2, count3, count4, count5] }
const loading = ref(false)

// 悬浮提示状态
const tooltip = ref({
  visible: false,
  content: '',
  x: 0,
  y: 0
})

// 颜色级别（从浅到深）
const getColorClass = (count) => {
  if (count === 0) return 'bg-base-200'
  if (count <= 2) return 'bg-primary/20'
  if (count <= 5) return 'bg-primary/40'
  if (count <= 10) return 'bg-primary/60'
  if (count <= 20) return 'bg-primary/80'
  return 'bg-primary'
}

// 获取月份显示名称（中文）
const getMonthName = (year, month) => {
  const monthNames = ['一月', '二月', '三月', '四月', '五月', '六月', '七月', '八月', '九月', '十月', '十一月', '十二月']
  return monthNames[month - 1]
}

// 判断是否需要显示月份标签：从左到右，第一个显示，第二个不显示，以此类推
const shouldShowMonthLabel = (index) => {
  return index % 2 === 0
}

// 显示悬浮提示
const showTooltip = (event, year, month, weekIndex, count) => {
  if (count > 0) {
    const content = `${year}年/${month}月 第${weekIndex}周（${count}条笔记）`
    
    // 计算悬浮提示的位置
    const tooltipWidth = 200 // 估算的宽度
    const tooltipHeight = 50 // 估算的高度
    const padding = 10
    
    let x = event.clientX
    let y = event.clientY - padding
    
    // 检查是否超出右边界
    if (x + tooltipWidth / 2 > window.innerWidth) {
      x = window.innerWidth - tooltipWidth / 2 - padding
    }
    // 检查是否超出左边界
    if (x - tooltipWidth / 2 < 0) {
      x = tooltipWidth / 2 + padding
    }
    // 检查是否超出上边界
    if (y - tooltipHeight < 0) {
      y = event.clientY + tooltipHeight + padding
    }
    
    tooltip.value = {
      visible: true,
      content: content,
      x: x,
      y: y
    }
  }
}

// 隐藏悬浮提示
const hideTooltip = () => {
  tooltip.value.visible = false
}

// 加载热度图数据
async function loadHeatmapData() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const data = await invoke('get_notes_heatmap', { workDirectory })
    monthData.value = data
  } catch (error) {
    console.error('加载热度图数据失败:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadHeatmapData()
})
</script>

<template>
  <div class="w-full py-3">
    <div v-if="loading" class="flex justify-center items-center h-20">
      <span class="loading loading-spinner loading-sm text-primary"></span>
    </div>
    <div v-else-if="monthData.length > 0" class="flex flex-col gap-2">
      <!-- 每一行是一周，从第1周到第5周 -->
      <div v-for="weekIndex in 5" :key="weekIndex" class="flex items-center gap-1 overflow-x-auto">
        <div
          v-for="(month, index) in monthData"
          :key="`${month.year}-${month.month}-week-${weekIndex}`"
          :class="['w-5 h-5 rounded-sm flex-shrink-0 transition-colors cursor-pointer', getColorClass(month.weeks[weekIndex - 1])]"
          @mouseenter="showTooltip($event, month.year, month.month, weekIndex, month.weeks[weekIndex - 1])"
          @mouseleave="hideTooltip"
        ></div>
      </div>
      <!-- 月份标签（底部） -->
      <div class="flex items-center gap-1 overflow-x-auto">
        <span
          v-for="(month, index) in monthData"
          :key="`${month.year}-${month.month}-label`"
          class="text-xs text-base-content/60 flex-shrink-0 w-5 text-center whitespace-nowrap"
          :title="`${month.year}年${getMonthName(month.year, month.month)}`"
        >
          {{ shouldShowMonthLabel(index) ? getMonthName(month.year, month.month) : '' }}
        </span>
      </div>
    </div>
    <div v-else class="flex items-center justify-center h-20 text-base-content/40 text-xs">
      暂无数据
    </div>

    <!-- 悬浮提示 -->
    <transition name="fade">
      <div
        v-if="tooltip.visible"
        class="fixed z-50 px-3 py-2 bg-black/90 text-white text-xs rounded-lg shadow-xl pointer-events-none whitespace-nowrap"
        :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px', transform: 'translate(-50%, -100%)' }"
      >
        {{ tooltip.content }}
      </div>
    </transition>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>