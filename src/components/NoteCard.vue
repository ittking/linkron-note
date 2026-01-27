<script setup>
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { MoreHorizontal, Edit, Trash2, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { getResourceUrl } from '@/utils/fileUpload'
import { openUrl, revealItemInDir } from '@tauri-apps/plugin-opener'
import { invoke } from '@tauri-apps/api/core'
import { useNoteStore } from '@/store/noteStore'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import Image from '@tiptap/extension-image'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { common, createLowlight } from 'lowlight'
import { TagExtension } from '@/extensions/tag-extension'
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
watch(() => props.note.sourceUrl, async (newSourceUrl) => {
  if (isFileNote.value && newSourceUrl) {
    try {
      attachmentUrl.value = await getResourceUrl(newSourceUrl)
    } catch (error) {
      console.error('获取附件 URL 失败:', error)
      attachmentUrl.value = ''
    }
  } else {
    attachmentUrl.value = ''
  }
}, { immediate: true })

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

watch(editor, (newEditor) => {
  if (newEditor) {
    setTimeout(checkOverflow, 200)
  }
})

watch(() => props.note.content, (newValue) => {
  if (editor.value && newValue !== editor.value.getHTML()) {
    editor.value.commands.setContent(newValue, false)
    setTimeout(checkOverflow, 200)
  }
})

// 格式化日期
const formattedDate = computed(() => {
  const date = dayjs(props.note.createdAt)
  return date.format('YYYY-MM-DD HH:mm:ss')
})

// 检查内容是否溢出
function checkOverflow() {
  if (!contentRef.value || !editor.value) return

  const scrollHeight = contentRef.value.scrollHeight
  isOverflowing.value = scrollHeight > MAX_HEIGHT
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
    console.error('打开链接失败:', error)
  }
}

// 在文件夹中显示文件
async function revealFile() {
  if (!props.note.sourceUrl) return
  try {
    // 先将相对路径转换为协议 URL
    const protocolUrl = await getResourceUrl(props.note.sourceUrl)
    // 再将协议 URL 转换为本地文件路径
    const workDirectory = await noteStore.getWorkDirectory()
    const localPath = await invoke('get_local_path_from_protocol', {
      protocolUrl,
      workDirectory
    })
    // 使用本地路径打开文件夹
    await revealItemInDir(localPath)
  } catch (error) {
    console.error('显示文件失败:', error)
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

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div :data-note-id="note.id"
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 cursor-pointer transition-all duration-200 hover:shadow-md"
    :class="{
      'link-card': isLinkNote,
      'expanded': isExpanded,
      'overflowing': isOverflowing
    }" @click="handleCardClick">
    <!-- 顶部：日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3">
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

    <!-- 笔记内容：TipTap 编辑器渲染（所有类型统一） -->
    <div v-if="note.content">
      <div ref="contentRef" class="text-base-content leading-relaxed break-words" :class="{
        'line-clamp-5': !isExpanded && isOverflowing,
        'max-h-[120px] overflow-hidden': !isExpanded && isOverflowing
      }">
        <EditorContent class="ProseMirror" :editor="editor" />
      </div>

      <!-- 展开/收起按钮 -->
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

    <!-- 图片列表 -->
    <div v-if="note.images && note.images.length > 0"
      class="grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-2 mt-3">
      <div v-for="(imageUrl, index) in note.images" :key="index"
        class="relative aspect-square rounded-md overflow-hidden border border-base-200 bg-base-200">
        <img :src="imageUrl" class="w-full h-full object-cover" alt="笔记图片" loading="lazy" />
      </div>
    </div>

    <!-- 底部信息 -->
    <div v-if="note.sourceUrl" class="mt-3 pt-2 border-t border-base-content/10 text-xs text-base-content/50">
      <span v-if="isLinkNote"
        class="inline-flex items-center gap-1 max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap">
        来源：
        <a href="#" @click.prevent.stop="openLink"
          class="text-primary break-all hover:underline cursor-pointer overflow-hidden text-ellipsis whitespace-nowrap">
          {{ note.sourceUrl }}
        </a>
      </span>
      <span v-else-if="isFileNote" class="inline-flex items-center gap-1">
        附件：
        <a :href="attachmentUrl" @click.stop="revealFile" target="_blank"
          class="text-primary hover:underline cursor-pointer">
          {{ note.sourceUrl.split('/').pop() }}
        </a>
      </span>
    </div>
  </div>
</template>


<style scoped></style>