<script setup>
import { ref, watch } from 'vue'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'

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

const form = ref({
  id: null,
  name: '',
  type: 'url',
  urlPattern: '',
  template: ''
})

const errors = ref({})

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
          <Input
            v-model="form.urlPattern"
            placeholder="例如: ^https?://(www\\.)?github\\.com"
            :error="errors.urlPattern"
          />
          <label class="label">
            <span class="label-text-alt text-base-content/60">
              匹配到网址后将使用此提示词
            </span>
          </label>
        </div>

        <!-- 提示词模板 -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">提示词模板</span>
          </label>
          <div class="relative">
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
            <span class="label-text-alt text-base-content/60">
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