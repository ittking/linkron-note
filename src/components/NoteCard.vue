<script setup>
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { MoreHorizontal, ExternalLink, Edit, Trash2, ChevronDown, ChevronUp, Image as ImageIcon, FileText, Link as LinkIcon, File } from 'lucide-vue-next'
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
  }
})

const emit = defineEmits(['click', 'open', 'edit', 'delete', 'tag-click'])

const noteStore = useNoteStore()

const menuVisible = ref(false)
const isExpanded = ref(false)
const contentRef = ref(null)
const isOverflowing = ref(false)
const MAX_HEIGHT = 120 // 最大高度，超过这个高度显示展开按钮

// 笔记类型判断
const noteType = computed(() => props.note.note_type || 'text')
const isImageNote = computed(() => noteType.value === 'image')
const isTextNote = computed(() => noteType.value === 'text')
const isLinkNote = computed(() => noteType.value === 'link')

// 图片笔记：从 note.images 数组获取
const imageUrls = ref([])

// 图文笔记：从内容中提取图片
const extractedImages = computed(() => {
  if (!props.note.content || isImageNote.value) return []
  const imgRegex = /<img[^>]*src=["']([^"']+)["'][^>]*>/gi
  const images = []
  let match
  while ((match = imgRegex.exec(props.note.content)) !== null) {
    images.push(decodeHtmlEntities(match[1]))
  }
  return images
})

// 所有需要显示的图片（根据笔记类型）
const displayImages = computed(() => {
  if (isImageNote.value) {
    return imageUrls.value
  } else {
    return extractedImages.value
  }
})

// 是否有图片
const hasImages = computed(() => {
  return displayImages.value.length > 0
})

// 是否有附件（文件）
const hasAttachments = computed(() => {
  return props.note.images && props.note.images.length > 0
})

// 获取文件名
const getFileName = (filePath) => {
  if (!filePath) return '未知文件'
  const parts = filePath.split(/[/\\]/)
  return parts[parts.length - 1] || filePath
}

// 解析图片路径
async function resolveImagePath(src) {
  if (src.startsWith('resources/')) {
    try {
      const resourceUrl = await getResourceUrl(src)
      return resourceUrl
    } catch (error) {
      console.error('Failed to resolve image path:', error)
      return src
    }
  }
  return src
}

// 监听图片变化，解析路径
watch(() => props.note.images, async (newImages) => {
  if (isImageNote.value && newImages) {
    imageUrls.value = await Promise.all(newImages.map(resolveImagePath))
  }
}, { immediate: true })

// 创建只读编辑器实例（仅用于图文笔记）
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

// 解码 HTML 实体
function decodeHtmlEntities(str) {
  const textarea = document.createElement('textarea')
  textarea.innerHTML = str
  return textarea.value
}

// 提取标签
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

function handleTagClick(tag) {
  emit('tag-click', tag)
}

// 检查内容是否溢出
function checkOverflow() {
  if (!contentRef.value || !editor.value || isImageNote.value) return

  const scrollHeight = contentRef.value.scrollHeight
  isOverflowing.value = scrollHeight > MAX_HEIGHT
}

// 切换展开/收起
function toggleExpand(event) {
  event.stopPropagation()
  isExpanded.value = !isExpanded.value
}

// 菜单项点击处理
function handleMenuClick(action) {
  menuVisible.value = false
  if (action === 'open') {
    openLink()
  } else if (action === 'edit') {
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

// 卡片点击
function handleCardClick() {
  emit('click', props.note)
}

// 打开附件
async function openAttachment() {
  if (!hasAttachments.value || !props.note.images || props.note.images.length === 0) return
  
  const filePath = props.note.images[0]
  try {
    // 获取工作目录
    const workDirectory = await noteStore.getWorkDirectory()
    // 将协议 URL 转换为本地文件路径
    const resourceUrl = await getResourceUrl(filePath)
    const localPath = await invoke('get_local_path_from_protocol', { 
      protocolUrl: resourceUrl,
      workDirectory: workDirectory
    })
    // 使用 revealItemInDir 在文件资源管理器中显示文件
    await revealItemInDir(localPath)
  } catch (error) {
    console.error('打开附件失败:', error)
  }
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
  <div
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 cursor-pointer transition-all duration-200 hover:shadow-md"
    :class="{ 'image-card': isImageNote, 'link-card': isLinkNote }"
    @click="handleCardClick">
    <!-- 顶部：类型图标 + 日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <!-- 类型图标 -->
        <component 
          :is="isImageNote ? ImageIcon : isLinkNote ? LinkIcon : FileText" 
          :size="16" 
          class="text-base-content/50"
        />
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
          <button v-if="note.sourceUrl" @click="handleMenuClick('open')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors">
            <ExternalLink :size="14" />
            打开链接
          </button>
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

    <!-- 图片笔记：网格缩略图展示 -->
    <div v-if="isImageNote && hasImages" class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2 mb-3">
      <img 
        v-for="(img, index) in displayImages" 
        :key="index" 
        :src="img"
        class="w-full h-[120px] object-cover rounded-lg cursor-pointer hover:opacity-80 transition-opacity"
        alt="Note image"
        @click.stop
      />
    </div>

    <!-- 图文笔记：TipTap 编辑器渲染 -->
    <div v-else-if="isTextNote && note.content">
      <div
        ref="contentRef"
        class="text-base-content leading-relaxed break-words"
        :class="{
          'line-clamp-5': !isExpanded && isOverflowing,
          'max-h-[120px] overflow-hidden': !isExpanded && isOverflowing
        }">
        <EditorContent class="ProseMirror" :editor="editor" />
      </div>

      <!-- 展开/收起按钮 -->
      <button v-if="isOverflowing" @click="toggleExpand"
        class="mt-2 text-xs text-primary hover:text-primary/80 flex items-center gap-1">
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

    <!-- 链接笔记：链接预览卡片 -->
    <div v-else-if="isLinkNote" class="p-3 bg-base-200 rounded-lg">
      <p v-if="note.content" class="text-sm text-base-content/70 mb-3 leading-relaxed">{{ note.content }}</p>
      <a v-if="note.sourceUrl" :href="note.sourceUrl" target="_blank" class="text-xs text-primary break-all hover:underline" @click.stop>
        {{ note.sourceUrl }}
      </a>
    </div>

    <!-- 底部信息 -->
    <div v-if="note.sourceUrl || (hasAttachments && !isImageNote) || extractedTags.length > 0" class="mt-3 pt-2 border-t border-base-content/10 text-xs text-base-content/50 flex flex-wrap gap-3 items-center">
      <span v-if="note.sourceUrl" class="inline-flex items-center gap-1 max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap">
        来源：
        <a href="#" @click.prevent.stop="openLink" class="text-primary break-all hover:underline cursor-pointer overflow-hidden text-ellipsis whitespace-nowrap">
          {{ note.sourceUrl }}
        </a>
      </span>
      <span v-if="hasAttachments && !isImageNote" class="inline-flex items-center gap-1 max-w-[200px] overflow-hidden cursor-pointer hover:text-primary transition-colors" @click="openAttachment">
        <File :size="12" />
        <span class="overflow-hidden text-ellipsis whitespace-nowrap">附件：{{ getFileName(note.images[0]) }}</span>
      </span>      <span v-if="extractedTags.length > 0" class="flex flex-wrap gap-1.5 ml-auto">
        <span v-for="tag in extractedTags" :key="tag.id" @click.stop="handleTagClick(tag)"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs font-medium cursor-pointer hover:bg-primary/20 transition-colors">
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd"
              d="M17.707 9.293a1 1 0 010 1.414l-7 7a1 1 0 01-1.414 0l-7-7A.997.997 0 012 10V5a3 3 0 013-3h5c.256 0.512.098.707.293l7 7zM5 6a1 1 0 100-2 1 1 0 000 2z"
              clip-rule="evenodd" />
          </svg>
          {{ tag.displayName }}
        </span>
      </span>
    </div>
  </div>
</template>


<style scoped>
</style>