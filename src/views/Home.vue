<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const droppedContent = ref({ type: '', content: '' })
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

  const data = e.dataTransfer

  // 1. 检查是否是图片
  if (data.types.includes('Files') && data.files.length > 0) {
    const file = data.files[0]
    if (file.type.startsWith('image/')) {
      const reader = new FileReader()
      reader.onload = (event) => {
        droppedContent.value = {
          type: 'image',
          content: event.target.result,
          fileName: file.name
        }
        console.log('拖拽的图片:', file.name)
      }
      reader.readAsDataURL(file)
      return
    }
  }

  // 2. 检查是否是 URL/链接
  if (data.types.includes('text/uri-list')) {
    const url = data.getData('text/uri-list')
    droppedContent.value = {
      type: 'url',
      content: url.trim()
    }
    console.log('拖拽的链接:', url.trim())
    return
  }

  // 3. 检查是否是纯文本
  if (data.types.includes('text/plain')) {
    const text = data.getData('text/plain')
    // 简单的 URL 验证
    if (text.startsWith('http://') || text.startsWith('https://')) {
      droppedContent.value = {
        type: 'url',
        content: text.trim()
      }
      console.log('拖拽的文本链接:', text.trim())
    } else {
      droppedContent.value = {
        type: 'text',
        content: text.trim()
      }
      console.log('拖拽的文本:', text.trim())
    }
    return
  }

  // 4. 检查是否是 HTML（包含链接）
  if (data.types.includes('text/html')) {
    const html = data.getData('text/html')
    // 提取 HTML 中的链接
    const urlMatch = html.match(/href="([^"]+)"/)
    if (urlMatch && urlMatch[1]) {
      droppedContent.value = {
        type: 'url',
        content: urlMatch[1]
      }
      console.log('从 HTML 提取的链接:', urlMatch[1])
    }
    return
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
      const paths = payload.paths || []
      paths.forEach(path => {
        console.log('拖拽的文件路径:', path)
        // 可以根据文件扩展名判断类型
        if (path.match(/\.(jpg|jpeg|png|gif|webp|bmp)$/i)) {
          droppedContent.value = {
            type: 'image',
            content: `file://${path}`,
            fileName: path.split(/[/\\]/).pop()
          }
        } else if (path.endsWith('.url') || path.endsWith('.webloc')) {
          droppedContent.value = {
            type: 'url',
            content: path,
            fileName: path.split(/[/\\]/).pop()
          }
        } else {
          droppedContent.value = {
            type: 'file',
            content: path,
            fileName: path.split(/[/\\]/).pop()
          }
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
    <div class="flex flex-col items-center gap-4 w-full max-w-2xl px-4">
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
          'w-full px-8 py-12 rounded-lg transition-colors border-2 border-dashed',
          isDragging 
            ? 'bg-blue-600/30 border-blue-500 text-blue-300' 
            : 'bg-gray-800 border-gray-600 text-gray-400'
        ]"
      >
        <div class="text-center">
          <div class="text-2xl mb-2">
            {{ isDragging ? '📥' : '📤' }}
          </div>
          <div>{{ isDragging ? '释放以添加内容' : '拖拽文本、图片或链接到这里' }}</div>
        </div>
      </div>
      
      <!-- 显示拖拽的内容 -->
      <div v-if="droppedContent.content" class="w-full">
        <!-- 图片类型 -->
        <div v-if="droppedContent.type === 'image'" class="px-6 py-4 bg-green-600/30 border border-green-500 rounded-lg text-green-300">
          <div class="text-sm mb-2">拖拽的图片:</div>
          <div v-if="droppedContent.fileName" class="text-xs mb-2 text-green-400">{{ droppedContent.fileName }}</div>
          <img :src="droppedContent.content" class="max-w-full max-h-64 rounded" alt="拖拽的图片" />
        </div>
        
        <!-- URL 类型 -->
        <div v-else-if="droppedContent.type === 'url'" class="px-6 py-4 bg-blue-600/30 border border-blue-500 rounded-lg text-blue-300">
          <div class="text-sm mb-2">拖拽的链接:</div>
          <a :href="droppedContent.content" target="_blank" class="font-mono break-all hover:underline">
            {{ droppedContent.content }}
          </a>
        </div>
        
        <!-- 文本类型 -->
        <div v-else-if="droppedContent.type === 'text'" class="px-6 py-4 bg-yellow-600/30 border border-yellow-500 rounded-lg text-yellow-300">
          <div class="text-sm mb-2">拖拽的文本:</div>
          <div class="font-mono break-all whitespace-pre-wrap">{{ droppedContent.content }}</div>
        </div>
        
        <!-- 文件类型 -->
        <div v-else-if="droppedContent.type === 'file'" class="px-6 py-4 bg-purple-600/30 border border-purple-500 rounded-lg text-purple-300">
          <div class="text-sm mb-2">拖拽的文件:</div>
          <div class="font-mono break-all">{{ droppedContent.content }}</div>
        </div>
      </div>
    </div>
  </main>
</template>