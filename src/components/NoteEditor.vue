<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { invoke } from '@tauri-apps/api/core'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import Placeholder from '@tiptap/extension-placeholder'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { common, createLowlight } from 'lowlight'
import { TagExtension } from '@/extensions/tag-extension'
import tippy from 'tippy.js'
import { useSettingStore } from '@/store/settingStore'
import { saveImage, getResourceUrl } from '@/utils/fileUpload'
import SelectionMenu from './SelectionMenu.vue'
import {
  Hash,
  Image as ImageIcon,
  ListOrdered,
  List,
  Send,
  Code
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

// 监听 props.images 变化，同步到本地状态
watch(() => props.images, (newImages) => {
  // 只在编辑模式下才同步外部 images 变化
  // 避免在新建笔记模式下被意外重置
  if (props.isEditing && newImages) {
    // 比较新旧数组，避免不必要的更新
    const currentImagesStr = JSON.stringify(images.value)
    const newImagesStr = JSON.stringify(newImages)
    if (currentImagesStr !== newImagesStr) {
      images.value = [...newImages]
    }
  }
}, { deep: true })

// 获取工作目录
async function getWorkDirectory() {
  return await settingStore.get('workDirectory', '')
}

// 组件挂载时初始化编辑器内容
onMounted(() => {
  // 只在编辑模式下初始化内容
  if (props.isEditing && editor.value && props.modelValue) {
    editor.value.commands.setContent(props.modelValue, false)
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
    }),
    Highlight.configure({
      multicolor: true,
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

              // 获取 #iterm-panel 元素，确保标签建议框在命名空间内
              const itermPanel = document.getElementById('iterm-panel')
              const appendTarget = itermPanel || document.body

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
    emit('update:modelValue', editor.getHTML())
  },
})

// 移除了对 props.modelValue 的 watch，避免用户输入时频繁更新编辑器导致状态丢失
// 编辑器内容只在初始化时设置一次，后续用户输入不会反向同步

// 监听 shouldClear 标志，强制清空编辑器和图片
watch(() => props.shouldClear, (shouldClear) => {
  if (shouldClear && editor.value) {
    editor.value.commands.clearContent()
    images.value = []
  }
})

// 监听 modelValue 变化（仅用于编辑模式初始化内容）
watch(() => props.modelValue, (newValue) => {
  // 只在编辑模式下，且编辑器已初始化，且内容真正不同时才更新
  if (props.isEditing && editor.value && newValue && newValue !== editor.value.getHTML()) {
    editor.value.commands.setContent(newValue, false)
  }
})

// 计算是否有内容
const hasContent = computed(() => {
  if (!editor.value) return false
  
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
})

// 工具栏操作
function toggleHighlight() {
  editor.value?.chain().focus().toggleHighlight().run()
}

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
      // imageUrl 现在已经是完整 URL: Windows: http://iterm.localhost/resources/images/..., 其他平台: iterms://resources/images/...

      // 添加到图片列表（不再插入到编辑器）
      images.value.push(imageUrl)
    } catch (error) {
      console.error('图片上传失败:', error)
    }
  }
  // 重置 input
  event.target.value = ''
}

// 删除图片
function removeImage(index) {
  images.value.splice(index, 1)
}

function handleSubmit() {
  if (hasContent.value || images.value.length > 0) {
    // 通过 emit 传递完整的笔记数据
    emit('submit', {
      content: editor.value.getHTML(),
      images: images.value
    })

    // 无论编辑模式还是创建模式，提交后都清空编辑器和图片
    editor.value?.commands.clearContent()
    images.value = []
  }
}
</script>

<template>
  <div
    class="note-editor relative bg-base-100 border border-primary rounded-xl p-4 shadow-sm transition-all duration-200 focus-within:shadow-md focus-within:border-primary/80">
    <!-- 编辑器内容区域 -->
    <EditorContent class="transition-all duration-200 overflow-y-auto max-h-[400px] no-scrollbar"
      :editor="editor" :class="{
        'min-h-[80px]': props.isScrolledToTop,
        'min-h-[40px]': !props.isScrolledToTop
      }" />

      <!-- 图片列表 -->
      <div v-if="images.length > 0" class="mt-2">
        <div class="flex flex-wrap gap-2">
          <div
            v-for="(imageUrl, index) in images"
            :key="index"
            class="relative w-10 h-10 rounded overflow-hidden border border-base-200"
          >
            <img
              :src="imageUrl"
              class="w-full h-full object-cover"
              alt="上传的图片"
            />
            <button
              @click="removeImage(index)"
              class="absolute top-0 right-0 w-4 h-4 bg-black/50 hover:bg-black/70 rounded-full flex items-center justify-center text-white transition-all duration-200"
              title="删除图片"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
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

<style scoped>
</style>
