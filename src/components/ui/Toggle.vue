<script setup>
import { computed } from 'vue'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  size: {
    type: String,
    default: 'md',
    validator: (value) => ['sm', 'md', 'lg'].includes(value)
  }
})

const emit = defineEmits(['update:modelValue'])

function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
}

// 轨道尺寸和小球尺寸
const sizes = {
  sm: { track: 'w-8 h-4', thumb: 'w-3 h-3', offset: 'translate-x-[15px]' },
  md: { track: 'w-10 h-5', thumb: 'w-4 h-4', offset: 'translate-x-[20px]' },
  lg: { track: 'w-12 h-6', thumb: 'w-5 h-5', offset: 'translate-x-[25px]' }
}

const currentSize = computed(() => sizes[props.size])
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    @click="toggle"
    :class="[
      'relative rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
      currentSize.track,
      modelValue ? 'bg-primary' : 'bg-base-100',
      disabled && 'opacity-50 cursor-not-allowed'
    ]"
    role="switch"
    :aria-checked="modelValue"
    :aria-disabled="disabled"
  >
    <span
      :class="[
        'absolute top-1/2 -translate-y-1/2 left-0.5 rounded-full bg-white shadow-sm transition-transform duration-200',
        currentSize.thumb,
        modelValue && currentSize.offset
      ]"
      aria-hidden="true"
    />
  </button>
</template>
