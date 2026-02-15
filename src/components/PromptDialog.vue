<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import InputWithAI from './ui/InputWithAI.vue'
import { useSettingStore } from '../store/settingStore'

const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  prompt: {
    type: Object,
    default: null
  },
  isSystem: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['save', 'close'])

const settingStore = useSettingStore()

const form = ref({
  id: null,
  name: '',
  type: 'url',
  urlPattern: '',
  template: ''
})

const errors = ref({})
const isGeneratingRegex = ref(false)

watch(() => props.show, (newVal) => {
  if (newVal) {
    if (props.prompt) {
      form.value = {
        id: props.prompt.id,
        name: props.prompt.name,
        type: props.prompt.type,
        urlPattern: props.prompt.urlPattern || '',
        template: props.prompt.template
      }
    } else {
      form.value = {
        id: null,
        name: '',
        type: 'url',
        urlPattern: '',
        template: ''
      }
    }
    errors.value = {}
  }
})

async function handleGenerateRegex() {
  if (!form.value.urlPattern.trim()) {
    return
  }

  isGeneratingRegex.value = true
  
  try {
    // 获取当前激活的模型配置
    const providers = await settingStore.get('model.providers', [])
    const activeProviderId = await settingStore.get('model.activeProviderId', null)
    
    if (!activeProviderId || providers.length === 0) {
      alert('请先配置模型供应商')
      return
    }

    const activeProvider = providers.find(p => p.id === activeProviderId)
    if (!activeProvider || !activeProvider.currentModel) {
      alert('请先选择模型')
      return
    }

    // 调用 AI 生成正则表达式
    const prompt = `请为以下网址生成一个正则表达式，用于匹配该网址及其所有子路径。只返回正则表达式本身，不要任何解释或额外文字。\n网址：${form.value.urlPattern}`
    
    const result = await invoke('generate_regex', {
      prompt: prompt,
      provider: activeProvider.provider,
      apiKey: activeProvider.apiKey,
      apiUrl: activeProvider.apiUrl,
      model: activeProvider.currentModel
    })

    if (result && result.trim()) {
      // 验证生成的正则表达式
      try {
        new RegExp(result.trim())
        form.value.urlPattern = result.trim()
        errors.value.urlPattern = null
      } catch (e) {
        console.error('生成的正则表达式无效:', e)
        alert('生成的正则表达式格式错误，请手动调整')
      }
    }
  } catch (error) {
    console.error('生成正则表达式失败:', error)
    alert('生成失败：' + error)
  } finally {
    isGeneratingRegex.value = false
  }
}

function validateForm() {
  errors.value = {}

  if (!form.value.name.trim()) {
    errors.value.name = '请输入提示词名称'
  }

  if (!form.value.template.trim()) {
    errors.value.template = '请输入提示词模板'
  }

  if (form.value.type === 'url' && !form.value.urlPattern.trim()) {
    errors.value.urlPattern = '请输入网址匹配规则'
  }

  if (form.value.type === 'url') {
    try {
      new RegExp(form.value.urlPattern)
    } catch (e) {
      errors.value.urlPattern = '正则表达式格式错误'
    }
  }

  return Object.keys(errors.value).length === 0
}

function handleSave() {
  if (!validateForm()) {
    return
  }

  const data = {
    id: form.value.id || `prompt-${Date.now()}`,
    name: form.value.name.trim(),
    type: form.value.type,
    template: form.value.template.trim(),
    ...(form.value.type === 'url' && { urlPattern: form.value.urlPattern.trim() }),
    createdAt: props.prompt?.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString()
  }

  emit('save', data)
}

function handleCancel() {
  emit('close')
}
</script>

<template>
  <div v-if="show" class="modal modal-open">
    <div class="modal-box max-w-lg">
      <h3 class="font-bold text-lg mb-4">
        {{ prompt ? '编辑提示词' : '添加提示词' }}
      </h3>

      <div class="space-y-4">
        <!-- 名称 -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">名称</span>
          </label>
          <Input
            v-model="form.name"
            placeholder="输入提示词名称"
            :error="errors.name"
            :disabled="isSystem"
          />
        </div>

        <!-- 网址匹配规则（仅网址匹配类型显示，且不是系统提示词时） -->
        <div v-if="form.type === 'url' && !isSystem" class="form-control">
          <label class="label">
            <span class="label-text">网址匹配规则（正则表达式）</span>
          </label>
          <InputWithAI
            v-model="form.urlPattern"
            placeholder="输入网址，点击 AI 按钮自动生成正则表达式"
            :error="errors.urlPattern"
            :loading="isGeneratingRegex"
            @generate-regex="handleGenerateRegex"
          />
          <label class="label">
            <span class="label-text-alt text-[11px] text-base-content/40">
              匹配到网址后将使用此提示词
            </span>
          </label>
        </div>

        <!-- 提示词模板 -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">提示词模板</span>
          </label>
          <div class="relative mt-1">
            <textarea
              v-model="form.template"
              class="w-full rounded-lg border bg-base-100 px-3 py-2 text-sm text-base-content outline-none transition-all duration-200 focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100 placeholder:text-base-content/40"
              :class="[
                errors.template ? 'border-error ring-2 ring-error ring-offset-2 ring-offset-base-100' : 'border-base-300 focus:border-primary focus:ring-primary',
                'min-h-[120px] resize-y'
              ]"
              rows="6"
              placeholder="输入提示词内容，使用 {content} 作为网页内容占位符"
            ></textarea>
          </div>
          <label v-if="errors.template" class="label">
            <span class="label-text-alt text-error">{{ errors.template }}</span>
          </label>
          <label v-else class="label">
            <span class="label-text-alt text-[11px] text-base-content/40">
              使用 {content} 作为网页内容占位符
            </span>
          </label>
        </div>
      </div>

      <div class="modal-action">
        <Button variant="ghost" size="sm" @click="handleCancel">取消</Button>
        <Button variant="primary" size="sm" @click="handleSave">保存</Button>
      </div>
    </div>
    <div class="modal-backdrop" @click="handleCancel"></div>
  </div>
</template>