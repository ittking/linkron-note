<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Maximize, Square, X } from 'lucide-vue-next'
import { useWindowControl } from '@/composables/useWindowControl'

const appWindow = getCurrentWindow()
const { isMaximized, maximizeWindow } = useWindowControl()

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

// 关闭窗口
async function closeWindow() {
  try {
    await appWindow.close()
  } catch (error) {
    console.error('Failed to close window:', error)
  }
}
</script>

<template>
  <div class="flex items-center gap-1.5 text-base-content/60">
    <!-- 最小化 -->
    <button
      class="btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded hover:bg-base-200 text-base-content/60 hover:text-base-content flex items-center justify-center"
      title="收缩"
      @click="minimizeWindow"
    >
      <Minus :size="14" />
    </button>

    <!-- 最大化/恢复 -->
    <button
      class="btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded hover:bg-base-200 text-base-content/60 hover:text-base-content flex items-center justify-center"
      :title="isMaximized ? '恢复' : '最大化'"
      @click="handleMaximizeWindow"
    >
      <Maximize v-if="!isMaximized" :size="14" />
      <Square v-else :size="14" />
    </button>

    <!-- 关闭 -->
    <button
      class="btn btn-ghost btn-sm w-6 h-6 min-h-0 p-0 rounded hover:bg-error/20 hover:text-error text-base-content/60 flex items-center justify-center"
      title="关闭"
      @click="closeWindow"
    >
      <X :size="16" />
    </button>
  </div>
</template>
