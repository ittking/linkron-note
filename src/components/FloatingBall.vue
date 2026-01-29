<script setup>
function handleClick() {
  // 可以在这里添加展开/收起逻辑
  console.log('Floating ball clicked')
}
</script>

<template>
  <div data-tauri-drag-region class="floating-ball bg-base-100 select-none" @click="handleClick">
    <span class="ball-icon">&gt;_</span>
  </div>

</template>

<style scoped>
.floating-ball {
  width: 48px;
  height: 48px;
  border-radius: 9999px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
  transition: all 0.3s ease-in-out;
  border: 1px solid var(--color-base-100);
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  animation: float 3s ease-in-out infinite;
}

.floating-ball:hover {
  transform: scale(1.1);
  box-shadow:
    0 8px 32px rgba(var(--color-base-100), 0.2),
    0 0 0 1px hsl(var(--color-base-100)),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.floating-ball:active {
  transform: scale(1);
  transition: none;
}

@keyframes float {

  0%,
  100% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(-6px);
  }
}

.ball-icon {
  font-size: 16px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  color: var(--color-primary);
  text-shadow: 0 0 20px rgba(var(--color-primary), 0.15);
  transition: all 0.3s;
}

.floating-ball:hover .ball-icon {
  text-shadow:
    0 0 30px rgba(var(--color-primary), 0.15),
    0 0 60px rgba(var(--color-primary), 0.15);
}

.floating-ball::before {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 9999px;
  border: 1px solid var(--color-primary);
  opacity: 0;
  animation: pulse-ring 2s ease-out infinite;
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
</style>