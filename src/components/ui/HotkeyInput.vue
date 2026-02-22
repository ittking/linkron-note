<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Keyboard } from 'lucide-vue-next'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  placeholder: {
    type: String,
    default: '点击输入快捷键'
  },
  size: {
    type: String,
    default: 'md',
    validator: (value) => ['sm', 'md', 'lg'].includes(value)
  }
})

const emit = defineEmits(['update:modelValue'])

const isRecording = ref(false)
const supportedKeys = ref([])
const inputRef = ref(null)

// 获取支持的按键列表
async function loadSupportedKeys() {
  try {
    supportedKeys.value = await invoke('get_supported_keys')
  } catch (error) {
    console.error('获取支持的按键列表失败:', error)
  }
}

// 开始录制
function startRecording() {
  isRecording.value = true
}

// 停止录制
function stopRecording() {
  isRecording.value = false
}

// 处理按键按下
function handleKeyDown(e) {
  if (!isRecording.value) return

  e.preventDefault()
  e.stopPropagation()

  const key = e.key

  // 检查是否是支持的按键
  const normalizedKey = supportedKeys.value.find(k => {
    const keyLower = k.toLowerCase()
    if (keyLower === 'option' && key === 'Alt') return true
    if (keyLower === 'alt' && key === 'Alt') return true
    if (keyLower === 'control' && key === 'Control') return true
    if (keyLower === 'command' && key === 'Meta') return true
    if (keyLower === 'shift' && key === 'Shift') return true
    return false
  })

  if (normalizedKey) {
    emit('update:modelValue', normalizedKey)
    // 不停止录制，允许用户继续按其他键切换
  }
}

// 处理失去焦点
function handleBlur() {
  if (isRecording.value) {
    stopRecording()
  }
}

// 计算样式
const sizeClasses = computed(() => {
  const sizes = {
    sm: 'px-2 py-1 text-sm',
    md: 'px-3 py-2',
    lg: 'px-4 py-3 text-lg'
  }
  return sizes[props.size] || sizes.md
})

const iconSize = computed(() => {
  const sizes = {
    sm: 14,
    md: 16,
    lg: 20
  }
  return sizes[props.size] || 16
})

onMounted(() => {
  loadSupportedKeys()
})

// 监听录制状态
watch(isRecording, (newVal) => {
  if (newVal) {
    // 聚焦输入框
    inputRef.value?.focus()
  }
})
</script>

<template>
  <div class="relative">
    <input
      ref="inputRef"
      :value="modelValue"
      :placeholder="placeholder"
      :class="[
        'input input-bordered w-full flex items-center gap-2 cursor-pointer my-2',
        sizeClasses,
        isRecording ? 'input-primary' : ''
      ]"
      readonly
      @focus="startRecording"
      @blur="handleBlur"
      @keydown="handleKeyDown"
    />
    <Keyboard
      :size="iconSize"
      class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/40 pointer-events-none"
    />
    <!-- 录制状态指示 -->
    <div
      v-if="isRecording"
      class="absolute inset-y-0 left-0 right-0 flex items-center justify-center px-3 text-primary font-medium"
    >
      {{ modelValue || '请按键' }}
    </div>
  </div>
</template>