<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkDirectory } from '@/composables/useWorkDirectory'

// 使用 useWorkDirectory composable
const { getWorkDirectory } = useWorkDirectory('setting')

const noteCount = ref(0)
const tagCount = ref(0)
const todoCount = ref(99) // TODO 统计，暂时固定为 99

// 加载统计数据
async function loadStatistics() {
  try {
    const workDirectory = await getWorkDirectory()
    
    // 获取笔记数量
    const count = await invoke('count_notes', { workDirectory })
    noteCount.value = count

    // 获取标签数量
    const allTags = await invoke('get_all_tags', { workDirectory })
    tagCount.value = allTags.length

    // TODO 统计暂时固定为 99，等后台实现后再查询
    // todoCount.value = 99
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

onMounted(() => {
  loadStatistics()
})

defineExpose({
  refresh: loadStatistics
})
</script>

<template>
  <div class="p-4">
    <div class="flex gap-3">
      <!-- 笔记统计 -->
      <div class="flex-1 text-center">
        <div class="text-xl font-semibold text-primary">{{ noteCount }}</div>
        <div class="text-xs text-base-content/50 mt-1">NOTE</div>
      </div>

      <!-- TODO 统计 -->
      <div class="flex-1 text-center">
        <div class="text-xl font-semibold text-primary">{{ todoCount }}</div>
        <div class="text-xs text-base-content/50 mt-1">TODO</div>
      </div>

      <!-- 标签统计 -->
      <div class="flex-1 text-center">
        <div class="text-xl font-semibold text-primary">{{ tagCount }}</div>
        <div class="text-xs text-base-content/50 mt-1">TAG</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>