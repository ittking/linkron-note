<script setup>
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingStore } from '@/store/settingStore'

const settingStore = useSettingStore()

onMounted(async () => {
  // 获取当前窗口标签
  const currentWindow = getCurrentWindow()
  const windowLabel = currentWindow.label

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
  <div class="h-full">
    <router-view />
  </div>
</template>

<style>
/* 全局样式 */
html, body {
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

#app {
  height: 100%;
}
</style>
