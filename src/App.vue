<script setup>
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Terminal, Settings, Minimize2 } from 'lucide-vue-next'
import { useSettingStore } from './store/settingStore'

const settingStore = useSettingStore()
const router = useRouter()
const route = useRoute()

const tabs = [
  { name: '笔记', path: '/note', icon: BookOpen },
  { name: '终端', path: '/term', icon: Terminal },
  { name: '设置', path: '/setting', icon: Settings }
]

function navigateTo(path) {
  router.push(path)
}

function isActive(path) {
  return route.path === path
}

function getCurrentTitle() {
  return route.meta?.title || ''
}

onMounted(async () => {
  // 应用启动时加载并应用主题
  try {
    const theme = await settingStore.get('theme', 'light')
    document.documentElement.setAttribute('data-theme', theme)
  } catch (error) {
    console.error('Failed to load theme:', error)
  }
})
</script>

<template>
  <main class="h-full bg-base-100 flex flex-col overflow-hidden border border-base-300">
    <!-- 顶部控制栏 -->
    <div class="select-none h-9 border-b border-base-300 flex items-center justify-between px-3 flex-shrink-0"
      data-tauri-drag-region>
      <!-- 左侧：终端图标和名称 -->
      <div class="flex items-center gap-2">
        <Terminal :size="16" class="text-primary" />
        <span class="text-sm font-medium text-base-content">ITERM</span>
      </div>

      <!-- 中间：当前页面标题 -->
      <div class="flex-1 text-base-content/60 text-center text-xs" data-tauri-drag-region>{{ getCurrentTitle() }}</div>

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
      <div class="h-full max-w-200 mx-auto">
        <router-view v-slot="{ Component, route }">
          <keep-alive :include="['Note', 'Terminal', 'Setting']">
            <component :is="Component" :key="route.name" />
          </keep-alive>
        </router-view>
      </div>
    </div>
  </main>
</template>
