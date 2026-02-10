<script setup>
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Search, X, ChevronDown, ChevronRight, Tag } from 'lucide-vue-next'

const props = defineProps({
  isOpen: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'select-tag'])

// 搜索关键词
const searchQuery = ref('')

// 标签列表
const tags = ref([])
const loading = ref(false)

// 展开的标签节点
const expandedNodes = ref(new Set())

// 加载标签
async function loadTags() {
  loading.value = true
  try {
    const allTags = await invoke('get_all_tags')
    tags.value = allTags
  } catch (error) {
    console.error('加载标签失败:', error)
  } finally {
    loading.value = false
  }
}

// 监听侧边栏打开状态
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    loadTags()
  }
})

// 切换节点展开状态
function toggleNode(nodePath) {
  if (expandedNodes.value.has(nodePath)) {
    expandedNodes.value.delete(nodePath)
  } else {
    expandedNodes.value.add(nodePath)
  }
}

// 构建树形结构
const tagTree = computed(() => {
  if (!tags.value || tags.value.length === 0) {
    return []
  }

  // 构建映射
  const tagMap = new Map()
  tags.value.forEach(tag => {
    tagMap.set(tag.id, { ...tag, children: [] })
  })

  // 构建树
  const roots = []
  tags.value.forEach(tag => {
    const node = tagMap.get(tag.id)
    if (tag.parent_id) {
      const parent = tagMap.get(tag.parent_id)
      if (parent) {
        parent.children.push(node)
      } else {
        roots.push(node)
      }
    } else {
      roots.push(node)
    }
  })

  return roots
})

// 过滤后的树
const filteredTree = computed(() => {
  if (!searchQuery.value) {
    return tagTree.value
  }

  const query = searchQuery.value.toLowerCase()

  function filterNodes(nodes) {
    return nodes.reduce((acc, node) => {
      const nameMatches = node.name.toLowerCase().includes(query)
      const fullNameMatches = node.full_name.toLowerCase().includes(query)
      const filteredChildren = filterNodes(node.children)

      if (nameMatches || fullNameMatches || filteredChildren.length > 0) {
        acc.push({
          ...node,
          children: filteredChildren
        })
      }

      return acc
    }, [])
  }

  return filterNodes(tagTree.value)
})

// 递归渲染标签树
function renderTree(nodes, parentPath = '') {
  return nodes.map(node => {
    const currentPath = parentPath ? `${parentPath}/${node.name}` : node.name
    const hasChildren = node.children && node.children.length > 0
    const isExpanded = expandedNodes.value.has(currentPath)

    return `
      <div class="tag-tree-node">
        <div class="tag-tree-item flex items-center gap-2 px-3 py-2 hover:bg-base-200 cursor-pointer rounded-lg transition-colors" data-tag-path="${currentPath}">
          ${hasChildren ? `
            <button class="expand-btn w-5 h-5 flex items-center justify-center text-base-content/40 hover:text-base-content transition-colors" data-action="toggle" data-path="${currentPath}">
              ${isExpanded ? '<ChevronDown :size="14" />' : '<ChevronRight :size="14" />'}
            </button>
          ` : '<span class="w-5"></span>'}
          <Tag :size="14" class="text-primary flex-shrink-0" />
          <span class="text-sm text-base-content truncate">${node.name}</span>
        </div>
        ${hasChildren && isExpanded ? `
          <div class="tag-tree-children ml-4 pl-2 border-l border-base-300">
            ${renderTree(node.children, currentPath)}
          </div>
        ` : ''}
      </div>
    `
  }).join('')
}

// 处理标签点击
function handleTagClick(event) {
  const target = event.target.closest('[data-tag-path]')
  if (!target) return

  const action = event.target.closest('[data-action]')
  const path = target.dataset.tagPath

  if (action && action.dataset.action === 'toggle') {
    event.stopPropagation()
    toggleNode(path)
  } else {
    emit('select-tag', path)
  }
}
</script>

<template>
  <div class="fixed inset-y-0 left-0 z-50 transition-transform duration-300 ease-in-out"
    :class="isOpen ? 'translate-x-0' : '-translate-x-full'">
    <div class="w-80 h-full bg-base-100 border-r border-base-300 flex flex-col shadow-xl">
      <!-- 头部 -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-base-300">
        <h2 class="text-base font-semibold text-base-content">标签</h2>
        <button @click="emit('close')"
          class="w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200">
          <X :size="16" />
        </button>
      </div>

      <!-- 搜索框 -->
      <div class="px-4 py-3 border-b border-base-300">
        <div class="relative">
          <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
          <input v-model="searchQuery" type="text" placeholder="搜索标签..."
            class="w-full pl-9 pr-4 py-2 bg-base-200 border border-base-300 rounded-lg text-sm text-base-content placeholder:text-base-content/40 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all" />
        </div>
      </div>

      <!-- 标签列表 -->
      <div class="flex-1 overflow-y-auto p-4" @click="handleTagClick">
        <div v-if="loading" class="flex justify-center py-8">
          <span class="loading loading-spinner text-primary"></span>
        </div>

        <div v-else-if="filteredTree.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/40">
          <Tag :size="48" class="mb-3 opacity-50" />
          <div class="text-sm">{{ searchQuery ? '未找到匹配的标签' : '暂无标签' }}</div>
        </div>

        <div v-else class="tag-tree">
          <div v-html="renderTree(filteredTree)"></div>
        </div>
      </div>
    </div>

    <!-- 遮罩层 -->
    <div v-if="isOpen" @click="emit('close')"
      class="fixed inset-0 bg-black/30 backdrop-blur-sm transition-opacity duration-300 -z-10">
    </div>
  </div>
</template>

<style scoped>
.tag-tree-item:hover {
  background-color: var(--color-base-200, #f5f5f5);
}

.tag-tree-children {
  border-left-color: var(--color-base-300, #e5e5e5);
}
</style>