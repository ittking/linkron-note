<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const emit = defineEmits(['click'])
const isHovered = ref(false)

function handleClick() {
  emit('click')
}

function handleMouseEnter() {
  isHovered.value = true
}

function handleMouseLeave() {
  isHovered.value = false
}

// 监听主题变化
let themeObserver = null

onMounted(() => {
  // 监听 .iterm-root 元素的 data-theme 属性变化
  const itermRoot = document.querySelector('.iterm-root')
  if (itermRoot) {
    themeObserver = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
          // 主题变化时触发重新渲染，让 CSS 变量更新
          // DaisyUI 的 CSS 变量会自动根据 data-theme 属性变化
        }
      })
    })

    themeObserver.observe(itermRoot, {
      attributes: true,
      attributeFilter: ['data-theme']
    })
  }
})

onBeforeUnmount(() => {
  if (themeObserver) {
    themeObserver.disconnect()
  }
})
</script>

<template>
  <div data-tauri-drag-region
    class="relative w-12 h-12 bg-base-100 rounded-full flex items-center justify-center cursor-pointer transition-all duration-300 animate-float hover:scale-110 hover:shadow-[0_8px_32px_hsl(var(--p)/0.2),0_0_0_1px_hsl(var(--p)),inset_0_1px_0_hsl(var(--bc)/0.1)] active:scale-100 active:transition-none shadow-[0_4px_20px_rgba(0,0,0,0.3),0_0_0_1px_hsl(var(--bc)/0.1),inset_0_1px_0_hsl(var(--bc)/0.05)]"
    @click="handleClick" @mouseenter="handleMouseEnter" @mouseleave="handleMouseLeave">
    <!-- Pulse ring effect -->
    <div class="absolute inset-0 rounded-full border border-primary opacity-0 animate-pulse-ring"></div>

    <!-- Terminal icon -->
    <span class="font-mono text-base font-bold text-primary transition-all duration-300">
      &gt;_
    </span>

    <!-- Tooltip -->
    <span
      class="absolute left-[60px] bg-base-300 text-base-content/70 px-3.5 py-2 rounded-lg text-xs font-medium whitespace-nowrap opacity-0 pointer-events-none -translate-x-2.5 transition-all duration-250 shadow-[0_4px_20px_rgba(0,0,0,0.3),0_0_0_1px_hsl(var(--bc)/0.1)]"
      :class="{ 'opacity-100 translate-x-0': isHovered }">
      <div
        class="absolute left-[-5px] top-1/2 -translate-y-1/2 rotate-45 w-2.5 h-2.5 bg-base-300 border-l border-t border-base-content/10">
      </div>
      点击展开终端
    </span>
  </div>
</template>

<style scoped>
@keyframes float {

  0%,
  100% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(-6px);
  }
}

@keyframes pulse-ring {
  0% {
    transform: scale(0.8);
    opacity: 0.5;
  }

  100% {
    transform: scale(1.5);
    opacity: 0;
  }
}

.animate-float {
  animation: float 3s ease-in-out infinite;
}

.animate-pulse-ring {
  animation: pulse-ring 2s ease-out infinite;
}

/* Hover state for icon glow */
.group:hover .icon-glow {
  text-shadow: 0 0 30px hsl(var(--p)/0.3), 0 0 60px hsl(var(--p)/0.3);
}
</style>