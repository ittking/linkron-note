<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Settings, CheckSquare, Minus, Maximize2, Minimize2 } from 'lucide-vue-next'
import { useSettingStore } from './store/settingStore'
import { useNoteStore } from './store/noteStore'
import { useWindowControl } from './composables/useWindowControl'
import { useToast } from './composables/useToast'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const route = useRoute()
const settingStore = useSettingStore()
const noteStore = useNoteStore()
const appWindow = getCurrentWindow()
const { isFullscreen, isMaximized, toggleFullscreen, maximizeWindow } = useWindowControl()
const { toastVisible, toastMessage, toastType } = useToast()

// 全局快捷键相关
let hotkeyUnlisten = null

const tabs = [
  { name: '笔记', path: '/note', icon: BookOpen },
  { name: '待办', path: '/todo', icon: CheckSquare },
  { name: '设置', path: '/setting', icon: Settings }
]

function navigateTo(path) {
  router.push(path)
}

function isActive(path) {
  return route.path === path
}

// 最小化窗口
async function minimizeWindow() {
  try {
    await appWindow.minimize()
  } catch (error) {
    console.error('Failed to minimize window:', error)
  }
}

// 最大化/恢复窗口
async function handleMaximizeWindow() {
  try {
    await maximizeWindow()
  } catch (error) {
    console.error('Failed to maximize/restore window:', error)
  }
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
      // 将 data-theme 设置在 html 元素上
      document.documentElement.setAttribute('data-theme', theme)
    } catch (error) {
      console.error('Failed to load theme:', error)
    }
    // 初始化数据库
    try {
      await noteStore.initDatabase()
    } catch (error) {
      console.error('Failed to init database:', error)
    }

    // 初始化配置
    try {
      // 确保笔记图片最大展示数有默认值
      const savedMaxCount = await settingStore.get('noteImageMaxCount', 4)
      if (savedMaxCount === null || savedMaxCount === undefined) {
        await settingStore.set('noteImageMaxCount', 4)
      }
    } catch (error) {
      console.error('Failed to init config:', error)
    }

    // 初始化全局快捷键
    await initGlobalHotkey()
  }
})

// 初始化全局快捷键
async function initGlobalHotkey() {
  try {
    // 获取保存的快捷键或默认值
    const savedHotkey = await settingStore.get('globalHotkey', '')
    let hotkey = savedHotkey

    if (!hotkey) {
      const os = await invoke('get_os')
      hotkey = os === 'macos' ? 'Option' : 'Alt'
    }

    // 注册快捷键
    await invoke('register_hotkey', { keyName: hotkey })

    // 监听快捷键事件
    hotkeyUnlisten = await listen('global-hotkey-triggered', () => {
      toggleWindowVisibility()
    })
  } catch (error) {
    console.error('Failed to init global hotkey:', error)
  }
}

// 切换窗口显示/隐藏
async function toggleWindowVisibility() {
  try {
    const isMinimized = await appWindow.isMinimized()

    if (isMinimized) {
      // 如果窗口已最小化，恢复窗口
      await appWindow.unminimize()
      await appWindow.setFocus()
    } else {
      // 如果窗口未最小化，最小化窗口
      await appWindow.minimize()
    }
  } catch (error) {
    console.error('Failed to toggle window visibility:', error)
  }
}

onUnmounted(() => {
  // 清理快捷键监听器
  if (hotkeyUnlisten) {
    hotkeyUnlisten()
  }
})

</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-transparent select-none">
    <div class="main-content h-full flex flex-col overflow-hidden border border-base-300 rounded bg-base-100">
      <!-- 顶部控制栏 -->
      <div data-tauri-drag-region
        class="select-none h-9 border-b border-base-300 flex items-center justify-between px-3 flex-shrink-0">
        <!-- 左侧：应用图标和名称 -->
        <div class="flex items-center gap-2">
          <BookOpen :size="16" class="text-primary" data-tauri-drag-region />
          <span class="text-sm font-medium text-base-content" data-tauri-drag-region>LINKRON</span>
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
            title="收缩"
            @click="minimizeWindow">
            <Minus :size="14" />
          </button>
          <button
            class="btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded hover:bg-base-200 text-base-content/60 hover:text-base-content flex items-center justify-center"
            :title="isMaximized ? '恢复' : '最大化'"
            @click="handleMaximizeWindow">
            <Maximize2 v-if="!isMaximized" :size="14" />
            <Minimize2 v-else :size="14" />
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
    </div>

    <!-- Toast 提示 -->
    <div :class="['fixed top-4 right-4 z-[9999] px-4 py-3 rounded-lg shadow-lg transition-all duration-300 pointer-events-none', toastVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0', toastType === 'success' ? 'bg-success text-success-content' : toastType === 'error' ? 'bg-error text-error-content' : 'bg-info text-info-content']">
      {{ toastMessage }}
    </div>
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