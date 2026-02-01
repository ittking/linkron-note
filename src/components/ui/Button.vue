<script setup>
import { computed } from 'vue'

const props = defineProps({
  type: {
    type: String,
    default: 'button',
    validator: (value) => ['button', 'submit', 'reset'].includes(value)
  },
  variant: {
    type: String,
    default: 'primary',
    validator: (value) => ['primary', 'secondary', 'ghost', 'error', 'success'].includes(value)
  },
  size: {
    type: String,
    default: 'md',
    validator: (value) => ['sm', 'md', 'lg'].includes(value)
  },
  disabled: {
    type: Boolean,
    default: false
  },
  loading: {
    type: Boolean,
    default: false
  },
  block: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['click'])

function handleClick(e) {
  if (!props.disabled && !props.loading) {
    emit('click', e)
  }
}

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'px-3 py-1.5 text-sm min-h-[32px] gap-1.5'
    case 'lg':
      return 'px-6 py-3 text-lg min-h-[48px] gap-2'
    default: // md
      return 'px-4 py-2 text-sm min-h-[38px] gap-2'
  }
})

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'secondary':
      return 'bg-secondary text-secondary-content hover:bg-secondary/80 border-2 border-transparent'
    case 'ghost':
      return 'bg-transparent text-base-content hover:bg-base-300/50'
    case 'error':
      return 'bg-error text-error-content hover:bg-error/90 border-2 border-transparent'
    case 'success':
      return 'bg-success text-success-content hover:bg-success/90 border-2 border-transparent'
    default: // primary
      return 'bg-primary text-primary-content hover:bg-primary/90 border-2 border-transparent'
  }
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    @click="handleClick"
    :class="[
      'inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'border',
      'whitespace-nowrap',
      sizeClasses,
      variantClasses,
      block && 'w-full',
      loading && 'cursor-wait',
      (variant === 'primary' || variant === 'ghost') && 'focus:ring-primary',
      variant === 'secondary' && 'focus:ring-secondary',
      variant === 'error' && 'focus:ring-error',
      variant === 'success' && 'focus:ring-success'
    ]"
  >
    <!-- Loading spinner -->
    <svg
      v-if="loading"
      class="animate-spin flex-shrink-0"
      :class="size === 'sm' ? 'w-4 h-4' : size === 'lg' ? 'w-6 h-6' : 'w-5 h-5'"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      ></circle>
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      ></path>
    </svg>

    <!-- Slot content -->
    <span :class="{ 'opacity-0': loading }" class="whitespace-nowrap">
      <slot />
    </span>
  </button>
</template>
