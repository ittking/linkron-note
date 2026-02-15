<script setup>
import { computed } from 'vue'
import { Sparkles } from 'lucide-vue-next'
import Button from './Button.vue'

const props = defineProps({
  modelValue: {
    type: [String, Number],
    default: ''
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
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'generate-regex'])

const value = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

function handleGenerateRegex() {
  emit('generate-regex')
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
  <div class="relative">
    <input
      v-model="value"
      :placeholder="placeholder"
      :disabled="disabled || loading"
      :class="[
        'w-full rounded-lg border transition-all duration-200 outline-none pr-12',
        'focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
        'placeholder:text-base-content/50',
        'disabled:cursor-not-allowed disabled:opacity-50 my-2',
        sizeClasses,
        error
          ? 'border-error focus:border-error focus:ring-error'
          : 'border-base-300 focus:border-primary focus:ring-primary',
        'bg-base-100 text-base-content'
      ]"
    />
    <div class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        :disabled="!value || loading"
        @click="handleGenerateRegex"
        class="hover:bg-primary/10"
      >
        <Sparkles :size="14" :class="{ 'animate-spin': loading }" />
      </Button>
    </div>
  </div>
</template>