<template>
  <div ref="scrollContainer" class="h-full p-4 overflow-y-auto no-scrollbar">
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

      <!-- 工作目录设置 -->
      <div class="card bg-base-200 shadow-sm">
        <div class="card-body p-4">
          <h2 class="card-title text-sm font-medium">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-folder"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"></path></svg>
            工作目录
          </h2>
          <div class="space-y-3">
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">工作目录路径</span>
              </label>
              <input type="text" v-model="workDirectory" placeholder="留空使用默认路径" class="input input-bordered input-sm w-full" />
            </div>
            <div class="flex gap-2">
              <button class="btn btn-sm btn-primary flex-1" :class="{ 'loading': loading.workDir }" @click="saveWorkDirectory">
                保存工作目录
              </button>
              <button class="btn btn-sm btn-ghost" @click="selectWorkDirectory">
                选择目录
              </button>
            </div>
            <div v-if="workDirectoryStatus" :class="['text-xs', workDirectoryStatus.type === 'success' ? 'text-success' : 'text-error']">
              {{ workDirectoryStatus.message }}
            </div>
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
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onActivated, nextTick } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingStore } from '../store/settingStore'

const settingStore = useSettingStore()

// 滚动位置保存
const scrollContainer = ref(null)
let savedScrollTop = 0

// 开机启动
const autoStartEnabled = ref(false)
const loading = ref({
  autoStart: false,
  model: false,
  workDir: false
})

// 模型设置
const modelSettings = ref({
  apiKey: '',
  apiUrl: '',
  model: ''
})

// 工作目录
const workDirectory = ref('')
const workDirectoryStatus = ref(null)

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

// 路由离开前保存滚动位置
onBeforeRouteLeave((to, from, next) => {
  if (scrollContainer.value) {
    savedScrollTop = scrollContainer.value.scrollTop
    console.log('Setting onBeforeRouteLeave, scrollHeight:', scrollContainer.value.scrollHeight, 'clientHeight:', scrollContainer.value.clientHeight, 'saved scrollTop:', savedScrollTop)
  }
  next()
})

// 组件激活时恢复滚动位置
onActivated(async () => {
  console.log('Setting onActivated start, savedScrollTop:', savedScrollTop)
  await nextTick()
  
  // 使用 setTimeout 确保 DOM 完全更新
  setTimeout(() => {
    if (scrollContainer.value) {
      console.log('scrollContainer exists, scrollHeight:', scrollContainer.value.scrollHeight, 'clientHeight:', scrollContainer.value.clientHeight, 'current scrollTop:', scrollContainer.value.scrollTop)
      if (savedScrollTop > 0) {
        scrollContainer.value.scrollTop = savedScrollTop
        console.log('Restored scrollTop to:', savedScrollTop, 'actual scrollTop after set:', scrollContainer.value.scrollTop)
      }
    } else {
      console.log('scrollContainer is null')
    }
  }, 50)
})

// 初始化
onMounted(async () => {
  await loadAutoStartStatus()
  await loadModelSettings()
  await loadWorkDirectory()
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

// 加载工作目录
async function loadWorkDirectory() {
  try {
    workDirectory.value = await settingStore.get('workDirectory', '')
  } catch (error) {
    console.error('Failed to load work directory:', error)
  }
}

// 选择工作目录
async function selectWorkDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择工作目录'
    })
    
    if (selected) {
      workDirectory.value = selected
    }
  } catch (error) {
    console.error('Failed to select directory:', error)
  }
}

// 保存工作目录
async function saveWorkDirectory() {
  loading.value.workDir = true
  workDirectoryStatus.value = null
  
  try {
    if (workDirectory.value.trim()) {
      // 检查目录是否存在
      const exists = await invoke('check_directory_exists', { path: workDirectory.value.trim() })
      
      if (!exists) {
        // 创建目录
        await invoke('create_directory', { path: workDirectory.value.trim() })
        workDirectoryStatus.value = {
          type: 'success',
          message: '工作目录已创建并保存'
        }
      } else {
        workDirectoryStatus.value = {
          type: 'success',
          message: '工作目录已保存'
        }
      }
      
      await settingStore.set('workDirectory', workDirectory.value.trim())
    } else {
      // 清空工作目录
      await settingStore.set('workDirectory', '')
      workDirectoryStatus.value = {
        type: 'success',
        message: '已恢复默认工作目录'
      }
    }
  } catch (error) {
    console.error('Failed to save work directory:', error)
    workDirectoryStatus.value = {
      type: 'error',
      message: '保存失败: ' + error.message
    }
  } finally {
    loading.value.workDir = false
  }
}

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
  document.documentElement.setAttribute('data-theme', theme)
}
</script>