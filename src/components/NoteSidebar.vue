<script setup>
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, Tag } from 'lucide-vue-next'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import TagTreeNode from './TagTreeNode.vue'
import NoteHeatmap from './NoteHeatmap.vue'

const props = defineProps({
  isOpen: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'select-tag'])

// 使用 useWorkDirectory composable
const { getWorkDirectory } = useWorkDirectory('setting')

// 标签列表
const tags = ref([])
const loading = ref(false)

// 展开的标签节点
const expandedNodes = ref(new Set())

// 加载标签
async function loadTags() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const allTags = await invoke('get_all_tags', { workDirectory })
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

  // 第一步：构建映射，先创建所有节点
  const tagMap = new Map()
  tags.value.forEach(tag => {
    tagMap.set(tag.id, { ...tag, children: [] })
  })

  // 第二步：构建树，建立父子关系
  const roots = []
  tags.value.forEach(tag => {
    const node = tagMap.get(tag.id)
    if (tag.parentId) {
      const parent = tagMap.get(tag.parentId)
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

// 删除标签
async function deleteTag(tagId, event) {
  event.stopPropagation()
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('delete_tag', { id: tagId, workDirectory })
    await loadTags()
  } catch (error) {
    console.error('删除标签失败:', error)
  }
}

// 置顶/取消置顶标签
async function togglePin(tagId, event) {
  event.stopPropagation()
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('pin_tag', { id: tagId, workDirectory })
    await loadTags()
  } catch (error) {
    console.error('置顶标签失败:', error)
  }
}

// 处理标签点击
function handleTagClick(tag) {
  emit('select-tag', tag.fullName)
}
</script>

<template>
  <!-- 遮罩层 -->
  <Transition name="fade">
    <div v-if="isOpen" @click="emit('close')"
      class="fixed inset-0 bg-black/30 backdrop-blur-sm z-40 cursor-pointer">
    </div>
  </Transition>

  <!-- 侧边栏 -->
  <div class="fixed inset-y-0 left-0 z-50 transition-transform duration-300 ease-in-out"
    :class="isOpen ? 'translate-x-0' : '-translate-x-full'">
    <div data-tauri-drag-region class="w-80 h-full bg-base-100 flex flex-col shadow-xl">
      <!-- 头部 -->
      <div data-tauri-drag-region class="flex items-center px-4 py-3 border-b border-base-300">
        <h2 data-tauri-drag-region class="text-base font-semibold text-base-content">ITERM</h2>
        <div data-tauri-drag-region class="flex-1 h-full"></div>
        <button @click="emit('close')"
          class="w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200">
          <X :size="16" />
        </button>
      </div>

      <!-- 笔记热度图 -->
      <div class="px-4 py-3">
        <NoteHeatmap />
      </div>

      <!-- 标签列表 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="loading" class="flex justify-center py-8">
          <span class="loading loading-spinner text-primary"></span>
        </div>

        <div v-else-if="tagTree.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/40">
          <Tag :size="48" class="mb-3 opacity-50" />
          <div class="text-sm">暂无标签</div>
        </div>

        <div v-else class="tag-tree space-y-1">
          <TagTreeNode
            v-for="node in tagTree"
            :key="node.id"
            :node="node"
            :level="0"
            :expanded-nodes="expandedNodes"
            @toggle-node="toggleNode"
            @delete-tag="deleteTag"
            @toggle-pin="togglePin"
            @click="handleTagClick"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>