<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { BookOpen, Terminal, Settings, Minimize2 } from 'lucide-vue-next'

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
  <main class="bg-black/90 min-h-screen flex flex-col" @dragenter="handleDragEnter" @dragleave="handleDragLeave"
    @dragover="handleDragOver" @drop="handleDrop">
    <!-- 顶部控制栏 -->
    <div data-tauri-drag-region
      class="h-9 bg-[#1c1c21] border-b border-[#2a2a32] flex items-center justify-between px-3 flex-shrink-0">
      <!-- 左侧：终端图标和名称 -->
      <div class="flex items-center gap-2">
        <Terminal :size="16" class="text-[#00ff88]" />
        <span class="text-sm font-medium text-[#e8e8ed]">ITERM</span>
      </div>

      <!-- 右侧：功能按钮 -->
      <div class="flex items-center gap-1">
        <button
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="笔记">
          <BookOpen :size="13" />
        </button>
        <button
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="终端">
          <Terminal :size="13" />
        </button>
        <button
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="设置">
          <Settings :size="13" />
        </button>
        <button
          class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32] text-[#6b6b76] hover:text-[#e8e8ed] transition-all flex items-center justify-center"
          title="收缩">
          <Minimize2 :size="13" />
        </button>
      </div>
    </div>

    <!-- 子页面内容区域 -->
    <div class="flex-1 overflow-hidden">
      <router-view />
    </div>
  </main>
</template>