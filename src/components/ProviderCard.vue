<script setup>
import { computed } from 'vue'
import { Bot } from 'lucide-vue-next'

const props = defineProps({
  provider: {
    type: Object,
    required: true
  },
  isActive: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['select', 'delete', 'load-models', 'choose-model'])

const providerName = computed(() => {
  const names = {
    'deepseek': 'DeepSeek',
    'siliconflow': '硅基流动',
    'kimi': 'Kimi',
    'zhipu': '智谱AI',
    'custom': '自定义'
  }
  return props.provider.customName || names[props.provider.provider] || props.provider.provider
})

const providerIcon = computed(() => {
  return Bot
})

function handleLoadModels() {
  emit('load-models', props.provider)
}

function handleSelect() {
  emit('select', props.provider)
}
</script>

<template>
  <div 
    class="relative bg-base-100 rounded-lg border cursor-pointer transition-all hover:shadow-md"
    :class="{ 'border-primary ring-1 ring-primary': isActive, 'border-base-300': !isActive }"
    @click="handleSelect"
  >
    <div class="p-3">
      <div class="flex items-start justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded bg-primary/10 flex items-center justify-center flex-shrink-0">
            <component :is="providerIcon" :size="16" class="text-primary" />
          </div>
          <div class="min-w-0">
            <h3 class="text-sm font-medium truncate">{{ providerName }}</h3>
            <p class="text-xs text-base-content/60 mt-0.5">
              {{ provider.models?.length || 0 }} 个模型
            </p>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <span 
            @click.stop="emit('choose-model', provider)"
            class="px-1.5 rounded text-primary hover:bg-primary/20 transition-colors flex-shrink-0"
            title="选择模型"
            size="sm"
          >
            选择
        </span>
        </div>
      </div>
      <div v-if="provider.currentModel" class="pt-2 border-t border-base-200">
        <p class="text-xs text-base-content/50 mb-0.5">当前模型</p>
        <p class="text-xs font-medium text-primary truncate">{{ provider.currentModel }}</p>
      </div>
    </div>
  </div>
</template>