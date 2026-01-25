<script setup>
import { onMounted, onUnmounted } from 'vue'
import { useWindowThrough } from '@/composable/useWindowThrough'

const { register, unregister } = useWindowThrough()

onMounted(() => {
  register()
})

onUnmounted(() => {
  unregister()
})
</script>

<template>
  <main class="bg-black/90 min-h-screen flex items-center justify-center">
    <div 
      through-listener="test-button"
      class="px-8 py-4 bg-blue-600 text-white rounded-lg cursor-pointer hover:bg-blue-700 transition-colors"
    >
      点击测试（此区域不穿透）
    </div>
  </main>
</template>