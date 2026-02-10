<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkDirectory } from '@/composables/useWorkDirectory'

const { getWorkDirectory } = useWorkDirectory('setting')

// 月份数据：12列，每列5周
const monthData = ref([]) // 格式: { year, month, weeks: [count1, count2, count3, count4, count5] }
const loading = ref(false)

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
          :class="['w-5 h-5 rounded-sm flex-shrink-0 transition-colors', getColorClass(month.weeks[weekIndex - 1])]"
          :title="`${getMonthName(month.year, month.month)} 第${weekIndex}周: ${month.weeks[weekIndex - 1]} 条笔记`"
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
  </div>
</template>