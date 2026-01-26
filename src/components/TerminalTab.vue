<template>
  <div 
    ref="tabsContainerRef"
    class="terminal-tabs" 
    @mousedown="handleMouseDown" 
    @mousemove="handleMouseMove" 
    @mouseup="handleMouseUp" 
    @mouseleave="handleMouseUp"
  >
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === activeTabId }"
      @click="handleTabClick(tab.id)"
      @contextmenu.prevent="handleContextMenu($event, tab)"
    >
      <span class="tab-title">{{ tab.title }}</span>
      <span
        class="tab-close"
        @click.stop="closeTab(tab.id)"
        v-if="tabs.length > 1"
      >×</span>
    </div>
    <div class="tab-add" @click="addTab">+</div>
  </div>
  
  <!-- 右键菜单 -->
  <div
    v-if="contextMenu.visible"
    class="context-menu"
    :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    @click.stop
  >
    <div class="context-menu-item" @click="openInNewWindow">
      <span>🔗</span>
      <span>新窗口打开</span>
    </div>
    <div class="context-menu-item" @click="renameTab">
      <span>✏️</span>
      <span>重命名</span>
    </div>
    <div class="context-menu-divider"></div>
    <div class="context-menu-item danger" @click="closeTab(contextMenu.tab.id)">
      <span>✕</span>
      <span>关闭</span>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  tabs: { type: Array, required: true },
  activeTabId: { type: [String, null], default: null }
})

const emit = defineEmits(['select', 'add', 'close', 'rename', 'openInNewWindow'])

const tabsContainerRef = ref(null)
const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  tab: null
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
  emit('rename', contextMenu.tab.id)
  hideContextMenu()
}

// 点击其他地方关闭菜单
const handleClickOutside = (e) => {
  if (!e.target.closest('.context-menu') && !e.target.closest('.tab')) {
    hideContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.terminal-tabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #3e3e3e;
  overflow-x: auto;
  flex-shrink: 0;
  cursor: grab;
  user-select: none;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/Edge */
}

.terminal-tabs::-webkit-scrollbar {
  display: none; /* Chrome/Safari/Opera */
}

.tab {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  border-right: 1px solid #3e3e3e;
  white-space: nowrap;
  background: #2d2d2d;
  transition: background 0.2s;
  position: relative;
  flex-shrink: 0;
}

.tab:hover {
  background: #383838;
}

.tab.active {
  background: #1e1e1e;
}

.tab-title {
  font-size: 13px;
  color: #d4d4d4;
  pointer-events: none;
}

.tab-close {
  margin-left: 8px;
  opacity: 0.6;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0 4px;
  flex-shrink: 0;
}

.tab-close:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.tab-add {
  padding: 8px 16px;
  cursor: pointer;
  opacity: 0.6;
  font-size: 18px;
  line-height: 1;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.tab-add:hover {
  opacity: 1;
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background: #2d2d2d;
  border: 1px solid #3e3e3e;
  border-radius: 4px;
  padding: 4px 0;
  min-width: 160px;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
  color: #d4d4d4;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: #383838;
}

.context-menu-item.danger {
  color: #f48771;
}

.context-menu-item.danger:hover {
  background: rgba(244, 135, 113, 0.1);
}

.context-menu-divider {
  height: 1px;
  background: #3e3e3e;
  margin: 4px 0;
}
</style>