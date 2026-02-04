<script setup>
import { computed } from 'vue'
import { ChevronRight, ChevronDown, Check } from 'lucide-vue-next'

const props = defineProps({
  node: {
    type: Object,
    required: true
  },
  level: {
    type: Number,
    default: 0
  },
  selectedTagIds: {
    type: Array,
    default: () => []
  },
  expandedTags: {
    type: Set,
    default: () => new Set()
  }
})

const emit = defineEmits(['toggle-selection', 'toggle-expansion'])

const isExpanded = computed(() => props.expandedTags.has(props.node.name))
const isSelected = computed(() => props.selectedTagIds.includes(props.node.id))
const hasChildren = computed(() => props.node.children && props.node.children.length > 0)

function handleClick(event) {
  emit('toggle-selection', props.node.id, event)
}

function handleToggle(event) {
  event.stopPropagation()
  emit('toggle-expansion', props.node.name)
}

function getIndentStyle() {
  return {
    paddingLeft: `${props.level * 16}px`
  }
}
</script>

<template>
  <div>
    <!-- 标签节点 -->
    <div
      class="flex items-center gap-1 py-1.5 px-2 rounded-md cursor-pointer transition-colors"
      :class="{
        'bg-primary/10': isSelected,
        'hover:bg-base-200': !isSelected
      }"
      :style="getIndentStyle()"
      @click="handleClick"
    >
      <!-- 展开/收起箭头 -->
      <button
        v-if="hasChildren"
        @click="handleToggle"
        class="p-0.5 hover:bg-base-300 rounded transition-colors flex-shrink-0"
      >
        <ChevronDown v-if="isExpanded" :size="14" class="text-base-content/60" />
        <ChevronRight v-else :size="14" class="text-base-content/60" />
      </button>
      <span v-else class="w-5 flex-shrink-0"></span>

      <!-- 选中标记 -->
      <Check v-if="isSelected" :size="14" class="text-primary flex-shrink-0" />
      <span v-else class="w-4 flex-shrink-0"></span>

      <!-- 标签名称和计数 -->
      <span class="flex-1 text-sm truncate" :class="isSelected ? 'text-primary font-medium' : 'text-base-content'">
        {{ node.display_name }}
      </span>

      <!-- 笔记数量 -->
      <span class="text-xs text-base-content/40 flex-shrink-0">
        {{ node.count }}
      </span>
    </div>

    <!-- 子节点 -->
    <div v-if="hasChildren && isExpanded">
      <TagTreeNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :level="level + 1"
        :selected-tag-ids="selectedTagIds"
        :expanded-tags="expandedTags"
        @toggle-selection="emit('toggle-selection', $event)"
        @toggle-expansion="emit('toggle-expansion', $event)"
      />
    </div>
  </div>
</template>
