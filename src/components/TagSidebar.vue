<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { Search, X } from 'lucide-vue-next'
import { useNoteStore } from '@/store/noteStore'
import TagTreeNode from './TagTreeNode.vue'

const noteStore = useNoteStore()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'filter'])

const searchQuery = ref('')
const tagTree = ref([])
const selectedTagIds = ref([])
const expandedTags = ref(new Set())

// 获取标签树
async function loadTagTree() {
  try {
    const tags = await noteStore.getTagsWithStats()
    tagTree.value = buildTagTree(tags)
  } catch (error) {
    console.error('Failed to load tag tree:', error)
  }
}

// 构建树形结构
function buildTagTree(tags) {
  const root = []
  const map = new Map()

  tags.forEach(({ tag, count }) => {
    map.set(tag.name, {
      ...tag,
      count,
      children: []
    })
  })

  tags.forEach(({ tag, count }) => {
    const node = map.get(tag.name)
    node.count = count

    if (tag.path === '') {
      root.push(node)
    } else {
      const parent = map.get(tag.path)
      if (parent) {
        parent.children.push(node)
      }
    }
  })

  return root
}

// 搜索过滤
const filteredTree = computed(() => {
  if (!searchQuery.value) {
    return tagTree.value
  }

  const query = searchQuery.value.toLowerCase()
  const filter = (nodes) => {
    return nodes.reduce((acc, node) => {
      const matches = node.name.toLowerCase().includes(query) ||
                      node.display_name.toLowerCase().includes(query)
      const filteredChildren = node.children.length > 0 ? filter(node.children) : []

      if (matches || filteredChildren.length > 0) {
        acc.push({
          ...node,
          children: filteredChildren
        })
      }

      return acc
    }, [])
  }

  return filter(tagTree.value)
})

// 选择/取消选择标签
function toggleTagSelection(tagId, event) {
  if (event.metaKey || event.ctrlKey) {
    // 多选
    const index = selectedTagIds.value.indexOf(tagId)
    if (index === -1) {
      selectedTagIds.value.push(tagId)
    } else {
      selectedTagIds.value.splice(index, 1)
    }
  } else {
    // 单选
    selectedTagIds.value = [tagId]
  }

  // 触发笔记筛选
  filterNotes()
}

// 筛选笔记
async function filterNotes() {
  if (selectedTagIds.value.length === 0) {
    emit('filter', null)
    return
  }

  try {
    const tags = tagTree.value
    const selectedNames = getSelectedTagNames(tags, selectedTagIds.value)
    const filteredNotes = await noteStore.getNotesByTags(selectedNames)
    emit('filter', filteredNotes)
  } catch (error) {
    console.error('Failed to filter notes:', error)
  }
}

// 递归获取选中的标签名称
function getSelectedTagNames(nodes, selectedIds) {
  const names = []
  for (const node of nodes) {
    if (selectedIds.includes(node.id)) {
      names.push(node.name)
    }
    if (node.children.length > 0) {
      names.push(...getSelectedTagNames(node.children, selectedIds))
    }
  }
  return names
}

// 清除筛选
function clearFilter() {
  selectedTagIds.value = []
  emit('filter', null)
}

// 展开/收起标签
function toggleTagExpansion(tagName) {
  if (expandedTags.value.has(tagName)) {
    expandedTags.value.delete(tagName)
  } else {
    expandedTags.value.add(tagName)
  }
}

watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadTagTree()
  }
})

onMounted(() => {
  if (props.visible) {
    loadTagTree()
  }
})
</script>

<template>
  <Transition name="slide">
    <div v-if="visible" class="fixed left-0 top-0 h-full w-64 bg-base-100 border-r border-base-300 shadow-xl z-40 flex flex-col">
      <!-- 头部 -->
      <div class="p-4 border-b border-base-300">
        <div class="flex items-center justify-between mb-3">
          <h2 class="font-semibold text-base-content flex items-center gap-2">
            🏷️ 标签
          </h2>
          <button @click="emit('close')" class="p-1 hover:bg-base-200 rounded transition-colors">
            <X :size="18" class="text-base-content/60" />
          </button>
        </div>

        <!-- 搜索框 -->
        <div class="relative">
          <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索标签..."
            class="w-full pl-9 pr-3 py-2 bg-base-200 border border-base-300 rounded-md text-sm focus:outline-none focus:border-primary"
          />
        </div>
      </div>

      <!-- 标签树 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="filteredTree.length === 0" class="text-center text-base-content/40 py-8">
          {{ searchQuery ? '没有匹配的标签' : '暂无标签' }}
        </div>
        <TagTreeNode
          v-for="node in filteredTree"
          :key="node.id"
          :node="node"
          :selected-tag-ids="selectedTagIds"
          :expanded-tags="expandedTags"
          @toggle-selection="toggleTagSelection"
          @toggle-expansion="toggleTagExpansion"
        />
      </div>

      <!-- 底部筛选状态 -->
      <div v-if="selectedTagIds.length > 0" class="p-3 border-t border-base-300 bg-base-200">
        <div class="flex items-center justify-between text-sm">
          <span class="text-base-content/80">已选: {{ selectedTagIds.length }} 个标签</span>
          <button
            @click="clearFilter"
            class="px-2 py-1 text-xs bg-base-300 hover:bg-base-400 rounded transition-colors"
          >
            清除筛选
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(-100%);
}
</style>
