<script setup>
import { BookOpen, Terminal, Settings, Minimize2 } from 'lucide-vue-next'
</script>

<template>
  <main class="min-h-screen bg-[#1c1c21]  flex flex-col rounded-xl overflow-hidden border border-[#2a2a32]">
    <!-- 顶部控制栏 -->
    <div data-tauri-drag-region
      class="h-9 border-b border-[#2a2a32] flex items-center justify-between px-3 flex-shrink-0">
      <!-- 左侧：终端图标和名称 -->
      <div class="flex items-center gap-2">
        <Terminal :size="16" class="text-[#00ff88]" />
        <span class="text-sm font-medium text-[#e8e8ed]">ITERM</span>
      </div>

      <!-- 右侧：功能按钮 -->
      <div class="flex items-center gap-1 text-[#808080]">
        <button class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32]   transition-all flex items-center justify-center"
          title="笔记">
          <BookOpen :size="13" />
        </button>
        <button class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32]   transition-all flex items-center justify-center"
          title="终端">
          <Terminal :size="13" />
        </button>
        <button class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32]   transition-all flex items-center justify-center"
          title="设置">
          <Settings :size="13" />
        </button>
        <button class="w-6.5 h-6.5 rounded hover:bg-[#2a2a32]   transition-all flex items-center justify-center"
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