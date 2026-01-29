<script setup>
import { ref, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Store } from '@tauri-apps/plugin-store'
import FloatingBall from '@/components/FloatingBall.vue'

const appWindow = getCurrentWindow()
const currentTheme = ref('light')
let store = null

function handleBallClick() {
  // 点击悬浮球时打开主窗口
  // 这里可以实现打开主窗口的逻辑
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
  // 窗口挂载时的初始化逻辑
  console.log('Floating window mounted')

  // 加载主题
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