<script setup>
import { ref, onMounted, onBeforeUnmount, watch, computed } from 'vue'
import { X, ZoomIn, ZoomOut, RotateCw, FolderOpen, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { useSettingStore } from '@/store/settingStore'
import { revealFile, convertResourceUrl } from '@/utils/fileUpload'

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
  },
  // 图片列表（用于切换）
  images: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits([])

const settingStore = useSettingStore()

// 预览相关状态
const previewVisible = ref(false)
const imageRef = ref(null)
const scale = ref(1)
const rotation = ref(0)
const isDragging = ref(false)
const hasMoved = ref(false) // 标记是否发生了移动（用于区分点击和拖拽）
const dragStart = ref({ x: 0, y: 0 })
const startPosition = ref({ x: 0, y: 0 })
const currentImageIndex = ref(0)

// 转换后的图片地址（平台适配）
const convertedSrc = ref('')
const convertedImages = ref([])

// 监听 props.src 变化，转换 URL
watch(() => props.src, async (newSrc) => {
  if (newSrc) {
    convertedSrc.value = await convertResourceUrl(newSrc)
  } else {
    convertedSrc.value = ''
  }
}, { immediate: true })

// 监听 props.images 变化，转换所有 URL
watch(() => props.images, async (newImages) => {
  if (newImages && newImages.length > 0) {
    convertedImages.value = await Promise.all(newImages.map(img => convertResourceUrl(img)))
  } else {
    convertedImages.value = []
  }
}, { immediate: true, deep: true })

// 计算当前显示的图片（使用转换后的 URL）
const currentImage = computed(() => {
  if (convertedImages.value && convertedImages.value.length > 0) {
    return convertedImages.value[currentImageIndex.value] || convertedImages.value[0]
  }
  return convertedSrc.value
})

// 初始化时根据 src 查找索引
function initCurrentIndex() {
  if (convertedImages.value && convertedImages.value.length > 0) {
    const index = convertedImages.value.findIndex(img => img === convertedSrc.value)
    currentImageIndex.value = index >= 0 ? index : 0
  }
}

// 是否有多个图片
const hasMultipleImages = computed(() => {
  return convertedImages.value && convertedImages.value.length > 1
})

// 当前图片编号
const imageNumber = computed(() => {
  if (hasMultipleImages.value) {
    return `${currentImageIndex.value + 1} / ${convertedImages.value.length}`
  }
  return ''
})

// 判断是否为本地图片（不是 https 或 base64）
const isLocalImage = computed(() => {
  const imageSrc = currentImage.value
  if (!imageSrc) return false
  // 如果是 https 协议，不是本地图片
  if (imageSrc.startsWith('https://')) return false
  // 如果是 base64 数据，不是本地图片
  if (imageSrc.startsWith('data:')) return false
  // 如果是 http 协议，不是本地图片
  if (imageSrc.startsWith('http://')) return false
  // 其他情况认为是本地图片
  return true
})

const MIN_SCALE = 0.5
const MAX_SCALE = 5

// 打开预览
function openPreview() {
  initCurrentIndex()
  previewVisible.value = true
  reset()
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

// 切换到上一张图片
function handlePrevious() {
  if (!hasMultipleImages.value) return
  currentImageIndex.value = (currentImageIndex.value - 1 + props.images.length) % props.images.length
  reset()
}

// 切换到下一张图片
function handleNext() {
  if (!hasMultipleImages.value) return
  currentImageIndex.value = (currentImageIndex.value + 1) % props.images.length
  reset()
}

// 处理图片点击（左半边上一张，右半边下一张）
function handleImageClick(e) {
  // 如果发生了移动（拖拽），不处理点击
  if (hasMoved.value) return
  
  // 如果按下了 Ctrl 键，不处理点击
  if (e.ctrlKey || e.metaKey) return
  
  // 如果没有多个图片，不处理
  if (!hasMultipleImages.value) return
  
  // 获取图片容器
  const container = imageRef.value
  if (!container) return
  
  // 计算点击位置在图片容器的水平位置比例
  const rect = container.getBoundingClientRect()
  const clickX = e.clientX - rect.left
  const relativeX = clickX / rect.width
  
  // 点击左半边：上一张
  // 点击右半边：下一张
  if (relativeX < 0.5) {
    handlePrevious()
  } else {
    handleNext()
  }
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

// 在文件夹中显示图片
async function revealImageFile() {
  try {
    const workDirectory = await settingStore.get('workDirectory', '')
    await revealFile(props.src, workDirectory)
  } catch (error) {
    console.error('显示图片失败:', error)
  }
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
  hasMoved.value = false
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

  // 如果移动超过阈值，标记为发生了移动
  if (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5) {
    hasMoved.value = true
  }

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
  } else if (e.key === 'ArrowLeft' && !e.ctrlKey && hasMultipleImages.value) {
    handlePrevious()
  } else if (e.key === 'ArrowRight' && !e.ctrlKey && hasMultipleImages.value) {
    handleNext()
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
    'relative overflow-hidden border border-base-200 bg-base-200 cursor-pointer hover:border-primary/50 transition-colors',
    aspectRatio === 'square' ? 'aspect-square' :
      aspectRatio === 'portrait' ? 'aspect-[3/4]' : 'aspect-video',
    className
  ]" @click="openPreview">
    <img :src="convertedSrc" :alt="alt" class="w-full h-full object-cover" loading="lazy" />
  </div>

  <!-- 预览弹窗 -->
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="previewVisible" class="fixed inset-0 z-[9999] bg-black/90 flex items-center justify-center select-none"
        @wheel="handleWheel" @mousedown.self="closePreview">

        <!-- 关闭按钮 -->
        <button @click="closePreview"
          class="absolute top-4 right-4 w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors z-10">
          <X :size="18" />
        </button>

        <!-- 工具栏 -->
        <div
          class="absolute bottom-8 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-black/60 backdrop-blur-sm rounded-full px-4 py-2 z-10">
          <!-- 上一张图片按钮 -->
          <button v-if="hasMultipleImages" @click="handlePrevious"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
            title="上一张">
            <ChevronLeft :size="14" />
          </button>
          
          <button @click="handleZoom(-0.2)" :disabled="scale <= MIN_SCALE"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
            <ZoomOut :size="14" />
          </button>
          <span class="text-white text-xs font-medium min-w-[50px] text-center">{{ Math.round(scale * 100) }}%</span>
          <button @click="handleZoom(0.2)" :disabled="scale >= MAX_SCALE"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
            <ZoomIn :size="14" />
          </button>
          <button @click="handleRotate"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors">
            <RotateCw :size="14" />
          </button>
          <button v-if="isLocalImage" @click="revealImageFile"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
            title="在文件夹中显示">
            <FolderOpen :size="14" />
          </button>
          
          <!-- 下一张图片按钮 -->
          <button v-if="hasMultipleImages" @click="handleNext"
            class="w-6 h-6 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
            title="下一张">
            <ChevronRight :size="14" />
          </button>
        </div>

        <!-- 预览图片容器 -->
        <div class="relative overflow-hidden cursor-grab active:cursor-grabbing"
          :style="{ width: '80vw', height: '80vh' }">
          <img ref="imageRef" :src="currentImage" :alt="alt"
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
            }" @mousedown="handleMouseDown" @dragstart="handleDragStart" @click="handleImageClick" />
        </div>

        <!-- 提示 -->
        <div class="absolute top-4 left-4 text-white/60 text-xs flex flex-col gap-1">
          <span>Ctrl + 滚轮缩放 • 拖拽移动{{ imageNumber ? ` (${imageNumber})` : '' }}</span>
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