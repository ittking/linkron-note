<script setup>
import { onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Terminal, Settings, CheckSquare, Minimize2 } from 'lucide-vue-next'
import { useSettingStore } from './store/settingStore'
import { useNoteStore } from './store/noteStore'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

const settingStore = useSettingStore()
const noteStore = useNoteStore()
const router = useRouter()
const route = useRoute()
const appWindow = getCurrentWindow()

// 防抖函数
function debounce(func, wait) {
  let timeout
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout)
      func(...args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}

// 保存窗口大小
async function saveWindowSize() {
  try {
    const size = await appWindow.innerSize()
    // 转换为 LogicalSize 以处理 DPI 缩放
    const scaleFactor = await appWindow.scaleFactor()
    const logicalWidth = size.width / scaleFactor
    const logicalHeight = size.height / scaleFactor

    // 确保值有效
    if (logicalWidth > 0 && logicalHeight > 0) {
      await settingStore.set('windowWidth', logicalWidth)
      await settingStore.set('windowHeight', logicalHeight)
    }
  } catch (error) {
    console.error('Failed to save window size:', error)
  }
}

// 防抖的保存函数
const debouncedSaveWindowSize = debounce(saveWindowSize, 300)

// 恢复窗口大小
async function restoreWindowSize() {
  try {
    const width = await settingStore.get('windowWidth', 800)
    const height = await settingStore.get('windowHeight', 600)

    // 确保值有效
    if (width && height && width > 0 && height > 0) {
      await appWindow.setSize(new LogicalSize(width, height))
    }
  } catch (error) {
    console.error('Failed to restore window size:', error)
  }
}

// 监听窗口大小变化
let unlistenResize = null

async function setupWindowResizeListener() {
  try {
    unlistenResize = await appWindow.onResized(() => {
      debouncedSaveWindowSize()
    })
  } catch (error) {
    console.error('Failed to setup window resize listener:', error)
  }
}

const tabs = [
  { name: '笔记', path: '/note', icon: BookOpen },
  { name: '待办', path: '/todo', icon: CheckSquare },
  { name: '终端', path: '/term', icon: Terminal },
  { name: '设置', path: '/setting', icon: Settings }
]

function navigateTo(path) {
  router.push(path)
}

function isActive(path) {
  return route.path === path
}

onMounted(async () => {
  // 获取当前窗口标签
  const currentWindow = getCurrentWindow()
  const windowLabel = currentWindow.label

  // 只在主窗口中执行初始化操作
  if (windowLabel === 'main') {
    // 应用启动时加载并应用主题
    try {
      const theme = await settingStore.get('theme', 'light')
      // 由于使用了命名空间 .iterm-root，data-theme 需要设置在这个元素上
      const itermPanel = document.querySelector('.iterm-root')
      if (itermPanel) {
        itermPanel.setAttribute('data-theme', theme)
      }
    } catch (error) {
      console.error('Failed to load theme:', error)
    }
    // 初始化数据库
    try {
      await noteStore.initDatabase()
    } catch (error) {
      console.error('Failed to init database:', error)
    }

    // 恢复窗口大小
    await restoreWindowSize()

    // 设置窗口大小变化监听
    await setupWindowResizeListener()
  }
})

// 组件卸载时清理监听器
onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize()
  }
})
</script>

<template>
  <div class="iterm-root h-full">
    <main class="h-full bg-base-100 flex flex-col overflow-hidden border border-base-300">
      <!-- 顶部控制栏 -->
      <div data-tauri-drag-region
        class="select-none h-9 border-b border-base-300 flex items-center justify-between px-3 flex-shrink-0">
        <!-- 左侧：终端图标和名称 -->
        <div class="flex items-center gap-2">
          <Terminal :size="16" class="text-primary" data-tauri-drag-region />
          <span class="text-sm font-medium text-base-content" data-tauri-drag-region>ITERM</span>
        </div>

        <!-- 右侧：功能按钮 -->
        <div class="flex items-center gap-1.5 text-base-content/60">
          <button v-for="tab in tabs" :key="tab.path" @click="navigateTo(tab.path)" :class="[
            'btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded relative flex items-center justify-center',
            isActive(tab.path)
              ? 'text-primary bg-primary/10'
              : 'hover:text-base-content hover:bg-base-200'
          ]" :title="tab.name">
            <component :is="tab.icon" :size="14" />
            <!-- 选中状态底部指示条 -->
            <span v-if="isActive(tab.path)"
              class="absolute bottom-[-1px] left-1/2 -translate-x-1/2 w-5 h-0.5 bg-primary rounded-full"></span>
          </button>
          <button
            class="btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded hover:bg-base-200 text-base-content/60 hover:text-base-content flex items-center justify-center"
            title="收缩">
            <Minimize2 :size="14" />
          </button>
        </div>
      </div>

      <!-- 子页面内容区域 -->
      <div class="flex-1 overflow-hidden">
        <router-view v-slot="{ Component, route }">
          <transition name="page-fade" mode="out-in">
            <keep-alive v-if="route.meta?.keepAlive">
              <component :is="Component" :key="route.path" />
            </keep-alive>
            <component v-else :is="Component" :key="route.path" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>

</template>

<style scoped>
/* 页面过渡动画 */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>