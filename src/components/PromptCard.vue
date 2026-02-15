<script setup>
import { Trash2, Edit2, Shield } from 'lucide-vue-next'
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
    class="card bg-base-100 shadow-sm hover:shadow-md transition-shadow cursor-pointer"
    :class="{ 'ring-2 ring-primary ring-offset-2': isActive }"
  >
    <div class="card-body p-3">
      <div class="flex items-start justify-between gap-2">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 mb-1">
            <span class="font-medium text-sm truncate">{{ prompt.name }}</span>
            <span v-if="isSystem" class="badge badge-xs badge-primary flex items-center gap-1">
              <Shield :size="10" />
              系统默认
            </span>
            <span v-else class="badge badge-xs" :class="getPromptTypeBadgeClass(prompt.type)">
              {{ getPromptTypeLabel(prompt.type) }}
            </span>
          </div>
          <p v-if="prompt.type === 'url' && !isSystem" class="text-xs text-base-content/60 font-mono">
            {{ prompt.urlPattern }}
          </p>
          <p class="text-xs text-base-content/40 truncate mt-1">
            {{ prompt.template.substring(0, 50) }}{{ prompt.template.length > 50 ? '...' : '' }}
          </p>
        </div>
        <div class="flex gap-1">
          <Button variant="ghost" size="icon-xs" @click.stop="$emit('edit', prompt)">
            <Edit2 :size="12" />
          </Button>
          <Button 
            v-if="!isSystem"
            variant="ghost" 
            size="icon-xs" 
            class="text-error hover:text-error" 
            @click.stop="$emit('delete', prompt)"
          >
            <Trash2 :size="12" />
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>