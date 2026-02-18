<script setup>
import { computed, ref } from 'vue'
import { Bot, MoreHorizontal, Edit2, Database, Trash2 } from 'lucide-vue-next'
import Dropdown from './ui/Dropdown.vue'

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

const emit = defineEmits(['select', 'delete', 'load-models', 'choose-model', 'edit'])

const dropdownRef = ref(null)

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
  dropdownRef.value?.close()
}

function handleSelect() {
  emit('select', props.provider)
}

function handleEdit() {
  emit('edit', props.provider)
  dropdownRef.value?.close()
}

function handleDelete() {
  emit('delete', props.provider)
  dropdownRef.value?.close()
}

function handleChooseModel() {
  emit('choose-model', props.provider)
  dropdownRef.value?.close()
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
        <Dropdown ref="dropdownRef" position="bottom-end">
          <template #trigger="{ isOpen, toggle }">
            <button
              @click.stop="toggle"
              class="p-1.5 rounded hover:bg-base-200 transition-colors text-base-content/60 hover:text-base-content"
            >
              <MoreHorizontal :size="16" />
            </button>
          </template>
          <template #default="{ close }">
            <div class="py-1 min-w-[120px]">
              <button
                @click.stop="handleEdit"
                class="w-full px-3 py-2 text-left text-sm hover:bg-base-200 transition-colors flex items-center gap-2"
              >
                <Edit2 :size="14" />
                编辑
              </button>
              <button
                @click.stop="handleChooseModel"
                class="w-full px-3 py-2 text-left text-sm hover:bg-base-200 transition-colors flex items-center gap-2"
              >
                <Database :size="14" />
                模型
              </button>
              <button
                @click.stop="handleDelete"
                class="w-full px-3 py-2 text-left text-sm hover:bg-base-200 transition-colors flex items-center gap-2 text-error"
              >
                <Trash2 :size="14" />
                删除
              </button>
            </div>
          </template>
        </Dropdown>
      </div>
      <div v-if="provider.currentModel" class="pt-2 border-t border-base-200">
        <p class="text-xs text-base-content/50 mb-0.5">当前模型</p>
        <p class="text-xs font-medium text-primary truncate">{{ provider.currentModel }}</p>
      </div>
    </div>
  </div>
</template>