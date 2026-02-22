<script setup>
import { ref, computed } from 'vue'
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
const pressedModifiers = ref(new Set())
const inputRef = ref(null)

// 检测是否是 macOS（使用 navigator.userAgent 更加可靠）
const isMac = /Mac|iPod|iPhone|iPad/.test(navigator.userAgent)

// Code 到显示名称的映射（物理按键）
const codeToKey = {
  // 字母键
  'KeyA': 'A', 'KeyB': 'B', 'KeyC': 'C', 'KeyD': 'D', 'KeyE': 'E',
  'KeyF': 'F', 'KeyG': 'G', 'KeyH': 'H', 'KeyI': 'I', 'KeyJ': 'J',
  'KeyK': 'K', 'KeyL': 'L', 'KeyM': 'M', 'KeyN': 'N', 'KeyO': 'O',
  'KeyP': 'P', 'KeyQ': 'Q', 'KeyR': 'R', 'KeyS': 'S', 'KeyT': 'T',
  'KeyU': 'U', 'KeyV': 'V', 'KeyW': 'W', 'KeyX': 'X', 'KeyY': 'Y',
  'KeyZ': 'Z',
  // 数字键
  'Digit0': '0', 'Digit1': '1', 'Digit2': '2', 'Digit3': '3', 'Digit4': '4',
  'Digit5': '5', 'Digit6': '6', 'Digit7': '7', 'Digit8': '8', 'Digit9': '9',
  // 特殊键
  'Space': 'Space',
  'Enter': 'Enter', 'NumpadEnter': 'Enter',
  'Tab': 'Tab',
  'Escape': 'Escape', 'Backspace': 'Backspace', 'Delete': 'Delete'
}

// 修饰键 code 到显示名称的映射
const modifierCodeToName = {
  'AltLeft': isMac ? 'Option' : 'Alt',
  'AltRight': isMac ? 'Option' : 'Alt',
  'ControlLeft': 'Control',
  'ControlRight': 'Control',
  'MetaLeft': 'Command',
  'MetaRight': 'Command',
  'ShiftLeft': 'Shift',
  'ShiftRight': 'Shift'
}

// 开始录制
function startRecording() {
  isRecording.value = true
  pressedModifiers.value.clear()
}

// 停止录制
function stopRecording() {
  isRecording.value = false
  pressedModifiers.value.clear()
}

// 处理按键按下
function handleKeyDown(e) {
  if (!isRecording.value) return

  e.preventDefault()
  e.stopPropagation()

  const code = e.code

  // 检查是否是修饰键
  if (modifierCodeToName[code]) {
    pressedModifiers.value.add(modifierCodeToName[code])
    return
  }

  // 检查是否是有效的触发键
  const triggerKey = codeToKey[code]
  if (triggerKey) {
    const modifiers = Array.from(pressedModifiers.value).sort()

    if (modifiers.length > 0) {
      // 组合键格式：Modifier+Trigger
      const hotkey = modifiers.join('+') + '+' + triggerKey
      emit('update:modelValue', hotkey)
    } else {
      // 没有修饰键，只使用触发键
      emit('update:modelValue', triggerKey)
    }
    stopRecording()
  }
}

// 处理按键释放
function handleKeyUp(e) {
  if (!isRecording.value) return

  const code = e.code
  if (modifierCodeToName[code]) {
    pressedModifiers.value.delete(modifierCodeToName[code])
  }
}

// 处理失去焦点
function handleBlur() {
  if (isRecording.value) {
    stopRecording()
  }
}

// 格式化显示的快捷键
const displayHotkey = computed(() => {
  if (isRecording.value && pressedModifiers.value.size > 0) {
    const keys = Array.from(pressedModifiers.value)
    return keys.join('+') + '+...'
  }
  return props.modelValue || props.placeholder
})

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
</script>

<template>
  <div class="relative">
    <input
      ref="inputRef"
      :value="displayHotkey"
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
      @keyup="handleKeyUp"
    />
    <Keyboard
      :size="iconSize"
      class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/40 pointer-events-none"
    />
    <!-- 录制状态提示 -->
    <div
      v-if="isRecording"
      class="absolute inset-y-0 left-0 right-0 flex items-center justify-center px-3 text-primary font-medium"
    >
      <span v-if="pressedModifiers.size === 0">请按下快捷键组合...</span>
      <span v-else>{{ Array.from(pressedModifiers).join('+') }}+按下触发键</span>
    </div>
  </div>
</template>
