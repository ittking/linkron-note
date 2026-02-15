<script setup>
import { ref, watch } from 'vue'
import { Plus, X } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import Select from './ui/Select.vue'

const props = defineProps({
  show: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'save'])

const formData = ref({
  provider: '',
  customName: '',
  apiKey: '',
  apiUrl: ''
})

const providers = [
  { value: 'deepseek', label: 'DeepSeek' },
  { value: 'siliconflow', label: '硅基流动' },
  { value: 'kimi', label: 'Kimi (Moonshot)' },
  { value: 'zhipu', label: '智谱AI (Zhipu AI)' },
  { value: 'custom', label: '自定义' }
]

const defaultApiUrls = {
  'deepseek': 'https://api.deepseek.com/v1',
  'siliconflow': 'https://api.siliconflow.cn/v1',
  'kimi': 'https://api.moonshot.cn/v1',
  'zhipu': 'https://open.bigmodel.cn/api/paas/v4'
}

function handleClose() {
  emit('close')
}

function handleSave() {
  if (!formData.value.provider) {
    alert('请选择供应商')
    return
  }
  if (!formData.value.apiKey) {
    alert('请输入 API Key')
    return
  }

  emit('save', {
    ...formData.value
  })
}

function resetForm() {
  formData.value = {
    provider: '',
    customName: '',
    apiKey: '',
    apiUrl: ''
  }
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    resetForm()
  }
})

watch(() => formData.value.provider, (newProvider) => {
  if (newProvider && newProvider !== 'custom') {
    formData.value.apiUrl = defaultApiUrls[newProvider] || ''
  }
})
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="card bg-base-100 shadow-xl w-full max-w-md mx-4">
      <div class="card-body p-5">
        <div class="flex items-center justify-between mb-4">
          <h3 class="card-title text-base font-medium">添加供应商</h3>
          <button @click="handleClose" class="p-1 rounded hover:bg-base-200 transition-colors">
            <X :size="18" />
          </button>
        </div>

        <div class="space-y-3">
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">供应商类型</span>
            </label>
            <Select 
              v-model="formData.provider"
              :options="providers"
              placeholder="选择供应商"
              size="sm"
            />
          </div>

          <div v-if="formData.provider === 'custom'" class="form-control">
            <label class="label">
              <span class="label-text text-xs">自定义名称</span>
            </label>
            <Input 
              v-model="formData.customName"
              placeholder="输入供应商名称"
              size="sm"
            />
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">API Key</span>
            </label>
            <Input 
              v-model="formData.apiKey"
              type="password"
              placeholder="输入 API Key"
              size="sm"
            />
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">API URL <span class="text-base-content/40">(可选)</span></span>
            </label>
            <Input 
              v-model="formData.apiUrl"
              placeholder="输入 API URL，留空使用默认"
              size="sm"
            />
          </div>

          <div class="flex gap-2 mt-4">
            <Button variant="ghost" size="sm" class="flex-1" @click="handleClose">
              取消
            </Button>
            <Button variant="primary" size="sm" class="flex-1" @click="handleSave">
              <Plus :size="14" />
              添加
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>