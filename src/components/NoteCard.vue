<script setup>
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { MoreHorizontal, ExternalLink, Edit, Trash2, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { invoke } from '@tauri-apps/api/core'
import StarterKit from '@tiptap/starter-kit'
import Image from '@tiptap/extension-image'
import Highlight from '@tiptap/extension-highlight'
import { TagExtension } from '@/extensions/tag-extension'
import { useSettingStore } from '@/store/settingStore'
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'

dayjs.locale('zh-cn')

const props = defineProps({
  note: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['click', 'open', 'edit', 'delete', 'tag-click'])

const settingStore = useSettingStore()
const menuVisible = ref(false)
const isExpanded = ref(false)
const contentRef = ref(null)
const isOverflowing = ref(false)
const MAX_HEIGHT = 120 // 最大高度，超过这个高度显示展开按钮

// 从内容中提取图片列表
const extractedImages = computed(() => {
  if (!props.note.content) return []
  const imgRegex = /<img[^>]*src=["']([^"']+)["'][^>]*>/gi
  const images = []
  let match
  while ((match = imgRegex.exec(props.note.content)) !== null) {
    // 解码 HTML 实体（如 &amp; -> &）
    images.push(decodeHtmlEntities(match[1]))
  }
  return images
})

// 解析图片路径（异步）
async function resolveImagePath(src) {
  // 如果是相对路径（以 resources/ 开头），使用 iterm:// 协议
  if (src.startsWith('resources/')) {
    try {
      const resourceUrl = await invoke('get_resource_url', {
        relativePath: src
      })
      return resourceUrl
    } catch (error) {
      console.error('Failed to resolve image path:', error)
      return src
    }
  }
  // 如果是 file:// 协议或其他格式，直接返回
  return src
}

// 解析后的图片路径列表
const resolvedImages = ref([])

// 监听 extractedImages 变化，解析图片路径
watch(extractedImages, async (newImages) => {
  resolvedImages.value = await Promise.all(newImages.map(resolveImagePath))
}, { immediate: true })

// 创建只读编辑器实例来渲染内容
const editor = useEditor({
  content: props.note.content || '',
  extensions: [
    StarterKit.configure({
      bulletList: {
        keepMarks: true,
        keepAttributes: false,
      },
      orderedList: {
        keepMarks: true,
        keepAttributes: false,
      },
    }),
    Image.configure({
      inline: true,
      allowBase64: true,
    }),
    Highlight.configure({
      multicolor: true,
    }),
    TagExtension, // 添加 TagExtension 以正确渲染标签
  ],
  editable: false, // 只读模式
  editorProps: {
    attributes: {
      class: 'prose prose-sm max-w-none text-[14px]',
    },
  },
})

// 监听 content 变化，更新编辑器内容
watch(() => props.note.content, (newValue) => {
  if (editor.value && newValue !== editor.value.getHTML()) {
    editor.value.commands.setContent(newValue, false)
    // 内容更新后重新检查溢出
    checkOverflow()
  }
})

// 监听编辑器创建，创建后检查溢出
watch(editor, (newEditor) => {
  if (newEditor) {
    checkOverflow()
  }
})

// 格式化日期 - 精确到秒，不包含星期
const formattedDate = computed(() => {
  const date = dayjs(props.note.createdAt)
  return date.format('YYYY-MM-DD HH:mm:ss')
})

// 解码 HTML 实体
function decodeHtmlEntities(str) {
  const textarea = document.createElement('textarea')
  textarea.innerHTML = str
  return textarea.value
}

// 从内容中提取标签列表
const extractedTags = computed(() => {
  if (!props.note.content) return []
  const tagRegex = /<span data-type="tag"[^>]*data-name="([^"]+)"[^>]*data-id="([^"]+)"[^>]*>(?:<[^>]*>)*([^<]+)(?:<[^>]*>)*<\/span>/g
  const tags = []
  let match
  while ((match = tagRegex.exec(props.note.content)) !== null) {
    tags.push({
      name: match[1],
      id: match[2],
      displayName: match[3],
    })
  }
  return tags
})

// 处理标签点击
function handleTagClick(tag) {
  emit('tag-click', tag)
}

// 检查内容是否溢出
function checkOverflow() {
  if (contentRef.value) {
    // 使用 requestAnimationFrame 确保 DOM 已渲染
    requestAnimationFrame(() => {
      if (contentRef.value) {
        // 临时移除高度限制来测量实际内容高度
        const originalClasses = contentRef.value.className
        contentRef.value.style.maxHeight = 'none'
        contentRef.value.style.overflow = 'visible'
        
        const scrollHeight = contentRef.value.scrollHeight
        
        // 恢复原始样式
        contentRef.value.style.maxHeight = ''
        contentRef.value.style.overflow = ''
        
        // 只有当实际内容高度超过最大高度时才显示展开按钮
        isOverflowing.value = scrollHeight > MAX_HEIGHT
      }
    })
  }
}

// 切换展开/收起
function toggleExpand(event) {
  event.stopPropagation()
  isExpanded.value = !isExpanded.value
  // 展开状态改变后重新检查溢出
  checkOverflow()
}

// 菜单项点击处理
function handleMenuClick(action) {
  menuVisible.value = false
  if (action === 'open') {
    emit('open', props.note)
  } else if (action === 'edit') {
    emit('edit', props.note)
  } else if (action === 'delete') {
    emit('delete', props.note)
  }
}

// 卡片点击
function handleCardClick() {
  emit('click', props.note)
}

// 点击外部关闭菜单
function handleClickOutside(event) {
  if (menuVisible.value) {
    menuVisible.value = false
  }
}

// 生命周期钩子
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  checkOverflow()
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 cursor-pointer transition-all duration-200 hover:shadow-md"
    @click="handleCardClick"
  >
    <!-- 顶部：日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs text-base-content/50">{{ formattedDate }}</span>
      <div class="relative">
        <button
          @click.stop="menuVisible = !menuVisible"
          class="w-6 h-6 rounded flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-colors"
        >
          <MoreHorizontal :size="20" />
        </button>
        
        <!-- 下拉菜单 -->
        <div
          v-if="menuVisible"
          class="absolute right-0 top-8 z-10 bg-base-100 border border-base-200 rounded-lg shadow-xl min-w-[120px] py-1"
          @click.stop
        >
          <button
            v-if="note.sourceUrl"
            @click="handleMenuClick('open')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <ExternalLink :size="14" />
            打开链接
          </button>
          <button
            @click="handleMenuClick('edit')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <Edit :size="14" />
            编辑
          </button>
          <button
            @click="handleMenuClick('delete')"
            class="w-full px-3 py-2 text-left text-xs text-error hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <Trash2 :size="14" />
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 内容 -->
    <div v-if="note.content">
      <div
        ref="contentRef"
        class="text-base-content leading-relaxed break-words"
        :class="{
          'line-clamp-5': !isExpanded && isOverflowing,
          'max-h-[120px] overflow-hidden': !isExpanded && isOverflowing
        }"
      >
        <EditorContent :editor="editor" />
      </div>
      
      <!-- 展开/收起按钮 -->
      <button
        v-if="isOverflowing"
        @click="toggleExpand"
        class="mt-2 text-xs text-primary hover:text-primary/80 flex items-center gap-1 transition-colors"
      >
        <template v-if="!isExpanded">
          展开全文
          <ChevronDown :size="14" />
        </template>
        <template v-else>
          收起
          <ChevronUp :size="14" />
        </template>
      </button>
    </div>

    <!-- 图片列表 -->
    <div v-if="resolvedImages.length > 0 && !isExpanded" class="flex flex-wrap gap-2 mt-3">
      <img
        v-for="(img, index) in resolvedImages"
        :key="index"
        :src="img"
        class="w-20 h-20 rounded-lg object-cover border border-base-200 cursor-pointer hover:opacity-80 transition-opacity"
        alt="Note image"
        @click.stop
      />
    </div>

    <!-- 标签列表 -->
    <div v-if="extractedTags.length > 0" class="flex flex-wrap gap-2 mt-3">
      <span
        v-for="tag in extractedTags"
        :key="tag.id"
        @click.stop="handleTagClick(tag)"
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs font-medium cursor-pointer hover:bg-primary/20 transition-colors"
      >
        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M17.707 9.293a1 1 0 010 1.414l-7 7a1 1 0 01-1.414 0l-7-7A.997.997 0 012 10V5a3 3 0 013-3h5c.256 0 .512.098.707.293l7 7zM5 6a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"/>
        </svg>
        {{ tag.displayName }}
      </span>
    </div>
  </div>
</template>