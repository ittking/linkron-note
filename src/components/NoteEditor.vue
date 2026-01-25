<script setup>
import { ref, computed, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Image from '@tiptap/extension-image'
import Underline from '@tiptap/extension-underline'
import Placeholder from '@tiptap/extension-placeholder'
import { 
  Hash, 
  Image as ImageIcon, 
  Type, 
  ListOrdered, 
  List, 
  AtSign, 
  Send 
} from 'lucide-vue-next'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  placeholder: {
    type: String,
    default: '现在的想法是...'
  },
  autofocus: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'submit', 'image-upload'])

const menuVisible = ref(false)

const editor = useEditor({
  content: props.modelValue,
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
    Underline,
    Placeholder.configure({
      placeholder: props.placeholder,
    }),
  ],
  autofocus: props.autofocus,
  editorProps: {
    attributes: {
      class: 'prose prose-sm max-w-none focus:outline-none min-h-[80px] py-2',
    },
  },
  onUpdate: ({ editor }) => {
    emit('update:modelValue', editor.getHTML())
  },
})

// 监听外部 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  if (editor.value && newValue !== editor.value.getHTML()) {
    editor.value.commands.setContent(newValue, false)
  }
})

// 计算是否有内容
const hasContent = computed(() => {
  return editor.value && editor.value.getText().trim().length > 0
})

// 工具栏操作
function toggleBold() {
  editor.value?.chain().focus().toggleBold().run()
}

function toggleItalic() {
  editor.value?.chain().focus().toggleItalic().run()
}

function toggleUnderline() {
  editor.value?.chain().focus().toggleUnderline().run()
}

function toggleBulletList() {
  editor.value?.chain().focus().toggleBulletList().run()
}

function toggleOrderedList() {
  editor.value?.chain().focus().toggleOrderedList().run()
}

function insertTag() {
  editor.value?.chain().focus().insertContent('# ').run()
}

function insertMention() {
  editor.value?.chain().focus().insertContent('@ ').run()
}

function handleImageUpload(event) {
  const file = event.target.files[0]
  if (file) {
    emit('image-upload', file)
  }
  // 重置 input
  event.target.value = ''
}

function handleSubmit() {
  if (hasContent.value) {
    emit('submit')
    editor.value?.commands.clearContent()
  }
}

// 键盘快捷键
function handleKeydown(event) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSubmit()
  }
}
</script>

<template>
  <div class="note-editor relative bg-base-100 border border-primary rounded-xl p-4 shadow-sm transition-all duration-200 focus-within:shadow-md focus-within:border-primary/80">
    <!-- 编辑器内容区域 -->
    <EditorContent 
      :editor="editor" 
      class="mb-3"
      @keydown="handleKeydown"
    />

    <!-- 底部工具栏 -->
    <div class="flex items-center justify-between">
      <!-- 左侧工具栏 -->
      <div class="flex items-center gap-3">
        <!-- 标签 # -->
        <button
          @click="insertTag"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入标签"
        >
          <Hash :size="18" />
        </button>

        <!-- 图片 -->
        <button
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入图片"
        >
          <ImageIcon :size="18" />
          <input
            type="file"
            accept="image/*"
            class="hidden"
            @change="handleImageUpload"
          />
        </button>

        <!-- 字体样式 Aa -->
        <button
          @click="toggleBold"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('bold') }"
          title="加粗"
        >
          <Type :size="18" />
        </button>

        <!-- 有序列表 -->
        <button
          @click="toggleOrderedList"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('orderedList') }"
          title="有序列表"
        >
          <ListOrdered :size="18" />
        </button>

        <!-- 无序列表 -->
        <button
          @click="toggleBulletList"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('bulletList') }"
          title="无序列表"
        >
          <List :size="18" />
        </button>

        <!-- 提及 @ -->
        <button
          @click="insertMention"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="提及用户"
        >
          <AtSign :size="18" />
        </button>
      </div>

      <!-- 右侧发送按钮 -->
      <button
        @click="handleSubmit"
        class="w-9 h-9 rounded-lg flex items-center justify-center transition-all duration-200"
        :class="[
          hasContent 
            ? 'bg-primary text-primary-content hover:bg-primary/90' 
            : 'bg-base-300 text-base-content/40 cursor-not-allowed'
        ]"
        :disabled="!hasContent"
        title="发送"
      >
        <Send :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Tiptap 编辑器样式 */
:deep(.ProseMirror) {
  outline: none;
}

:deep(.ProseMirror p.is-editor-empty:first-child::before) {
  color: #999;
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}

:deep(.ProseMirror ul),
:deep(.ProseMirror ol) {
  padding: 0 1rem;
  margin: 0.5rem 0;
}

:deep(.ProseMirror ul) {
  list-style-type: disc;
}

:deep(.ProseMirror ol) {
  list-style-type: decimal;
}

:deep(.ProseMirror li) {
  margin: 0.25rem 0;
}

:deep(.ProseMirror img) {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  margin: 0.5rem 0;
}

:deep(.ProseMirror strong) {
  font-weight: 600;
}

:deep(.ProseMirror u) {
  text-decoration: underline;
}
</style>