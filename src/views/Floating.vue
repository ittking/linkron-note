<script setup>
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import FloatingBall from '@/components/FloatingBall.vue'

const appWindow = getCurrentWindow()

function handleBallClick() {
  // 点击悬浮球时打开主窗口
  // 这里可以实现打开主窗口的逻辑
  console.log('Floating ball clicked')
}

onMounted(() => {
  // 窗口挂载时的初始化逻辑
  console.log('Floating window mounted')
})
</script>

<template>
  <div class="floating-container">
    <FloatingBall @click="handleBallClick" />
  </div>
</template>

<style scoped>
.floating-container {
  width: 100vw;
  height: 100vh;
  position: relative;
  background: transparent;
}
</style>