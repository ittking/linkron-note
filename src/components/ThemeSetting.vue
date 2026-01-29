<script setup>
import { ref, onMounted } from 'vue'
import { useSettingStore } from '../store/settingStore'
import { Palette } from 'lucide-vue-next'

const settingStore = useSettingStore()

// 主题
const currentTheme = ref('light')
const themes = [
  'light',
  'dark',
  'cupcake',
  'bumblebee',
  'emerald',
  'corporate',
  'synthwave',
  'retro',
  'cyberpunk',
  'valentine',
  'halloween',
  'garden',
  'forest',
  'aquarium',
  'lofi',
  'pastel',
  'fantasy',
  'wireframe',
  'black',
  'luxury',
  'dracula',
  'cmyk',
  'autumn',
  'business',
  'acid',
  'lemonade',
  'night',
  'coffee',
  'winter',
  'dim',
  'nord',
  'sunset',
  'caramellatte',
  'abyss',
  'silk'
]

// 初始化
onMounted(async () => {
  await loadTheme()
})

// 加载主题
async function loadTheme() {
  try {
    currentTheme.value = await settingStore.get('theme', 'light')
  } catch (error) {
    console.error('Failed to load theme:', error)
  }
}

// 切换主题
async function changeTheme(theme) {
  currentTheme.value = theme
  applyTheme(theme)
  try {
    await settingStore.set('theme', theme)
  } catch (error) {
    console.error('Failed to save theme:', error)
  }
}

// 应用主题
function applyTheme(theme) {
  // 由于使用了命名空间 .iterm-root，data-theme 需要设置在这个元素上
  const itermPanel = document.querySelector('.iterm-root')
  if (itermPanel) {
    itermPanel.setAttribute('data-theme', theme)
  }
}
</script>

<template>
  <div class="card bg-base-200 shadow-sm">
    <div class="card-body p-4">
      <h2 class="card-title text-sm font-medium">
        <Palette :size="16" />
        主题设置
      </h2>
      <div class="space-y-3">
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">当前主题</span>
          </label>
          <select v-model="currentTheme" @change="changeTheme(currentTheme)" class="select select-bordered select-sm w-full">
            <option v-for="theme in themes" :key="theme" :value="theme">{{ theme }}</option>
          </select>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-2">
          <div
            v-for="theme in themes"
            :key="theme"
            @click="changeTheme(theme)"
            :class="[
              'cursor-pointer rounded-lg border-2 p-2 transition-all hover:scale-105',
              currentTheme === theme ? 'border-primary ring-2 ring-primary ring-opacity-50' : 'border-base-300 hover:border-primary'
            ]"
            :data-theme="theme"
          >
            <div class="flex flex-col items-center gap-2">
              <div class="flex gap-1">
                <div class="w-4 h-4 rounded bg-primary flex items-center justify-center text-[10px] font-bold text-base-content">A</div>
                <div class="w-4 h-4 rounded bg-secondary flex items-center justify-center text-[10px] font-bold text-base-content">A</div>
                <div class="w-4 h-4 rounded bg-accent flex items-center justify-center text-[10px] font-bold text-base-content">A</div>
                <div class="w-4 h-4 rounded bg-neutral flex items-center justify-center text-[10px] font-bold text-base-content">A</div>
                <div class="w-4 h-4 rounded bg-base-300 flex items-center justify-center text-[10px] font-bold text-base-content">A</div>
              </div>
              <span class="text-xs font-medium truncate w-full text-center">{{ theme }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>