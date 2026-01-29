<script setup>
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { useWindowThrough } from './composable/useWindowThrough'
import MainContent from './components/MainContent.vue'
import { useSettingStore } from './store/settingStore'
import { useNoteStore } from './store/noteStore'
import Capsule from './components/Capsule.vue'

const settingStore = useSettingStore()
const noteStore = useNoteStore()
const { register, unregister } = useWindowThrough()
const appWindow = getCurrentWindow()
const isMaximized = ref(true)

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

// 最小化窗口
async function minimizeWindow() {
  isMaximized.value = true
}

// 展开窗口
async function expandWindow() {
  isMaximized.value = false
}

// 监听窗口状态变化，控制窗口是否可调整大小
watch(isMaximized, async (value) => {
  await appWindow.setResizable(!value)
})

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

    // 注册穿透监听
    await register()
  }
})

// 组件卸载时清理监听器
onUnmounted(async () => {
  if (unlistenResize) {
    unlistenResize()
  }
  // 清理穿透监听
  await unregister()
})
</script>

<template>
  <div class="iterm-root h-full flex flex-col overflow-hidden bg-transparent">
    <!-- 胶囊组件 -->
    <Capsule v-show="isMaximized" @expand="expandWindow" />

    <!-- 主页内容 -->
    <MainContent v-show="!isMaximized" :on-minimize="minimizeWindow" class="w-full h-full bg-base-100" />
  </div>

</template>