<script setup>
import { computed, ref, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps({
  modelValue: {
    type: [String, Number],
    required: true
  },
  options: {
    type: Array,
    default: () => []
  },
  optionLabel: {
    type: String,
    default: 'label'
  },
  optionValue: {
    type: String,
    default: 'value'
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
  placeholder: {
    type: String,
    default: '请选择'
  }
})

const emit = defineEmits(['update:modelValue', 'change', 'focus', 'blur'])

const isOpen = ref(false)
const selectRef = ref(null)
const dropdownRef = ref(null)
const dropdownPosition = ref({ top: 0, left: 0, width: 0, showAbove: false, triggerTop: 0, triggerBottom: 0 })

const selectedOption = computed(() => {
  if (!props.modelValue) return null
  return props.options.find(option => {
    const value = option[props.optionValue]
    return value === props.modelValue
  })
})

const displayLabel = computed(() => {
  return selectedOption.value ? selectedOption.value[props.optionLabel] : props.placeholder
})

const dropdownStyle = computed(() => {
  const position = dropdownPosition.value
  return {
    top: position.showAbove ? 'auto' : `${position.top}px`,
    bottom: position.showAbove ? `${window.innerHeight - position.triggerTop + 4}px` : 'auto',
    left: `${position.left}px`,
    width: `${position.width}px`,
    zIndex: 9999
  }
})

async function toggle() {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
    if (isOpen.value) {
      await nextTick()
      updateDropdownPosition()
    }
  }
}

function updateDropdownPosition() {
  if (selectRef.value && dropdownRef.value) {
    const rect = selectRef.value.getBoundingClientRect()
    const dropdownHeight = dropdownRef.value.scrollHeight
    const viewportHeight = window.innerHeight
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop
    const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft

    // 计算可用空间
    const spaceBelow = viewportHeight - rect.bottom - 20 // 20px 底部留白
    const spaceAbove = rect.top - 20 // 20px 顶部留白

    // 下拉框实际高度（限制最大高度）
    const maxDropdownHeight = 240
    const actualDropdownHeight = Math.min(dropdownHeight, maxDropdownHeight)

    // 判断是显示在上面还是下面
    // 只有当下方空间不足以容纳下拉框时，才考虑显示在上方
    const shouldShowAbove = spaceBelow < actualDropdownHeight && spaceAbove > spaceBelow

    dropdownPosition.value = {
      top: rect.bottom + 4,
      left: rect.left,
      width: rect.width,
      showAbove: shouldShowAbove,
      triggerTop: rect.top,
      triggerBottom: rect.bottom
    }
  }
}

function selectOption(option) {
  const value = option[props.optionValue]
  emit('update:modelValue', value)
  emit('change', value)
  isOpen.value = false
}

function handleClickOutside(event) {
  if (selectRef.value && !selectRef.value.contains(event.target)) {
    isOpen.value = false
  }
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

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside)
    window.addEventListener('resize', updateDropdownPosition)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('click', handleClickOutside)
    window.removeEventListener('resize', updateDropdownPosition)
  }
})
</script>

<template>
  <div ref="selectRef" class="relative">
    <!-- 触发按钮 -->
    <button
      type="button"
      :disabled="disabled"
      @click="toggle"
      :class="[
        'relative w-full rounded-lg border text-left transition-all duration-200 outline-none',
        'focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'flex items-center justify-between my-2',
        sizeClasses,
        isOpen ? 'border-primary ring-2 ring-primary ring-offset-2 ring-offset-base-100' : 'border-base-300',
        'bg-base-100 text-base-content hover:border-primary/50'
      ]"
    >
      <span :class="['truncate', !modelValue && 'text-base-content/50']">
        {{ displayLabel }}
      </span>
      <span
        :class="[
          'ml-2 transition-transform duration-200 flex-shrink-0',
          isOpen ? 'rotate-180' : ''
        ]"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M2.5 4.5L6 8L9.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </span>
    </button>

    <!-- 下拉选项 -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <div
          v-if="isOpen"
          ref="dropdownRef"
          :class="[
            'fixed rounded-lg border shadow-lg overflow-hidden',
            'bg-base-100 border-base-300'
          ]"
          :style="dropdownStyle"
        >
          <div class="max-h-60 overflow-y-auto no-scrollbar">
            <div
              v-for="(option, index) in options"
              :key="index"
              @click="selectOption(option)"
              :class="[
                'px-3 py-2 cursor-pointer transition-colors text-sm whitespace-nowrap',
                'hover:bg-primary/10 hover:text-primary',
                option[optionValue] === modelValue && 'bg-primary/10 text-primary font-medium',
                'text-base-content'
              ]"
            >
              {{ option[optionLabel] }}
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
