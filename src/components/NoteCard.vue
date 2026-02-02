<script setup>
import { computed, ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { MoreHorizontal, Edit, Trash2, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useNoteStore } from '@/store/noteStore'
import { revealFile } from '@/utils/fileUpload'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import ImageViewer from './ImageViewer.vue'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { common, createLowlight } from 'lowlight'
import { TagExtension } from '@/extensions/tag-extension'
import Image from '@tiptap/extension-image'
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'

dayjs.locale('zh-cn')

// 创建 lowlight 实例
const lowlight = createLowlight(common)

const props = defineProps({
  note: {
    type: Object,
    required: true
  },
  isPinned: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['click', 'edit', 'delete', 'expand', 'collapse'])

const noteStore = useNoteStore()

// 使用 useWorkDirectory composable
const { getWorkDirectory } = useWorkDirectory()

const menuVisible = ref(false)
const isExpanded = ref(false)
const contentRef = ref(null)
const isOverflowing = ref(false)
const MAX_HEIGHT = 120 // 最大高度，超过这个高度显示展开按钮

// 笔记类型判断
const noteType = computed(() => {
  // 优先使用 note_type 字段
  if (props.note.note_type) {
    return props.note.note_type
  }
  // 如果 note_type 不存在，使用 type 字段
  if (props.note.type) {
    return props.note.type
  }
  // 如果都没有，根据 sourceUrl 推断
  if (props.note.sourceUrl) {
    // 如果 sourceUrl 是 http/https 链接，则是 link 类型
    if (props.note.sourceUrl.startsWith('http://') || props.note.sourceUrl.startsWith('https://')) {
      return 'link'
    }
    // 否则是 file 类型
    return 'file'
  }
  // 默认返回 text
  return 'text'
})

const isLinkNote = computed(() => noteType.value === 'link')
const isFileNote = computed(() => noteType.value === 'file')

// 附件 URL
const attachmentUrl = ref('')

// 监听附件路径变化，更新 URL
watch(() => props.note.extractUrl, async (newExtractUrl) => {
  if (isFileNote.value && newExtractUrl) {
    attachmentUrl.value = newExtractUrl
  } else {
    attachmentUrl.value = ''
  }
}, { immediate: true })

// 提取文件名
const extractFileName = computed(() => {
  if (!props.note.extractUrl) return ''
  return props.note.extractUrl.split('/').pop()
})

// 简单的 HTML 内容渲染（使用 v-html）
const renderedContent = computed(() => {
  return props.note.content || ''
})

// 创建只读编辑器实例
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
      codeBlock: false,
    }),
    Highlight.configure({
      multicolor: true,
    }),
    CodeBlockLowlight.configure({
      lowlight,
      defaultLanguage: null,
    }),
    TagExtension,
    Image,
  ],
  editable: false,
  editorProps: {
    attributes: {
      class: 'prose prose-sm max-w-none text-[14px]',
    },
  },
})

// 监听内容变化，更新编辑器
watch([editor, () => props.note.content], ([newEditor, newContent]) => {
  if (newEditor && newContent && newContent !== newEditor.getHTML()) {
    newEditor.commands.setContent(newContent, false)
  }
}, { immediate: true })

// 格式化日期
const formattedDate = computed(() => {
  const date = dayjs(props.note.createdAt)
  return date.format('YYYY-MM-DD HH:mm:ss')
})

// 检查内容是否溢出
function checkOverflow() {
  if (!contentRef.value) return

  // 等待 DOM 完全渲染
  nextTick(() => {
    if (!contentRef.value) return

    const scrollHeight = contentRef.value.scrollHeight
    const clientHeight = MAX_HEIGHT

    // 如果 scrollHeight 为 0，说明元素还没有渲染，延迟后再检查
    if (scrollHeight === 0) {
      setTimeout(checkOverflow, 100)
      return
    }

    // 只有当内容高度超过 MAX_HEIGHT 时才认为溢出
    isOverflowing.value = scrollHeight > clientHeight
  })
}

// 切换展开/收起
function toggleExpand(event) {
  event.stopPropagation()
  if (isExpanded.value) {
    // 收起
    isExpanded.value = false
    emit('collapse', props.note.id)
  } else {
    // 展开
    isExpanded.value = true
    emit('expand', props.note.id)
  }
}

// 菜单项点击处理
function handleMenuClick(action) {
  menuVisible.value = false
  if (action === 'edit') {
    emit('edit', props.note)
  } else if (action === 'delete') {
    emit('delete', props.note)
  }
}

// 打开链接
async function openLink() {
  if (!props.note.sourceUrl) return
  try {
    await openUrl(props.note.sourceUrl)
  } catch (error) {
    // 静默处理错误，用户无感知
  }
}

// 在文件夹中显示文件
async function handleRevealFile() {
  if (!props.note.extractUrl) return
  try {
    const workDirectory = await noteStore.getWorkDirectory()
    await revealFile(props.note.extractUrl, workDirectory)
  } catch (error) {
    // 静默处理错误，用户无感知
  }
}

// 卡片点击
function handleCardClick() {
  emit('click', props.note)
}

// 点击外部关闭菜单
function handleClickOutside() {
  if (menuVisible.value) {
    menuVisible.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  // 组件挂载后检查溢出状态
  nextTick(() => {
    setTimeout(checkOverflow, 100)
    setTimeout(checkOverflow, 300)
  })
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
  // 销毁编辑器实例，避免内存泄漏
  if (editor.value) {
    editor.value.destroy()
  }
})

// 暴露方法给父组件
defineExpose({
  collapse: () => {
    if (isExpanded.value) {
      isExpanded.value = false
      emit('collapse', props.note.id)
    }
  }
})
</script>

<template>
  <div :data-note-id="note.id"
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 transition-all duration-200 hover:shadow-md"
    :class="{
      'link-card': isLinkNote,
      'expanded': isExpanded,
      'overflowing': isOverflowing
    }" @click="handleCardClick">
    <!-- 顶部：日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3 select-none">
      <div class="flex items-center gap-2">
        <span class="text-xs text-base-content/50">{{ formattedDate }}</span>
      </div>
      <div class="relative">
        <button @click.stop="menuVisible = !menuVisible"
          class="w-6 h-6 rounded flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-colors">
          <MoreHorizontal :size="20" />
        </button>

        <!-- 下拉菜单 -->
        <div v-if="menuVisible"
          class="absolute right-0 top-8 z-10 bg-base-100 border border-base-200 rounded-lg shadow-xl min-w-[120px] py-1"
          @click.stop>
          <button @click="handleMenuClick('edit')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors">
            <Edit :size="14" />
            编辑
          </button>
          <button @click="handleMenuClick('delete')"
            class="w-full px-3 py-2 text-left text-xs text-error hover:bg-base-200 flex items-center gap-2 transition-colors">
            <Trash2 :size="14" />
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 笔记内容：TipTap 编辑器渲染 -->
    <div v-if="note.content" class="relative">
      <div ref="contentRef" class="text-base-content leading-relaxed break-words transition-all duration-200" :class="{
        'max-h-[120px] overflow-hidden': !isExpanded && isOverflowing
      }">
        <EditorContent class="ProseMirror prose prose-sm max-w-none" :editor="editor" />
      </div>

      <!-- 展开/收起按钮 -->
      <div class="relative select-none">
        <!-- 渐变遮罩 -->
        <div v-if="!isExpanded && isOverflowing"
          class="absolute -top-8 left-0 right-0 h-8 bg-gradient-to-b from-transparent to-base-100 pointer-events-none">
        </div>
        <button v-if="isOverflowing" @click="toggleExpand" :class="[
          'mt-2 text-xs text-primary hover:text-primary/80 flex items-center gap-1',
          isPinned ? 'fixed bottom-4 left-1/2 -translate-x-1/2 z-50 bg-base-100 border border-base-200 shadow-lg px-4 py-2 rounded-lg' : ''
        ]">
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
    </div>

    <!-- 图片列表 -->
    <div v-if="note.images && note.images.length > 0"
      class="grid mn:grid-cols-5 xs:grid-cols-6 sm:grid-cols-7 sm:grid-cols-8 md:grid-cols-10 gap-2 mt-3">
      <ImageViewer v-for="(imageUrl, index) in note.images" :key="index" :src="imageUrl" :alt="`笔记图片 ${index + 1}`"
        :images="note.images" />
    </div>

    <!-- 底部信息 -->
    <div v-if="note.sourceUrl || note.extractUrl"
      class="mt-3 pt-2 border-t border-base-content/10 text-xs text-base-content/50">
      <span v-if="isLinkNote && note.sourceUrl"
        class="inline-flex items-center gap-1 max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap">
        来源：
        <a href="#" @click.prevent.stop="openLink"
          class="text-primary break-all hover:underline cursor-pointer overflow-hidden text-ellipsis whitespace-nowrap">
          {{ note.sourceUrl }}
        </a>
      </span>
      <span v-else-if="isFileNote && note.extractUrl" class="inline-flex items-center gap-1">
        附件：
        <a :href="attachmentUrl" @click.stop="handleRevealFile" target="_blank"
          class="text-primary hover:underline cursor-pointer">
          {{ extractFileName }}
        </a>
      </span>
    </div>
  </div>
</template>


<style scoped></style>