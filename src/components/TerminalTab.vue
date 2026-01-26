<template>
  <div 
    ref="tabsContainerRef"
    class="flex bg-base-200 border-b border-base-300 overflow-x-auto overflow-y-hidden flex-shrink-0 cursor-grab select-none no-scrollbar"
    @mousedown="handleMouseDown" 
    @mousemove="handleMouseMove" 
    @mouseup="handleMouseUp" 
    @mouseleave="handleMouseUp"
  >
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="flex items-center px-4 py-2 cursor-pointer border-r border-base-300 whitespace-nowrap bg-base-200 transition-colors relative flex-shrink-0"
      :class="[
        tab.id === activeTabId 
          ? 'text-primary bg-primary/10' 
          : 'hover:bg-base-300'
      ]"
      @click="handleTabClick(tab.id)"
      @contextmenu.prevent="handleContextMenu($event, tab)"
    >
      <span class="text-sm pointer-events-none">{{ tab.title }}</span>
      <span
        class="ml-2 opacity-60 cursor-pointer text-base leading-none p-0.5 flex-shrink-0 hover:opacity-100 hover:bg-base-content/10 rounded"
        @click.stop="closeTab(tab.id)"
        v-if="tabs.length > 1"
      >×</span>
      <!-- 选中状态底部指示条 -->
      <span
        v-if="tab.id === activeTabId"
        class="absolute bottom-0 left-1/2 -translate-x-1/2 w-5 h-0.5 bg-primary rounded-full"
      ></span>
    </div>
    <div 
      class="tab-add px-4 py-2 cursor-pointer opacity-60 text-xl leading-none transition-opacity flex-shrink-0 hover:opacity-100 hover:text-primary"
      @click="addTab"
    >+</div>
  </div>
  
  <!-- 透明蒙版层 -->
  <div
    v-if="contextMenu.visible"
    class="fixed inset-0 z-[999] bg-transparent"
    @click="hideContextMenu"
  ></div>
  
  <!-- 右键菜单 -->
  <div
    v-if="contextMenu.visible"
    data-context-menu
    class="fixed z-[1000] bg-base-200 border border-base-300 rounded-lg shadow-xl min-w-[192px] py-2"
    :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    @click.stop
  >
    <div 
      class="flex items-center gap-2 px-4 py-2 hover:bg-base-300 cursor-pointer text-base-content transition-colors"
      @click="openInNewWindow"
    >
      <ExternalLink :size="16" />
      <span class="text-sm">新窗口打开</span>
    </div>
    <div 
      class="flex items-center gap-2 px-4 py-2 hover:bg-base-300 cursor-pointer text-base-content transition-colors"
      @click="renameTab"
    >
      <Edit2 :size="16" />
      <span class="text-sm">重命名</span>
    </div>
    <div class="h-px bg-base-300 my-1"></div>
    <div 
      class="flex items-center gap-2 px-4 py-2 hover:bg-error/10 cursor-pointer text-error transition-colors"
      @click="closeTab(contextMenu.tab.id)"
    >
      <X :size="16" />
      <span class="text-sm">关闭</span>
    </div>
  </div>

  <!-- 重命名弹窗 -->
  <dialog :open="renameDialog.visible" class="modal">
    <div class="modal-box bg-base-200 border border-base-300">
      <h3 class="font-bold text-lg text-base-content">重命名终端</h3>
      <div class="py-4">
        <input
          v-model="renameDialog.newTitle"
          type="text"
          placeholder="输入新的名称"
          class="input input-bordered w-full"
          @keyup.enter="confirmRename"
          ref="renameInputRef"
        />
      </div>
      <div class="modal-action">
        <button class="btn btn-ghost text-base-content/60 hover:text-base-content" @click="cancelRename">
          取消
        </button>
        <button class="btn btn-primary text-primary-content" @click="confirmRename">
          确定
        </button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop bg-black/50" @click="cancelRename">
      <button></button>
    </form>
  </dialog>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted, nextTick } from 'vue'
import { ExternalLink, Edit2, X } from 'lucide-vue-next'

const props = defineProps({
  tabs: { type: Array, required: true },
  activeTabId: { type: [String, null], default: null }
})

const emit = defineEmits(['select', 'add', 'close', 'rename', 'openInNewWindow'])

const tabsContainerRef = ref(null)
const renameInputRef = ref(null)
const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  tab: null
})

const renameDialog = reactive({
  visible: false,
  newTitle: ''
})

// 拖动滚动相关
let isDragging = false
let startX = 0
let scrollLeft = 0
let hasMoved = false // 用于区分点击和拖动

const handleMouseDown = (e) => {
  // 在整个 tab 列表区域都可以拖动
  isDragging = true
  hasMoved = false
  startX = e.pageX - (tabsContainerRef.value?.offsetLeft || 0)
  scrollLeft = tabsContainerRef.value?.scrollLeft || 0
  if (tabsContainerRef.value) {
    tabsContainerRef.value.style.cursor = 'grabbing'
  }
}

const handleMouseMove = (e) => {
  if (!isDragging || !tabsContainerRef.value) return
  e.preventDefault()
  const x = e.pageX - tabsContainerRef.value.offsetLeft
  const walk = (x - startX) * 1.5 // 滚动速度

  // 如果移动距离超过 5px，则认为是拖动
  if (Math.abs(walk) > 5) {
    hasMoved = true
  }

  tabsContainerRef.value.scrollLeft = scrollLeft - walk
}

const handleMouseUp = () => {
  isDragging = false
  hasMoved = false
  if (tabsContainerRef.value) {
    tabsContainerRef.value.style.cursor = 'grab'
  }
}

const handleTabClick = (id) => {
  // 如果发生了拖动，则不触发点击事件
  if (hasMoved) return
  emit('select', id)
}

const addTab = () => emit('add')
const closeTab = (id) => {
  emit('close', id)
  hideContextMenu()
}

const handleContextMenu = (e, tab) => {
  contextMenu.visible = true
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.tab = tab
}

const hideContextMenu = () => {
  contextMenu.visible = false
}

const openInNewWindow = () => {
  emit('openInNewWindow', contextMenu.tab.id)
  hideContextMenu()
}

const renameTab = () => {
  renameDialog.visible = true
  renameDialog.newTitle = contextMenu.tab.title
  hideContextMenu()

  // 聚焦输入框
  nextTick(() => {
    if (renameInputRef.value) {
      renameInputRef.value.focus()
      renameInputRef.value.select()
    }
  })
}

const confirmRename = () => {
  if (renameDialog.newTitle.trim()) {
    emit('rename', contextMenu.tab.id, renameDialog.newTitle.trim())
  }
  cancelRename()
}

const cancelRename = () => {
  renameDialog.visible = false
  renameDialog.newTitle = ''
}

onMounted(() => {
  // 不再需要全局点击监听，蒙版层会处理
})

onUnmounted(() => {
  // 不再需要清理全局点击监听
})
</script>

<style scoped>
/* 无需自定义样式，全部使用 Tailwind CSS 和 DaisyUI */
</style>