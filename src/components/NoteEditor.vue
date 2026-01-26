<script setup>
import { ref, computed, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { invoke } from '@tauri-apps/api/core'
import StarterKit from '@tiptap/starter-kit'
import Highlight from '@tiptap/extension-highlight'
import Placeholder from '@tiptap/extension-placeholder'
import { TagExtension } from '@/extensions/tag-extension'
import { ResizableImage } from '@/extensions/resizable-image'
import tippy from 'tippy.js'
import { useSettingStore } from '@/store/settingStore'
import {
  Hash,
  Image as ImageIcon,
  Highlighter,
  ListOrdered,
  List,
  Underline,
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
  },
  isScrolledToTop: {
    type: Boolean,
    default: true
  },
  isEditing: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'submit', 'image-upload'])

const settingStore = useSettingStore()
const imageInputRef = ref(null)

// 获取工作目录
async function getWorkDirectory() {
  return await settingStore.get('workDirectory', '')
}

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
    ResizableImage,
    Highlight.configure({
      multicolor: true,
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
            console.error('Failed to search tags:', error)
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
                itemEl.className = `tag-suggestion-item flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${isSelected ? 'bg-primary/20 text-primary' : 'hover:bg-primary/10'}`
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
              component.className = 'bg-base-100 border border-base-200 rounded-lg shadow-xl max-h-60 overflow-y-auto p-2'

              // 使用正确的虚拟定位方式
              popup = tippy(document.body, {
                getReferenceClientRect: props.clientRect,
                appendTo: document.body,
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
      class: 'prose prose-sm max-w-none focus:outline-none py-2 text-[14px]',
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
function toggleHighlight() {
  editor.value?.chain().focus().toggleHighlight().run()
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

// 插入标签 #
function insertTag() {
  editor.value?.chain().focus().insertContent('#').run()
}

// 触发图片上传
function triggerImageUpload() {
  imageInputRef.value?.click()
}

async function handleImageUpload(event) {
  const file = event.target.files[0]
  if (file) {
    try {
      // 读取文件为 ArrayBuffer
      const arrayBuffer = await file.arrayBuffer()
      const uint8Array = new Uint8Array(arrayBuffer)
      
      // 调用后端命令保存图片
      const workDirectory = await getWorkDirectory()
      const imagePath = await invoke('save_image', {
        fileData: Array.from(uint8Array),
        fileName: file.name,
        workDirectory
      })
      
      // 使用 iterm:// 协议
      const resourceUrl = await invoke('get_resource_url', { relativePath: imagePath })
      
      // 插入图片到编辑器
      editor.value?.chain().focus().insertContent({
        type: 'resizableImage',
        attrs: {
          src: resourceUrl,
          width: '100px',
        },
      }).run()
    } catch (error) {
      console.error('Failed to save image:', error)
    }
  }
  // 重置 input
  event.target.value = ''
}

function handleSubmit() {
  if (hasContent.value) {
    emit('submit')
    if (!props.isEditing) {
      editor.value?.commands.clearContent()
    }
  }
}
</script>

<template>
  <div class="note-editor relative bg-base-100 border border-primary rounded-xl p-4 shadow-sm transition-all duration-200 focus-within:shadow-md focus-within:border-primary/80">
    <!-- 编辑器内容区域 -->
    <EditorContent 
      :editor="editor" 
      class="mb-3 transition-all duration-200 overflow-y-auto max-h-[400px]"
      :class="{ 
        'min-h-[80px]': props.isScrolledToTop,
        'min-h-[40px]': !props.isScrolledToTop
      }"
    />

    <!-- 底部工具栏 -->
    <div class="flex items-center justify-between">
      <!-- 左侧工具栏 -->
      <div class="flex items-center gap-3">
        <!-- 标签 # -->
        <button
          @click="insertTag"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入标签"
        >
          <Hash :size="14" />
        </button>

        <!-- 图片 -->
        <button
          @click="triggerImageUpload"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          title="插入图片"
        >
          <ImageIcon :size="14" />
          <input
            ref="imageInputRef"
            type="file"
            accept="image/*"
            class="hidden"
            @change="handleImageUpload"
          />
        </button>

        <!-- 背景高亮 -->
        <button
          @click="toggleHighlight"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('highlight') }"
          title="背景高亮"
        >
          <Highlighter :size="14" />
        </button>

        <!-- 下划线 -->
        <button
          @click="toggleUnderline"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('underline') }"
          title="下划线"
        >
          <Underline :size="14" />
        </button>

        <!-- 有序列表 -->
        <button
          @click="toggleOrderedList"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('orderedList') }"
          title="有序列表"
        >
          <ListOrdered :size="14" />
        </button>

        <!-- 无序列表 -->
        <button
          @click="toggleBulletList"
          class="w-6 h-6 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200"
          :class="{ 'text-primary bg-primary/10': editor?.isActive('bulletList') }"
          title="无序列表"
        >
          <List :size="14" />
        </button>
      </div>

      <!-- 右侧按钮区域 -->
      <div class="flex items-center gap-2">
        <!-- 插槽：用于放置取消按钮等自定义按钮 -->
        <slot name="actions"></slot>
        
        <!-- 发送按钮 -->
        <button
          @click="handleSubmit"
          class="w-6 h-6 rounded-md flex items-center justify-center transition-all duration-200"
          :class="[
            hasContent
              ? 'bg-primary text-primary-content hover:bg-primary/90'
              : 'bg-base-300 text-base-content/40 cursor-not-allowed'
          ]"
          :disabled="!hasContent"
          :title="props.isEditing ? '保存' : '发送'"
        >
          <Send :size="13" />
        </button>
      </div>
    </div>
  </div>
</template>
