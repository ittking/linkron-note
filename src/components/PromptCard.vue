<script setup>
import { Trash2, Edit2, Shield, MessageSquare } from 'lucide-vue-next'
import Button from './ui/Button.vue'

const props = defineProps({
  prompt: {
    type: Object,
    required: true
  },
  isActive: {
    type: Boolean,
    default: false
  },
  isSystem: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['delete', 'edit'])

function getPromptTypeLabel(type) {
  switch (type) {
    case 'general':
      return '通用'
    case 'url':
      return '网址匹配'
    default:
      return type
  }
}

function getPromptTypeBadgeClass(type) {
  switch (type) {
    case 'general':
      return 'badge-info'
    case 'url':
      return 'badge-success'
    default:
      return 'badge-neutral'
  }
}
</script>

<template>
  <div 
    class="relative bg-base-100 rounded-lg border cursor-pointer transition-all hover:shadow-md"
    :class="{ 'border-primary ring-1 ring-primary': isActive, 'border-base-300': !isActive }"
  >
    <div class="p-3">
      <div class="flex items-start justify-between gap-2 mb-2">
        <div class="flex items-center gap-2 flex-1 min-w-0">
          <div class="w-8 h-8 rounded bg-secondary/10 flex items-center justify-center flex-shrink-0">
            <MessageSquare :size="16" class="text-secondary" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium truncate">{{ prompt.name }}</span>
              <span v-if="isSystem" class="badge badge-xs badge-primary flex items-center gap-1 flex-shrink-0">
                <Shield :size="10" />
                系统默认
              </span>
              <span v-else class="badge badge-xs flex-shrink-0" :class="getPromptTypeBadgeClass(prompt.type)">
                {{ getPromptTypeLabel(prompt.type) }}
              </span>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-1 flex-shrink-0">
          <Button variant="ghost" size="sm" @click.stop="$emit('edit', prompt)">
            <Edit2 :size="12" />
          </Button>
          <Button 
            v-if="!isSystem"
            variant="ghost" 
            size="sm" 
            class="text-error hover:text-error" 
            @click.stop="$emit('delete', prompt)"
          >
            <Trash2 :size="12" />
          </Button>
        </div>
      </div>
      <div v-if="prompt.type === 'url' && !isSystem" class="pt-2 border-t border-base-200">
        <p class="text-xs text-base-content/50 mb-0.5">正则表达式</p>
        <p class="text-xs text-base-content/70 font-mono truncate">{{ prompt.urlPattern }}</p>
      </div>
      <div class="pt-2 border-t border-base-200 mt-2">
        <p class="text-xs text-base-content/50 mb-0.5">模板预览</p>
        <p class="text-xs text-base-content/60 truncate">
          {{ prompt.template.substring(0, 60) }}{{ prompt.template.length > 60 ? '...' : '' }}
        </p>
      </div>
    </div>
  </div>
</template>