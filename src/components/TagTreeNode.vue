<script setup>
import { ChevronDown, ChevronRight, MoreVertical, Pin, PinOff, Trash2 } from 'lucide-vue-next'
import { computed } from 'vue'
import Dropdown from './ui/Dropdown.vue'

const props = defineProps({
  node: {
    type: Object,
    required: true
  },
  level: {
    type: Number,
    default: 0
  },
  expandedNodes: {
    type: Set,
    required: true
  }
})

const emit = defineEmits(['toggle-node', 'delete-tag', 'toggle-pin', 'click'])

// 判断是否显示"置顶"文本（根节点且被置顶）
const shouldShowPinnedText = computed(() => {
  return props.level === 0 && props.node.pinned
})
</script>

<template>
  <div class="tag-tree-node">
    <div
      class="tag-tree-item flex items-center gap-2 pl-3 pr-1 py-2 hover:bg-base-200 cursor-pointer rounded-lg transition-colors group relative">
      <!-- 标签图标或置顶文本 -->
      <span v-if="!shouldShowPinnedText" class="text-primary flex-shrink-0 text-sm opacity-50">#</span>
      <div v-else class="flex items-center gap-1 flex-shrink-0">
        <span class="text-xs text-primary font-medium">置顶</span>
        <span class="w-1 h-1 rounded-full bg-primary"></span>
      </div>

      <!-- 标签名称 -->
      <div class="flex-1">
        <span class="text-sm text-base-content truncate max-w-full" @click="emit('click', node)">{{ node.name }}</span>
      </div>

      <!-- 下拉菜单 -->
      <Dropdown position="bottom-end">
        <template #trigger="{ toggle }">
          <button @click.stop="toggle"
            class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-all duration-200 opacity-0 group-hover:opacity-100">
            <MoreVertical :size="14" />
          </button>
        </template>
        <template #default="{ close }">
          <!-- 置顶/取消置顶 -->
          <button @click.stop="emit('toggle-pin', node.id, $event); close()"
            class="w-full px-3 py-2 flex items-center gap-2 text-sm text-base-content hover:bg-base-200 transition-colors">
            <Pin v-if="!node.pinned" :size="14" />
            <PinOff v-else :size="14" />
            <span>{{ node.pinned ? '取消置顶' : '置顶' }}</span>
          </button>

          <!-- 删除 -->
          <button @click.stop="emit('delete-tag', node.id, $event); close()"
            class="w-full px-3 py-2 flex items-center gap-2 text-sm text-error hover:bg-base-200 transition-colors">
            <Trash2 :size="14" />
            <span>删除</span>
          </button>
        </template>
      </Dropdown>

      <!-- 展开/收起按钮 -->
      <span class="w-6 h-6 flex items-center justify-center">
        <template v-if="node.children && node.children.length > 0">
          <button @click.stop="emit('toggle-node', node.fullName)"
            class="expand-btn w-6 h-6 flex items-center justify-center text-base-content/40 hover:text-base-content transition-colors">
            <ChevronRight :size="18" :class="{ 'rotate-90': expandedNodes.has(node.fullName) }"
              class="transition-transform duration-200" />
          </button>
        </template>
      </span>
    </div>

    <!-- 子节点 -->
    <Transition enter-active-class="transition-all duration-200 ease-out" enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0" leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-1">
      <div v-if="node.children && node.children.length > 0 && expandedNodes.has(node.fullName)"
        class="tag-tree-children ml-5 pl-1 border-l border-base-300 space-y-1">
        <TagTreeNode v-for="child in node.children" :key="child.id" :node="child" :level="level + 1"
          :expanded-nodes="expandedNodes" @toggle-node="emit('toggle-node', $event)"
          @delete-tag="(tagId, event) => emit('delete-tag', tagId, event)"
          @toggle-pin="(tagId, event) => emit('toggle-pin', tagId, event)" @click="emit('click', $event)" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.tag-tree-item:hover {
  background-color: var(--color-base-200, #f5f5f5);
}

.tag-tree-children {
  border-left-color: var(--color-base-300, #e5e5e5);
  margin-top: 2px;
  margin-bottom: 2px;
}
</style>