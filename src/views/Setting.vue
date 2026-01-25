<template>
  <div class="h-full p-4 overflow-y-auto">
    <div class="space-y-4">
      <!-- 开机启动 -->
      <div class="card bg-base-200 shadow-sm">
        <div class="card-body p-4">
          <h2 class="card-title text-sm font-medium">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-power"><path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path><line x1="12" x2="12" y1="2" y2="12"></line></svg>
            开机启动
          </h2>
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">开机自动启动</span>
              <input type="checkbox" class="toggle toggle-sm" :checked="autoStartEnabled" @change="toggleAutoStart" :disabled="loading.autoStart" />
            </label>
          </div>
        </div>
      </div>

      <!-- 模型设置 -->
      <div class="card bg-base-200 shadow-sm">
        <div class="card-body p-4">
          <h2 class="card-title text-sm font-medium">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-bot"><path d="M12 8V4H8"></path><rect width="16" height="12" x="4" y="8" rx="2"></rect><path d="M2 14h2"></path><path d="M20 14h2"></path><path d="M15 13v2"></path><path d="M9 13v2"></path></svg>
            模型设置
          </h2>
          <div class="space-y-3">
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">API Key</span>
              </label>
              <input type="password" v-model="modelSettings.apiKey" placeholder="请输入 API Key" class="input input-bordered input-sm w-full" />
            </div>
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">API URL</span>
              </label>
              <input type="text" v-model="modelSettings.apiUrl" placeholder="请输入 API URL" class="input input-bordered input-sm w-full" />
            </div>
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">模型名称</span>
              </label>
              <input type="text" v-model="modelSettings.model" placeholder="请输入模型名称" class="input input-bordered input-sm w-full" />
            </div>
            <button class="btn btn-primary btn-sm w-full" :class="{ 'loading': loading.model }" @click="saveModelSettings">
              保存模型设置
            </button>
          </div>
        </div>
      </div>

      <!-- 主题设置 -->
      <div class="card bg-base-200 shadow-sm">
        <div class="card-body p-4">
          <h2 class="card-title text-sm font-medium">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-palette"><circle cx="13.5" cy="6.5" r=".5"></circle><circle cx="17.5" cy="10.5" r=".5"></circle><circle cx="8.5" cy="7.5" r=".5"></circle><circle cx="6.5" cy="12.5" r=".5"></circle><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"></path></svg>
            主题设置
          </h2>
          <div class="space-y-3">
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">当前主题</span>
              </label>
              <select v-model="currentTheme" @change="changeTheme" class="select select-bordered select-sm w-full">
                <option v-for="theme in themes" :key="theme" :value="theme">{{ theme }}</option>
              </select>
            </div>
            <div class="grid grid-cols-5 gap-2">
              <div
                v-for="theme in themes"
                :key="theme"
                @click="changeTheme(theme)"
                :class="[
                  'cursor-pointer rounded border-2 p-2 text-center text-xs transition-all',
                  currentTheme === theme ? 'border-primary scale-105' : 'border-base-300 hover:border-primary'
                ]"
                :data-theme="theme"
              >
                <div class="flex flex-col items-center gap-1">
                  <div class="flex gap-1">
                    <div class="w-3 h-3 rounded bg-primary"></div>
                    <div class="w-3 h-3 rounded bg-secondary"></div>
                    <div class="w-3 h-3 rounded bg-accent"></div>
                  </div>
                  <span class="truncate w-full">{{ theme }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'

const settingStore = useSettingStore()

// 开机启动
const autoStartEnabled = ref(false)
const loading = ref({
  autoStart: false,
  model: false
})

// 模型设置
const modelSettings = ref({
  apiKey: '',
  apiUrl: '',
  model: ''
})

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
  'procyon'
]

// 初始化
onMounted(async () => {
  await loadAutoStartStatus()
  await loadModelSettings()
  await loadTheme()
})

// 加载开机启动状态
async function loadAutoStartStatus() {
  try {
    autoStartEnabled.value = await invoke('is_autostart_enabled')
  } catch (error) {
    console.error('Failed to load autostart status:', error)
  }
}

// 切换开机启动
async function toggleAutoStart() {
  loading.value.autoStart = true
  try {
    const newState = !autoStartEnabled.value
    await invoke('set_autostart', { enable: newState })
    autoStartEnabled.value = newState
  } catch (error) {
    console.error('Failed to toggle autostart:', error)
    // 恢复状态
    await loadAutoStartStatus()
  } finally {
    loading.value.autoStart = false
  }
}

// 加载模型设置
async function loadModelSettings() {
  try {
    modelSettings.value.apiKey = await settingStore.get('model.apiKey', '')
    modelSettings.value.apiUrl = await settingStore.get('model.apiUrl', '')
    modelSettings.value.model = await settingStore.get('model.model', '')
  } catch (error) {
    console.error('Failed to load model settings:', error)
  }
}

// 保存模型设置
async function saveModelSettings() {
  loading.value.model = true
  try {
    await settingStore.set('model.apiKey', modelSettings.value.apiKey)
    await settingStore.set('model.apiUrl', modelSettings.value.apiUrl)
    await settingStore.set('model.model', modelSettings.value.model)
  } catch (error) {
    console.error('Failed to save model settings:', error)
  } finally {
    loading.value.model = false
  }
}

// 加载主题
async function loadTheme() {
  try {
    currentTheme.value = await settingStore.get('theme', 'light')
    applyTheme(currentTheme.value)
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
  document.documentElement.setAttribute('data-theme', theme)
}
</script>