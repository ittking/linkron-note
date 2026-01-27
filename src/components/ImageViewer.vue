<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { X, ZoomIn, ZoomOut, RotateCw } from 'lucide-vue-next'

const props = defineProps({
  src: {
    type: String,
    required: true
  },
  alt: {
    type: String,
    default: ''
  },
  // 默认显示样式
  aspectRatio: {
    type: String,
    default: 'square' // 'square' | 'portrait' | 'landscape'
  },
  className: {
    type: String,
    default: ''
  }
})

const emit = defineEmits([])

// 预览相关状态
const previewVisible = ref(false)
const imageRef = ref(null)
const scale = ref(1)
const rotation = ref(0)
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const startPosition = ref({ x: 0, y: 0 })

const MIN_SCALE = 0.5
const MAX_SCALE = 5

// 打开预览
function openPreview() {
  previewVisible.value = true
}

// 关闭预览
function closePreview() {
  previewVisible.value = false
  reset()
}

// 重置预览状态
function reset() {
  scale.value = 1
  rotation.value = 0
  startPosition.value = { x: 0, y: 0 }
}

// 缩放
function handleZoom(delta) {
  const newScale = scale.value + delta
  scale.value = Math.max(MIN_SCALE, Math.min(MAX_SCALE, newScale))
}

// 滚轮缩放
function handleWheel(e) {
  if (e.ctrlKey) {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    handleZoom(delta)
  }
}

// 旋转
function handleRotate() {
  rotation.value = (rotation.value + 90) % 360
}

// 阻止原生拖拽
function handleDragStart(e) {
  e.preventDefault()
  e.stopPropagation()
  return false
}

// 预览中的鼠标按下
function handleMouseDown(e) {
  // 只响应左键
  if (e.button !== 0) return

  // 阻止默认拖拽行为
  e.preventDefault()

  isDragging.value = true
  dragStart.value = {
    x: e.clientX,
    y: e.clientY
  }
  startPosition.value = {
    x: startPosition.value.x,
    y: startPosition.value.y
  }
}

// 预览中的鼠标移动
function handleMouseMove(e) {
  if (!isDragging.value) return

  const deltaX = e.clientX - dragStart.value.x
  const deltaY = e.clientY - dragStart.value.y

  startPosition.value = {
    x: startPosition.value.x + deltaX,
    y: startPosition.value.y + deltaY
  }

  dragStart.value = {
    x: e.clientX,
    y: e.clientY
  }
}

// 预览中的鼠标释放
function handleMouseUp(e) {
  if (e.button === 0) {
    isDragging.value = false
  }
}

// 键盘事件
function handleKeyDown(e) {
  if (e.key === 'Escape') {
    closePreview()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeyDown)
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <!-- 默认显示的图片 -->
  <div :class="[
    'relative rounded-md overflow-hidden border border-base-200 bg-base-200 cursor-pointer hover:border-primary/50 transition-colors',
    aspectRatio === 'square' ? 'aspect-square' :
      aspectRatio === 'portrait' ? 'aspect-[3/4]' : 'aspect-video',
    className
  ]" @click="openPreview">
    <img :src="src" :alt="alt" class="w-full h-full object-cover" loading="lazy" />
  </div>

  <!-- 预览弹窗 -->
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="previewVisible" class="fixed inset-0 z-[9999] bg-black/90 flex items-center justify-center select-none"
        @wheel="handleWheel" @mousedown.self="closePreview">

        <!-- 关闭按钮 -->
        <button @click="closePreview"
          class="absolute top-4 right-4 w-10 h-10 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors z-10">
          <X :size="24" />
        </button>

        <!-- 工具栏 -->
        <div
          class="absolute bottom-8 left-1/2 -translate-x-1/2 flex items-center gap-3 bg-black/60 backdrop-blur-sm rounded-full px-6 py-3 z-10">
          <button @click="handleZoom(-0.2)" :disabled="scale <= MIN_SCALE"
            class="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
            <ZoomOut :size="18" />
          </button>
          <span class="text-white text-sm font-medium min-w-[60px] text-center">{{ Math.round(scale * 100) }}%</span>
          <button @click="handleZoom(0.2)" :disabled="scale >= MAX_SCALE"
            class="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
            <ZoomIn :size="18" />
          </button>
          <div class="w-px h-6 bg-white/30"></div>
          <button @click="handleRotate"
            class="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors">
            <RotateCw :size="18" />
          </button>
        </div>

        <!-- 预览图片容器 -->
        <div class="relative overflow-hidden cursor-grab active:cursor-grabbing"
          :style="{ width: '80vw', height: '80vh' }">
          <img ref="imageRef" :src="src" :alt="alt"
            class="absolute top-1/2 left-1/2 transition-transform duration-75 ease-out select-none" :style="{
              transform: `translate(calc(-50% + ${startPosition.x}px), calc(-50% + ${startPosition.y}px)) scale(${scale}) rotate(${rotation}deg)`,
              maxWidth: '100%',
              maxHeight: '100%',
              objectFit: 'contain',
              pointerEvents: 'auto',
              userSelect: 'none',
              WebkitUserDrag: 'none',
              KhtmlUserDrag: 'none',
              MozUserDrag: 'none',
              OUserDrag: 'none',
              userDrag: 'none'
            }" @mousedown="handleMouseDown" @dragstart="handleDragStart" />
        </div>

        <!-- 提示 -->
        <div class="absolute top-4 left-4 text-white/60 text-xs">
          Ctrl + 滚轮缩放 • 拖拽移动
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>