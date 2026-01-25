<script setup>
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Terminal, Settings, Minimize2 } from 'lucide-vue-next'

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
</script>

<template>
  <main class="min-h-screen bg-[#1c1c21] flex flex-col rounded-xl overflow-hidden border border-[#2a2a32]">
    <!-- 顶部控制栏 -->
    <div data-tauri-drag-region
      class="h-9 border-b border-[#2a2a32] flex items-center justify-between px-3 flex-shrink-0">
      <!-- 左侧：终端图标和名称 -->
      <div class="flex items-center gap-2">
        <Terminal :size="16" class="text-[#00ff88]" />
        <span class="text-sm font-medium text-[#e8e8ed]">ITERM</span>
      </div>

      <!-- 右侧：功能按钮 -->
      <div class="flex items-center gap-1 text-[#e8e8ed]">
        <button v-for="tab in tabs" :key="tab.path" @click="navigateTo(tab.path)" :class="[
          'w-6.5 h-6.5 rounded transition-all flex items-center justify-center',
          isActive(tab.path)
            ? 'bg-[#2a2a32]'
            : 'hover:bg-[#2a2a32]'
        ]" :title="tab.name">
          <component :is="tab.icon" :size="13" />
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