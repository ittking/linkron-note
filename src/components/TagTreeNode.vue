<script setup>
import { ChevronDown, ChevronRight, Tag, MoreVertical, Pin, PinOff, Trash2 } from 'lucide-vue-next'

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
  },
  activeMenuTagId: {
    type: String,
    default: null
  }
})

const emit = defineEmits(['toggle-node', 'toggle-menu', 'delete-tag', 'toggle-pin', 'click'])

// 添加日志查看 node 结构
console.log('[TagTreeNode] node 对象:', props.node)
console.log('[TagTreeNode] node.fullName:', props.node.fullName)
console.log('[TagTreeNode] node.full_name:', props.node.full_name)
</script>

<template>
  <div class="tag-tree-node">
    <div
      class="tag-tree-item flex items-center gap-2 px-3 py-2 hover:bg-base-200 cursor-pointer rounded-lg transition-colors group relative"
      @click="emit('click', node)"
    >
      <!-- 展开/收起按钮 -->
      <span class="w-6 h-6 flex items-center justify-center">
        <template v-if="node.children && node.children.length > 0">
          <button
            @click.stop="emit('toggle-node', node.fullName)"
            class="expand-btn w-6 h-6 flex items-center justify-center text-base-content/40 hover:text-base-content transition-colors"
          >
            <ChevronRight v-if="!expandedNodes.has(node.fullName)" :size="18" />
            <ChevronDown v-else :size="18" />
          </button>
        </template>
      </span>

      <!-- 标签图标 -->
      <Tag :size="14" class="text-primary flex-shrink-0" />

      <!-- 标签名称 -->
      <span class="text-sm text-base-content truncate flex-1">{{ node.name }}</span>

      <!-- 更多按钮 -->
      <button
        @click.stop="emit('toggle-menu', node.id, $event)"
        class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-all duration-200 opacity-0 group-hover:opacity-100"
      >
        <MoreVertical :size="14" />
      </button>

      <!-- 下拉菜单 -->
      <div
        v-if="activeMenuTagId === node.id"
        @click.stop
        class="absolute right-2 top-full mt-1 z-20 bg-base-100 border border-base-300 rounded-lg shadow-lg py-1 min-w-[120px]"
      >
        <!-- 置顶/取消置顶 -->
        <button
          @click="emit('toggle-pin', node.id, $event)"
          class="w-full px-3 py-2 flex items-center gap-2 text-sm text-base-content hover:bg-base-200 transition-colors"
        >
          <Pin v-if="!node.pinned" :size="14" />
          <PinOff v-else :size="14" />
          <span>{{ node.pinned ? '取消置顶' : '置顶' }}</span>
        </button>

        <!-- 删除 -->
        <button
          @click="emit('delete-tag', node.id, $event)"
          class="w-full px-3 py-2 flex items-center gap-2 text-sm text-error hover:bg-base-200 transition-colors"
        >
          <Trash2 :size="14" />
          <span>删除</span>
        </button>
      </div>
    </div>

    <!-- 子节点 -->
    <div
      v-if="node.children && node.children.length > 0 && expandedNodes.has(node.fullName)"
      class="tag-tree-children ml-6 border-l border-base-300 space-y-1"
    >
      <TagTreeNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :level="level + 1"
        :expanded-nodes="expandedNodes"
        :active-menu-tag-id="activeMenuTagId"
        @toggle-node="emit('toggle-node', $event)"
        @toggle-menu="(tagId, event) => emit('toggle-menu', tagId, event)"
        @delete-tag="(tagId, event) => emit('delete-tag', tagId, event)"
        @toggle-pin="(tagId, event) => emit('toggle-pin', tagId, event)"
        @click="emit('click', $event)"
      />
    </div>
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

.tag-tree-children .tag-tree-item {
  padding-top: 2px;
  padding-bottom: 2px;
}
</style>