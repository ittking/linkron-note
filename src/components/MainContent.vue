<script setup>
import { useRouter, useRoute } from 'vue-router'
import { BookOpen, Terminal, Settings, CheckSquare, Minimize2 } from 'lucide-vue-next'

const router = useRouter()
const route = useRoute()

defineProps({
  onMinimize: {
    type: Function,
    required: true
  }
})

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
</script>

<template>
  <div through-listener="true" class="main-content h-full flex flex-col overflow-hidden border border-base-300">
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
          title="收缩"
          @click="onMinimize">
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