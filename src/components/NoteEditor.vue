<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { invoke } from '@tauri-apps/api/core'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import Placeholder from '@tiptap/extension-placeholder'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import Link from '@tiptap/extension-link'
import Image from '@tiptap/extension-image'
import { common, createLowlight } from 'lowlight'
import tippy from 'tippy.js'
import { useSettingStore } from '@/store/settingStore'
import { saveImage, deleteResource } from '@/utils/fileUpload'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import SelectionMenu from './SelectionMenu.vue'
import ImageViewer from './ImageViewer.vue'
import { TagMark, TagInputRuleExtension } from '@/extensions/tag-mark'
import { ResizableImage } from '@/extensions/resizable-image'
import { TagSuggestion } from '@/extensions/tag-suggestion'
import {
  Hash,
  Image as ImageIcon,
  ListOrdered,
  List,
  Send,
  Code,
  X
} from 'lucide-vue-next'

// 创建 lowlight 实例
const lowlight = createLowlight(common)

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
  },
  isScrolledToTop: {
    type: Boolean,
    default: true
  },
  isEditing: {
    type: Boolean,
    default: false
  },
  shouldClear: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'submit', 'image-upload'])

const settingStore = useSettingStore()
const imageInputRef = ref(null)
const isSettingContent = ref(false) // 标志：是否正在从外部设置内容
const isUnmounting = ref(false) // 标志：组件是否正在卸载

// 使用 useWorkDirectory composable（从 settingStore 获取）
const { getWorkDirectory } = useWorkDirectory('setting')

// 组件挂载时初始化编辑器内容
onMounted(() => {
  // 只在编辑模式下初始化内容
  if (props.isEditing && editor.value && props.modelValue) {
    // 设置内容，不触发更新事件，并标记为初始化
    editor.value.commands.setContent(props.modelValue, false)
    // 在下一个事件循环中标记初始化完成
    setTimeout(() => {
      if (editor.value) {
        editor.value.view.dispatch(
          editor.value.state.tr.setMeta('isInitializing', false)
        )
      }
    }, 0)
  }
})

// 组件卸载时销毁编辑器实例，避免内存泄漏
onBeforeUnmount(() => {
  isUnmounting.value = true
  // 安全销毁编辑器，避免在过渡动画中 DOM 已被移除时报错
  try {
    if (editor.value && !editor.value.isDestroyed) {
      editor.value.destroy()
    }
  } catch (error) {
    // 忽略销毁过程中的错误，通常发生在过渡动画中 DOM 已被移除
    // 静默处理，不打印警告
  }
})

const editor = useEditor({
  content: '', // 初始为空，在 onMounted 中设置
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
      codeBlock: false, // 禁用默认的 CodeBlock，使用 CodeBlockLowlight 代替
      link: false, // 排除 Link 扩展，因为我们要单独添加并配置它
    }),
    Highlight.configure({
      multicolor: true,
    }),
    Link.configure({
      openOnClick: false,
      HTMLAttributes: {
        class: 'text-primary hover:text-primary/80 underline',
      },
    }),
    CodeBlockLowlight.configure({
      lowlight,
      defaultLanguage: null,
    }),
    ResizableImage,
    Placeholder.configure({
      placeholder: props.placeholder,
    }),
    TagMark,
    TagInputRuleExtension,
    TagSuggestion,
  ],
  autofocus: props.autofocus,
  editorProps: {
    attributes: {
      class: 'prose prose-sm max-w-none focus:outline-none py-2 text-sm leading-loose',
    },
    getWorkDirectory: () => getWorkDirectory(),
    handleDragOver: (view, event) => {
      // 阻止默认的拖拽行为，防止 TipTap 自动处理拖拽
      event.preventDefault()
      return true
    },
    handleDrop: (view, event, slice, moved) => {
      // 阻止默认的 drop 行为，防止 TipTap 自动插入拖拽的内容
      // 所有拖拽操作都由父组件 Note.vue 的 handleDrop 统一处理
      event.preventDefault()
      // 不阻止事件冒泡，让父组件能够接收 drop 事件并重置拖拽状态
      return true
    },
    handlePaste: (view, event, slice) => {
      // 获取粘贴的数据
      const items = Array.from(event.clipboardData?.items || [])
      const hasImage = items.some(item => item.type.startsWith('image/'))

      // 如果粘贴的是图片
      if (hasImage) {
        event.preventDefault()

        // 查找图片文件
        const imageFile = items.find(item => item.type.startsWith('image/'))?.getAsFile()

        if (imageFile) {
          handleImageUpload({ target: { files: [imageFile] } })
        }
        return true
      }

      // 如果粘贴的是 HTML 内容，处理 h 标签
      const html = event.clipboardData?.getData('text/html')
      if (html) {
        event.preventDefault()

        // 创建临时 DOM 元素来解析 HTML
        const tempDiv = document.createElement('div')
        tempDiv.innerHTML = html

        // 将所有 h 标签替换为 p 标签，但保留文本内容
        const headings = tempDiv.querySelectorAll('h1, h2, h3, h4, h5, h6')
        headings.forEach(heading => {
          const text = heading.textContent
          // 使用 <strong> 标签来模拟标题的加粗效果
          heading.innerHTML = `<strong>${text}</strong>`
          heading.replaceWith(document.createElement('p').appendChild(heading.firstChild.cloneNode(true)))
        })

        // 获取处理后的纯文本（包含基本的加粗标记）
        const text = tempDiv.innerText

        // 插入文本内容
        view.dispatch(
          view.state.tr.insertText(text)
        )

        return true
      }

      // 如果粘贴的是纯文本，清除格式
      const text = event.clipboardData?.getData('text/plain')
      if (text) {
        event.preventDefault()
        view.dispatch(
          view.state.tr
            .insertText(text)
        )
        return true
      }

      return false
    },
  },
  onUpdate: ({ editor }) => {
    // 如果正在从外部设置内容，不触发 update:modelValue
    if (!isSettingContent.value) {
      emit('update:modelValue', editor.getHTML())
    }
  },
})

// 监听 shouldClear 标志，强制清空编辑器
watch(() => props.shouldClear, (shouldClear) => {
  if (shouldClear && editor.value) {
    editor.value.commands.clearContent()
  }
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  // 如果正在卸载，不处理
  if (isUnmounting.value) return

  // 如果编辑器已初始化且内容不同，则更新
  if (editor.value && !editor.value.isDestroyed && newValue && newValue !== editor.value.getHTML()) {
    isSettingContent.value = true
    // 设置内容，不触发更新事件
    editor.value.commands.setContent(newValue, false)
    // 在下一个事件循环中标记初始化完成
    setTimeout(() => {
      if (!isUnmounting.value) {
        isSettingContent.value = false
        if (editor.value) {
          editor.value.view.dispatch(
            editor.value.state.tr.setMeta('isInitializing', false)
          )
        }
      }
    }, 0)
  }
})

// 计算是否有内容
const hasContent = computed(() => {
  // 如果正在卸载或编辑器未初始化或已销毁，返回 false
  if (isUnmounting.value || !editor.value || editor.value.isDestroyed) return false

  try {
    // 检查是否有文本内容
    const hasText = editor.value.getText().trim().length > 0

    // 检查是否有图片
    let hasImage = false
    editor.value.state.doc.descendants((node) => {
      if (node.type.name === 'image') {
        hasImage = true
        return false // 找到图片后停止遍历
      }
    })

    // 有文本或有图片都可以提交
    return hasText || hasImage
  } catch {
    // 编辑器可能已被销毁，忽略错误
    return false
  }
})

function toggleBulletList() {
  editor.value?.chain().focus().toggleBulletList().run()
}

function toggleOrderedList() {
  editor.value?.chain().focus().toggleOrderedList().run()
}

// 插入标签 #
function insertTag() {
  editor.value?.chain().focus().insertContent('#').run()
}

// 插入代码块
function insertCodeBlock() {
  editor.value?.chain().focus().toggleCodeBlock().run()
}

// 触发图片上传
function triggerImageUpload() {
  imageInputRef.value?.click()
}

async function handleImageUpload(event) {
  const file = event.target.files[0]
  if (file) {
    try {
      // 使用抽离的工具方法保存图片
      const workDirectory = await getWorkDirectory()
      const imageUrl = await saveImage(file, workDirectory)
      // imageUrl 现在已经是完整 URL: http://linkron.localhost/resources/images/...

      // 插入到编辑器光标处
      if (editor.value) {
        editor.value.chain().focus().insertContent({
          type: 'resizableImage',
          attrs: {
            src: imageUrl,
            alt: file.name,
          }
        }).run()
      }
    } catch (error) {
      // 图片上传失败，静默处理
    }
  }
  // 重置 input
  event.target.value = ''
}

// 清空编辑器内容
function clearEditor() {
  if (editor.value) {
    editor.value.commands.clearContent()
  }
}

async function handleSubmit() {
  if (hasContent.value) {
    // 通过 emit 传递笔记数据（不包含 images）
    emit('submit', {
      content: editor.value.getHTML()
    })

    // 提交后清空编辑器
    editor.value?.commands.clearContent()
  }
}

// 添加图片到编辑器（从网页抓取）
function addImages(newImages) {
  if (newImages && newImages.length > 0) {
    // 将图片插入到编辑器中
    if (editor.value) {
      newImages.forEach((imageUrl) => {
        editor.value.chain().focus().insertContent({
          type: 'resizableImage',
          attrs: {
            src: imageUrl,
            alt: '图片',
          }
        }).run()
      })
    }
  }
}

// 暴露方法给父组件
defineExpose({
  addImages
})
</script>

<template>
  <div
    class="note-editor relative bg-base-100 border border-primary rounded-xl p-4 shadow-sm transition-all duration-200 focus-within:shadow-md focus-within:border-primary/80">
    <!-- 编辑器内容区域 -->
    <EditorContent class="transition-all duration-200 overflow-y-auto max-h-[400px] no-scrollbar" :editor="editor"
      :class="{
        'min-h-[80px]': props.isScrolledToTop,
        'min-h-[40px]': !props.isScrolledToTop
      }" />

    <!-- 底部工具栏 -->
    <div class="flex items-center justify-between mt-2">
      <!-- 左侧工具栏 -->
      <div class="flex items-center gap-3">
        <!-- 标签 # -->
        <button @click="insertTag"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入标签">
          <Hash :size="14" />
        </button>

        <!-- 图片 -->
        <button @click="triggerImageUpload"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入图片">
          <ImageIcon :size="14" />
          <input ref="imageInputRef" type="file" accept="image/*" class="hidden" @change="handleImageUpload" />
        </button>

        <!-- 有序列表 -->
        <button @click="toggleOrderedList"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('orderedList') }" title="有序列表">
          <ListOrdered :size="14" />
        </button>

        <!-- 无序列表 -->
        <button @click="toggleBulletList"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('bulletList') }" title="无序列表">
          <List :size="14" />
        </button>

        <!-- 代码块 -->
        <button @click="insertCodeBlock"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('codeBlock') }" title="代码块">
          <Code :size="14" />
        </button>
      </div>

      <!-- 右侧按钮区域 -->
      <div class="flex items-center gap-2">
        <!-- 插槽：用于放置取消按钮等自定义按钮 -->
        <slot name="actions"></slot>

        <!-- 清空按钮：只在新建笔记模式且有内容时显示 -->
        <button v-if="!props.isEditing && hasContent" @click="clearEditor"
          class="px-2 h-6 rounded-md flex items-center justify-center text-xs text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="清空内容">
          清空
        </button>

        <!-- 发送按钮 -->
        <button @click="handleSubmit"
          class="w-6 h-6 rounded-md flex items-center justify-center transition-all duration-200" :class="[
            hasContent
              ? 'bg-primary text-primary-content hover:bg-primary/90'
              : 'bg-base-300 text-base-content/40 cursor-not-allowed'
          ]" :disabled="!hasContent" :title="props.isEditing ? '保存' : '发送'">
          <Send :size="13" />
        </button>
      </div>
    </div>

    <!-- 悬浮工具栏 -->
    <SelectionMenu v-if="editor" :editor="editor" />
  </div>
</template>

<style scoped></style>
