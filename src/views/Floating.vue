<script setup>
import { ref, onMounted } from 'vue'
import { Store } from '@tauri-apps/plugin-store'
import FloatingBall from '@/components/FloatingBall.vue'

const currentTheme = ref('light')
let store = null

function handleBallClick() {
  console.log('Floating ball clicked')
}

// 初始化 store
async function initStore() {
  if (!store) {
    store = await Store.load('settings.json')
  }
  return store
}

// 从 store 加载主题
async function loadTheme() {
  try {
    const storeInstance = await initStore()
    const theme = await storeInstance.get('theme')
    currentTheme.value = theme || 'light'
    // 应用主题到当前窗口
    const itermRoot = document.querySelector('.iterm-root')
    if (itermRoot) {
      itermRoot.setAttribute('data-theme', currentTheme.value)
    }
  } catch (error) {
    console.error('Failed to load theme:', error)
    currentTheme.value = 'light'
  }
}

onMounted(async () => {
  await loadTheme()

  // 定期检查主题变化（每秒检查一次）
  setInterval(async () => {
    try {
      const storeInstance = await initStore()
      const theme = await storeInstance.get('theme')
      if (theme && theme !== currentTheme.value) {
        currentTheme.value = theme
        const itermRoot = document.querySelector('.iterm-root')
        if (itermRoot) {
          itermRoot.setAttribute('data-theme', theme)
        }
      }
    } catch (error) {
      console.error('Failed to check theme:', error)
    }
  }, 1000)
})
</script>

<template>
  <div class="iterm-root border h-full w-full flex items-center justify-center !bg-transparent" :data-theme="currentTheme">
    <FloatingBall @click="handleBallClick" />
  </div>
</template>