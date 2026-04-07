<script setup>
import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Settings, CheckSquare, RefreshCw } from 'lucide-vue-next'
import WindowControls from './components/ui/WindowControls.vue'
import { useSettingStore } from './store/settingStore'
import { useNoteStore } from './store/noteStore'
import { useToast } from './composables/useToast'
import { useSync } from './composables/useSync'

const router = useRouter()
const route = useRoute()
const settingStore = useSettingStore()
const noteStore = useNoteStore()
const { toastVisible, toastMessage, toastType } = useToast()
const { isSyncing, formattedLastSyncTime, loadSyncTime } = useSync()

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

    // 加载同步时间
    await loadSyncTime()
  }
})

onUnmounted(() => {
  // 清理工作
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
          <WindowControls />
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

      <!-- 底部同步状态栏 -->
      <div v-if="formattedLastSyncTime || isSyncing"
        class="h-5 border-t border-base-200 flex items-center justify-between px-3 text-[10px] text-base-content/40 flex-shrink-0">
        <div class="flex items-center gap-1.5">
          <span v-if="isSyncing" class="flex items-center gap-1">
            <RefreshCw :size="10" class="animate-spin" />
            同步中...
          </span>
          <span v-else>上次同步: {{ formattedLastSyncTime }}</span>
        </div>
      </div>
    </div>

    <!-- Toast 提示 -->
    <div v-if="toastVisible" class="toast toast-start toast-bottom z-[9999]">
      <div :class="['alert', toastType === 'success' ? 'alert-success' : toastType === 'error' ? 'alert-error' : 'alert-info']">
        <span>{{ toastMessage }}</span>
      </div>
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