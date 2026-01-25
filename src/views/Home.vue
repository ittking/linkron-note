<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const mouseEvents = ref([])
const isListening = ref(false)
let unlisten = null

// 监听鼠标事件
async function startListening() {
  try {
    await invoke('start_mouse_listener')
    isListening.value = true
    
    // 监听来自 Rust 的事件
    unlisten = await listen('mouse-event', (event) => {
      const mouseEvent = event.payload
      mouseEvents.value.unshift({
        ...mouseEvent,
        timestamp: new Date().toLocaleTimeString()
      })
      
      // 只保留最近 100 条记录
      if (mouseEvents.value.length > 100) {
        mouseEvents.value = mouseEvents.value.slice(0, 100)
      }
    })
  } catch (error) {
    console.error('启动监听失败:', error)
  }
}

// 停止监听
async function stopListening() {
  try {
    await invoke('stop_mouse_listener')
    isListening.value = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  } catch (error) {
    console.error('停止监听失败:', error)
  }
}

// 获取事件类型的中文名称
function getEventTypeName(type) {
  const typeMap = {
    'Move': '移动',
    'ButtonPress': '按下',
    'ButtonRelease': '释放',
    'Wheel': '滚轮'
  }
  return typeMap[type] || type
}

// 获取按钮名称
function getButtonName(button) {
  if (!button) return '-'
  const buttonMap = {
    'Left': '左键',
    'Right': '右键',
    'Middle': '中键',
    'Unknown': '未知'
  }
  return buttonMap[button] || button
}

onMounted(() => {
  startListening()
})

onUnmounted(() => {
  stopListening()
})
</script>

<template>
  <main class="bg-black/90 min-h-screen flex flex-col items-center justify-center p-4">
    <div class="w-full max-w-4xl">
      <h1 class="text-3xl font-bold text-white mb-6 text-center">鼠标事件监听器</h1>
      
      <!-- 控制按钮 -->
      <div class="flex gap-4 mb-6 justify-center">
        <button 
          @click="startListening"
          :disabled="isListening"
          :class="[
            'px-6 py-2 rounded-lg font-medium transition-colors',
            isListening 
              ? 'bg-gray-600 text-gray-400 cursor-not-allowed' 
              : 'bg-blue-600 text-white hover:bg-blue-700'
          ]"
        >
          {{ isListening ? '监听中...' : '启动监听' }}
        </button>
        
        <button 
          @click="stopListening"
          :disabled="!isListening"
          :class="[
            'px-6 py-2 rounded-lg font-medium transition-colors',
            !isListening 
              ? 'bg-gray-600 text-gray-400 cursor-not-allowed' 
              : 'bg-red-600 text-white hover:bg-red-700'
          ]"
        >
          停止监听
        </button>
      </div>

      <!-- 事件列表 -->
      <div class="bg-gray-800 rounded-lg p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-white">事件记录</h2>
          <span class="text-gray-400">共 {{ mouseEvents.length }} 条记录</span>
        </div>
        
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left text-gray-400 border-b border-gray-700">
                <th class="pb-2 pr-4">时间</th>
                <th class="pb-2 pr-4">事件类型</th>
                <th class="pb-2 pr-4">X 坐标</th>
                <th class="pb-2 pr-4">Y 坐标</th>
                <th class="pb-2 pr-4">按钮</th>
                <th class="pb-2">滚轮</th>
              </tr>
            </thead>
            <tbody>
              <tr 
                v-for="(event, index) in mouseEvents" 
                :key="index"
                class="text-gray-300 border-b border-gray-700/50 hover:bg-gray-700/30"
              >
                <td class="py-2 pr-4">{{ event.timestamp }}</td>
                <td class="py-2 pr-4">{{ getEventTypeName(event.event_type) }}</td>
                <td class="py-2 pr-4">{{ event.x }}</td>
                <td class="py-2 pr-4">{{ event.y }}</td>
                <td class="py-2 pr-4">{{ getButtonName(event.button) }}</td>
                <td class="py-2">{{ event.wheel_delta ? event.wheel_delta : '-' }}</td>
              </tr>
            </tbody>
          </table>
          
          <div v-if="mouseEvents.length === 0" class="text-center text-gray-500 py-8">
            暂无事件记录
          </div>
        </div>
      </div>
    </div>
  </main>
</template>