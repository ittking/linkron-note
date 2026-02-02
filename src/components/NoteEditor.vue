<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { invoke } from '@tauri-apps/api/core'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import Placeholder from '@tiptap/extension-placeholder'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import Link from '@tiptap/extension-link'
import { common, createLowlight } from 'lowlight'
import { TagExtension } from '@/extensions/tag-extension'
import tippy from 'tippy.js'
import { useSettingStore } from '@/store/settingStore'
import { saveImage, deleteResource } from '@/utils/fileUpload'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import SelectionMenu from './SelectionMenu.vue'
import ImageViewer from './ImageViewer.vue'
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
  images: {
    type: Array,
    default: () => []
  },
  shouldClear: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'submit', 'image-upload'])

const settingStore = useSettingStore()
const imageInputRef = ref(null)
const images = ref([])
const deletedImages = ref([]) // 追踪编辑模式下被删除的图片
const isSettingContent = ref(false) // 标志：是否正在从外部设置内容
const isUnmounting = ref(false) // 标志：组件是否正在卸载

// 使用 useWorkDirectory composable（从 settingStore 获取）
const { getWorkDirectory } = useWorkDirectory('setting')

// 监听 props.images 变化，同步到本地状态
watch(() => props.images, (newImages) => {
  // 只在编辑模式下才同步外部 images 变化
  // 避免在新建笔记模式下被意外重置
  if (!props.isEditing || !newImages) return

  // 只比较引用，如果引用变化则更新
  if (newImages !== images.value) {
    images.value = [...newImages]
  }
})

// 组件挂载时初始化编辑器内容
onMounted(() => {
  // 只在编辑模式下初始化内容
  if (props.isEditing && editor.value && props.modelValue) {
    editor.value.commands.setContent(props.modelValue, false)
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
    Placeholder.configure({
      placeholder: props.placeholder,
    }),
    TagExtension.configure({
      suggestion: {
        items: async ({ query }) => {
          // 从后端获取标签建议
          try {
            const workDirectory = await getWorkDirectory()
            const tags = await invoke('search_tags', { keyword: query, workDirectory })
            return tags.map(tag => ({
              id: tag.id,
              name: tag.name,
              displayName: tag.displayName,
              path: tag.path,
              level: tag.level,
            }))
          } catch (error) {
            return []
          }
        },
        render: () => {
          let component
          let popup
          let selectedIndex = 0
          let isDestroyed = false
          let items = []
          let currentProps = null
          let currentQuery = ''

          function renderItems() {
            component.innerHTML = ''
            if (items && items.length > 0) {
              items.forEach((item, index) => {
                const itemEl = document.createElement('div')
                const isSelected = index === selectedIndex
                itemEl.className = `tag-suggestion-item flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${isSelected ? 'bg-primary/20 text-primary' : ''}`
                itemEl.textContent = '#' + item.name
                itemEl.dataset.index = index
                itemEl.addEventListener('click', () => {
                  // 替换当前正在输入的标签名称
                  const editor = currentProps.editor
                  const { from } = currentProps.range

                  // 先聚焦编辑器
                  editor.view.focus()

                  // 删除当前标签输入（从 # 开始）
                  const tr = editor.view.state.tr.delete(from, currentProps.range.to)

                  // 插入选中的标签名称
                  tr.insertText('#' + item.name, from)

                  // 设置光标位置到插入的文本后面
                  const newTo = from + item.name.length + 1
                  tr.setSelection(editor.view.state.tr.selection.constructor.near(tr.doc.resolve(newTo)))

                  // 应用 transaction
                  editor.view.dispatch(tr)

                  // 更新 range，让建议继续工作
                  currentProps.range = { from, to: newTo }

                  // 重置选中索引
                  selectedIndex = 0

                  // 重新渲染
                  renderItems()
                })
                component.appendChild(itemEl)
              })
            } else {
              if (popup) popup.hide()
            }
          }

          return {
            onStart: async (props) => {
              currentProps = props
              currentQuery = props.query || ''
              isDestroyed = false
              component = document.createElement('div')
              component.className = 'bg-base-100 border border-base-200 rounded-lg shadow-xl max-h-60 overflow-y-auto no-scrollbar p-2'

              // 将标签建议框添加到 body 中
              const appendTarget = document.body

              // 使用正确的虚拟定位方式
              popup = tippy(document.body, {
                getReferenceClientRect: props.clientRect,
                appendTo: appendTarget,
                content: component,
                showOnCreate: true,
                interactive: true,
                trigger: 'manual',
                placement: 'bottom-start',
              })

              // 渲染标签列表
              items = await props.items
              renderItems()
            },
            onUpdate: async (props) => {
              currentProps = props
              currentQuery = props.query || ''
              if (popup && popup.setProps) {
                popup.setProps({
                  getReferenceClientRect: props.clientRect,
                })
              }
              selectedIndex = 0

              // 重新渲染标签列表
              items = await props.items

              // 如果有匹配的标签，确保显示虚拟列表
              if (items && items.length > 0 && popup) {
                popup.show()
              }

              renderItems()
            },
            onKeyDown: (props) => {
              if (props.event.key === 'Escape') {
                if (popup && !isDestroyed) popup.hide()
                return true
              }
              if (props.event.key === 'ArrowDown') {
                if (items && items.length > 0) {
                  selectedIndex = (selectedIndex + 1) % items.length
                  renderItems()
                }
                return true
              }
              if (props.event.key === 'ArrowUp') {
                if (items && items.length > 0) {
                  selectedIndex = (selectedIndex - 1 + items.length) % items.length
                  renderItems()
                }
                return true
              }
              if (props.event.key === 'Enter') {
                // 回车键：选中当前标签并替换当前输入
                if (items && items[selectedIndex]) {
                  // 替换当前正在输入的标签名称
                  const selectedTag = items[selectedIndex]
                  const editor = currentProps.editor
                  const { from } = currentProps.range

                  // 先聚焦编辑器
                  editor.view.focus()

                  // 删除当前标签输入（从 # 开始）
                  const tr = editor.view.state.tr.delete(from, currentProps.range.to)

                  // 插入选中的标签名称
                  tr.insertText('#' + selectedTag.name, from)

                  // 设置光标位置到插入的文本后面
                  const newTo = from + selectedTag.name.length + 1
                  tr.setSelection(editor.view.state.tr.selection.constructor.near(tr.doc.resolve(newTo)))

                  // 应用 transaction
                  editor.view.dispatch(tr)

                  // 更新 range，让建议继续工作
                  currentProps.range = { from, to: newTo }

                  // 重置选中索引
                  selectedIndex = 0

                  // 重新渲染
                  renderItems()
                  return true
                }
                return false
              }
              if (props.event.key === ' ') {
                // 空格键：创建当前输入的标签
                if (currentQuery.length > 0) {
                  currentProps.command({
                    id: Date.now().toString(),
                    name: currentQuery,
                    displayName: currentQuery,
                    path: '',
                    level: currentQuery.split('/').length,
                  })
                  return true
                }
                return false
              }
              return false
            },
            onExit: () => {
              if (isDestroyed) return
              isDestroyed = true
              if (popup) {
                try {
                  popup.destroy()
                } catch (e) {
                  // 忽略已销毁的错误
                }
              }
              if (component) {
                try {
                  component.remove()
                } catch (e) {
                  // 忽略移除错误
                }
              }
            },
          }
        },
      },
    }),
  ],
  autofocus: props.autofocus,
  editorProps: {
    attributes: {
      class: 'prose prose-sm max-w-none focus:outline-none py-2 text-sm',
    },
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

// 监听 shouldClear 标志，强制清空编辑器和图片
watch(() => props.shouldClear, (shouldClear) => {
  if (shouldClear && editor.value) {
    editor.value.commands.clearContent()
    images.value = []
  }
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  // 如果正在卸载，不处理
  if (isUnmounting.value) return

  // 如果编辑器已初始化且内容不同，则更新
  if (editor.value && !editor.value.isDestroyed && newValue && newValue !== editor.value.getHTML()) {
    isSettingContent.value = true
    editor.value.commands.setContent(newValue, false)
    // 稍后重置标志
    setTimeout(() => {
      if (!isUnmounting.value) {
        isSettingContent.value = false
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
      // imageUrl 现在已经是完整 URL: http://iterm.localhost/resources/images/...

      // 添加到图片列表（不再插入到编辑器）
      images.value.push(imageUrl)
    } catch (error) {
      // 图片上传失败，静默处理
    }
  }
  // 重置 input
  event.target.value = ''
}

// 删除图片
async function removeImage(index) {
  const removedImage = images.value[index]

  // 从当前图片列表中移除
  images.value.splice(index, 1)

  // 根据模式处理文件删除
  if (props.isEditing) {
    // 编辑模式：记录被删除的图片，在保存时统一删除
    deletedImages.value.push(removedImage)
  } else {
    // 创建模式：立即删除文件
    try {
      const workDirectory = await getWorkDirectory()
      await deleteResource(removedImage, workDirectory)
    } catch (error) {
      // 删除图片文件失败，静默处理
    }
  }
}
// 清理函数：取消编辑时调用，清空被删除的图片列表
function clearDeletedImages() {
  deletedImages.value = []
}

// 清空编辑器内容和图片
function clearEditor() {
  if (editor.value) {
    editor.value.commands.clearContent()
    images.value = []
  }
}

async function handleSubmit() {
  if (hasContent.value || images.value.length > 0) {
    // 如果是编辑模式，先删除被移除的图片文件
    if (props.isEditing && deletedImages.value.length > 0) {
      try {
        const workDirectory = await getWorkDirectory()
        for (const imageUrl of deletedImages.value) {
          await deleteResource(imageUrl, workDirectory)
        }
        deletedImages.value = [] // 清空已删除列表
      } catch (error) {
        // 删除图片文件失败，静默处理
      }
    }

    // 通过 emit 传递完整的笔记数据
    emit('submit', {
      content: editor.value.getHTML(),
      images: images.value
    })

    // 无论编辑模式还是创建模式，提交后都清空编辑器和图片
    editor.value?.commands.clearContent()
    images.value = []
    deletedImages.value = []
  }
}

// 添加图片到图片列表
function addImages(newImages) {
  if (newImages && newImages.length > 0) {
    images.value = [...images.value, ...newImages]
  }
}

// 暴露方法给父组件
defineExpose({
  clearDeletedImages,
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

    <!-- 图片列表 -->
    <div v-if="images.length > 0" class="mt-2 max-h-22 overflow-y-auto no-scrollbar">
      <div class="flex flex-wrap gap-2">
        <div v-for="(imageUrl, index) in images" :key="index" class="relative">
          <ImageViewer :src="imageUrl" :alt="`上传的图片 ${index + 1}`" :images="images" aspectRatio="square"
            className="w-12 h-12" />
          <button @click="removeImage(index)"
            class="absolute top-1 right-1 w-4 h-4 bg-black/50 hover:bg-black/70 rounded-full flex items-center justify-center text-white transition-all duration-200 -mt-1 -mr-1"
            title="删除图片">
            <X size="10" />
          </button>
        </div>
      </div>
    </div>

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
        <button v-if="!props.isEditing && (hasContent || images.length > 0)" @click="clearEditor"
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
