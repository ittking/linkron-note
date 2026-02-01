<script setup>
import { computed } from 'vue'

const props = defineProps({
  modelValue: {
    type: [String, Number],
    default: ''
  },
  type: {
    type: String,
    default: 'text'
  },
  placeholder: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  },
  size: {
    type: String,
    default: 'md',
    validator: (value) => ['sm', 'md', 'lg'].includes(value)
  },
  error: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'focus', 'blur', 'keyup', 'keydown'])

const value = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

function handleFocus(e) {
  emit('focus', e)
}

function handleBlur(e) {
  emit('blur', e)
}

function handleKeyup(e) {
  emit('keyup', e)
}

function handleKeydown(e) {
  emit('keydown', e)
}

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'px-3 py-1.5 text-sm min-h-[32px]'
    case 'lg':
      return 'px-4 py-3 text-lg min-h-[48px]'
    default: // md
      return 'px-3 py-2 text-sm min-h-[38px]'
  }
})
</script>

<template>
  <input
    :type="type"
    v-model="value"
    :placeholder="placeholder"
    :disabled="disabled"
    @focus="handleFocus"
    @blur="handleBlur"
    @keyup="handleKeyup"
    @keydown="handleKeydown"
    :class="[
      'w-full rounded-lg border transition-all duration-200 outline-none',
      'focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
      'placeholder:text-base-content/50',
      'disabled:cursor-not-allowed disabled:opacity-50',
      sizeClasses,
      error
        ? 'border-error focus:border-error focus:ring-error'
        : 'border-base-300 focus:border-primary focus:ring-primary',
      'bg-base-100 text-base-content'
    ]"
  />
</template>
