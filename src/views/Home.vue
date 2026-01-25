<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const droppedUrl = ref('')
const isDragging = ref(false)
let unlistenFileDrop = null

// 处理拖拽进入
function handleDragEnter(e) {
  e.preventDefault()
  isDragging.value = true
}

// 处理拖拽离开
function handleDragLeave(e) {
  e.preventDefault()
  isDragging.value = false
}

// 处理拖拽悬停
function handleDragOver(e) {
  e.preventDefault()
  e.dataTransfer.dropEffect = 'copy'
}

// 处理拖拽释放
function handleDrop(e) {
  e.preventDefault()
  isDragging.value = false

  // 获取拖拽的数据
  const data = e.dataTransfer

  // 检查是否是 URL/链接
  if (data.types.includes('text/uri-list')) {
    const url = data.getData('text/uri-list')
    droppedUrl.value = url.trim()
    console.log('拖拽的链接:', droppedUrl.value)
  } 
  // 检查是否是纯文本（可能是 URL）
  else if (data.types.includes('text/plain')) {
    const text = data.getData('text/plain')
    // 简单的 URL 验证
    if (text.startsWith('http://') || text.startsWith('https://')) {
      droppedUrl.value = text.trim()
      console.log('拖拽的文本链接:', droppedUrl.value)
    }
  }
  // 检查是否是 HTML（包含链接）
  else if (data.types.includes('text/html')) {
    const html = data.getData('text/html')
    // 提取 HTML 中的链接
    const urlMatch = html.match(/href="([^"]+)"/)
    if (urlMatch && urlMatch[1]) {
      droppedUrl.value = urlMatch[1]
      console.log('从 HTML 提取的链接:', droppedUrl.value)
    }
  }
}

// 监听 Tauri 文件拖拽事件
async function setupTauriFileDrop() {
  const window = getCurrentWindow()
  
  unlistenFileDrop = await window.onFileDropEvent((event) => {
    const { payload } = event
    
    if (payload.type === 'hover') {
      isDragging.value = true
    } else if (payload.type === 'drop') {
      isDragging.value = false
      // 检查拖拽的是否是 URL 文件（.url 或 .webloc）
      const paths = payload.paths || []
      paths.forEach(path => {
        if (path.endsWith('.url') || path.endsWith('.webloc')) {
          console.log('检测到 URL 快捷方式:', path)
          // 这里可以读取 .url 文件内容提取实际 URL
        }
      })
    } else if (payload.type === 'cancelled') {
      isDragging.value = false
    }
  })
}

onMounted(() => {
  setupTauriFileDrop()
})

onUnmounted(() => {
  if (unlistenFileDrop) {
    unlistenFileDrop()
  }
})
</script>

<template>
  <main 
    class="bg-black/90 min-h-screen flex items-center justify-center"
    @dragenter="handleDragEnter"
    @dragleave="handleDragLeave"
    @dragover="handleDragOver"
    @drop="handleDrop"
  >
    <div class="flex flex-col items-center gap-4">
      <!-- 可拖拽区域 -->
      <div 
        data-tauri-drag-region 
        class="px-8 py-4 bg-gray-700 text-white rounded-lg cursor-move hover:bg-gray-600 transition-colors"
      >
        可以拖拽
      </div>
      
      <!-- 拖放区域指示 -->
      <div 
        :class="[
          'px-8 py-4 rounded-lg transition-colors border-2 border-dashed',
          isDragging 
            ? 'bg-blue-600/30 border-blue-500 text-blue-300' 
            : 'bg-gray-800 border-gray-600 text-gray-400'
        ]"
      >
        {{ isDragging ? '释放以添加链接' : '拖拽链接到这里' }}
      </div>
      
      <!-- 显示拖拽的链接 -->
      <div v-if="droppedUrl" class="px-8 py-4 bg-green-600/30 border border-green-500 rounded-lg text-green-300 max-w-md break-all">
        <div class="text-sm mb-1">拖拽的链接:</div>
        <div class="font-mono">{{ droppedUrl }}</div>
      </div>
    </div>
  </main>
</template>