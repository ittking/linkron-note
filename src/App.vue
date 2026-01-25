<script setup>
import { onMounted } from 'vue'
import { useSettingStore } from './store/settingStore'

const settingStore = useSettingStore()

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
  <router-view />
</template>
