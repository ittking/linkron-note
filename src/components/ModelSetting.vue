<script setup>
import { ref, onMounted } from 'vue'
import { useSettingStore } from '../store/settingStore'
import { Bot } from 'lucide-vue-next'

const settingStore = useSettingStore()

// 模型设置
const modelSettings = ref({
  apiKey: '',
  apiUrl: '',
  model: ''
})

const isLoading = ref(false)

// 初始化
onMounted(async () => {
  await loadModelSettings()
})

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
  isLoading.value = true
  try {
    await settingStore.set('model.apiKey', modelSettings.value.apiKey)
    await settingStore.set('model.apiUrl', modelSettings.value.apiUrl)
    await settingStore.set('model.model', modelSettings.value.model)
  } catch (error) {
    console.error('Failed to save model settings:', error)
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="card bg-base-200 shadow-sm">
    <div class="card-body p-4">
      <h2 class="card-title text-sm font-medium">
        <Bot :size="16" />
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
        <button class="btn btn-primary btn-sm w-full" :class="{ 'loading': isLoading }" @click="saveModelSettings">
          保存模型设置
        </button>
      </div>
    </div>
  </div>
</template>