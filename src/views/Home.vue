<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { X, Minus, Maximize2, BookOpen, Terminal, Settings, Minimize2 } from 'lucide-vue-next'

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
    class="bg-black/90 min-h-screen flex flex-col"
    @dragenter="handleDragEnter"
    @dragleave="handleDragLeave"
    @dragover="handleDragOver"
    @drop="handleDrop"
  >
    <!-- 顶部控制栏 -->
    <div 
      data-tauri-drag-region
      class="h-9 bg-[#1c1c21] border-b border-[#2a2a32] flex items-center justify-between px-3 flex-shrink-0"
    >
      <!-- 左侧：Mac 窗口控制按钮 -->
      <div class="flex items-center gap-2">
        <button class="w-3.5 h-3.5 rounded-full bg-[#ff5f57] hover:brightness-110 transition-all flex items-center justify-center">
          <X :size="8" class="text-[#ff5f57]" />
        </button>
        <button class="w-3.5 h-3.5 rounded-full bg-[#ffbd2e] hover:brightness-110 transition-all flex items-center justify-center">
          <Minus :size="8" class="text-[#ffbd2e]" />
        </button>
        <button class="w-3.5 h-3.5 rounded-full bg-[#28c840] hover:brightness-110 transition-all flex items-center justify-center">
          <Maximize2 :size="8" class="text-[#28c840]" />
        </button>
      </div>

      <!-- 右侧：功能按钮 -->
      <div class="flex items-center gap-1">
        <button 
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="笔记"
        >
          <BookOpen :size="13" />
        </button>
        <button 
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="终端"
        >
          <Terminal :size="13" />
        </button>
        <button 
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="设置"
        >
          <Settings :size="13" />
        </button>
        <button 
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="收缩"
        >
          <Minimize2 :size="13" />
        </button>
      </div>
    </div>

    <!-- 子页面内容区域 -->
    <div class="flex-1 flex items-center justify-center p-4 overflow-hidden">
      <div class="w-full max-w-2xl">
        <!-- 拖放区域指示 -->
        <div 
          :class="[
            'w-full px-8 py-12 rounded-lg transition-colors border-2 border-dashed',
            isDragging 
              ? 'bg-blue-600/30 border-blue-500 text-blue-300' 
              : 'bg-[#141417] border-[#2a2a32] text-[#6b6b76]'
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
        <div v-if="droppedContent.content" class="w-full mt-4">
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
    </div>
  </main>
</template>