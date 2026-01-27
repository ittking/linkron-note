<script setup>
import { ref, watch, onBeforeUnmount, h, render } from 'vue'
import { Highlighter, Underline } from 'lucide-vue-next'
import tippy from 'tippy.js'

const props = defineProps({
  editor: {
    type: Object,
    required: true
  }
})

const tippyInstance = ref(null)
const referenceElement = ref(null)

// 检查是否有选中的文本
const shouldShow = ({ view, state }) => {
  if (!state || !state.selection) {
    return false
  }

  const { selection } = state
  const { empty } = selection
  const isTextSelected = !empty && state.doc.textBetween(selection.from, selection.to).trim().length > 0

  return isTextSelected
}

// 创建悬浮菜单
function createSelectionMenu() {
  if (!props.editor) {
    console.log('[SelectionMenu] Editor not available')
    return
  }

  console.log('[SelectionMenu] Creating selection menu...')

  // 创建菜单容器
  const menu = document.createElement('div')
  menu.className = 'selection-menu flex items-center gap-1 bg-base-100 border border-base-200 rounded-lg shadow-xl p-1'

  // 创建 Vue 应用来渲染按钮
  const highlightBtn = document.createElement('button')
  highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  highlightBtn.title = '背景高亮'
  highlightBtn.addEventListener('click', (e) => {
    e.preventDefault()
    console.log('[SelectionMenu] Highlight button clicked')
    props.editor.chain().focus().toggleHighlight().run()
  })

  const underlineBtn = document.createElement('button')
  underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  underlineBtn.title = '下划线'
  underlineBtn.addEventListener('click', (e) => {
    e.preventDefault()
    console.log('[SelectionMenu] Underline button clicked')
    props.editor.chain().focus().toggleUnderline().run()
  })

  // 渲染图标
  render(h(Highlighter, { size: 14 }), highlightBtn)
  render(h(Underline, { size: 14 }), underlineBtn)

  menu.appendChild(highlightBtn)
  menu.appendChild(underlineBtn)

  // 创建一个虚拟的 reference 元素
  const reference = document.createElement('div')
  reference.style.position = 'fixed'
  reference.style.top = '0'
  reference.style.left = '0'
  referenceElement.value = reference
  document.body.appendChild(reference)

  // 创建 Tippy 实例
  tippyInstance.value = tippy(reference, {
    content: menu,
    trigger: 'manual',
    interactive: true,
    placement: 'top',
    appendTo: document.body,
    getReferenceClientRect: () => {
      const { view, state } = props.editor
      const { selection } = state

      // 获取选中文本的坐标
      const coords = view.coordsAtPos(selection.from)
      const endCoords = view.coordsAtPos(selection.to)

      return {
        left: coords.left + (endCoords.left - coords.left) / 2 - 50,
        right: coords.right,
        top: coords.top,
        bottom: coords.top,
        width: 100,
        height: 0,
      }
    },
    duration: [200, 100],
    onShow: () => {
      console.log('[SelectionMenu] Tippy menu shown')
    },
    onHide: () => {
      console.log('[SelectionMenu] Tippy menu hidden')
    }
  })

  console.log('[SelectionMenu] Selection menu created successfully')
}

// 设置编辑器监听
function setupEditorListeners(newEditor) {
  if (!newEditor) return

  console.log('[SelectionMenu] Setting up editor listeners...')

  // 等待编辑器完全初始化
  setTimeout(() => {
    createSelectionMenu()

    // 监听所有事务变化
    newEditor.on('update', ({ transaction }) => {
      console.log('[SelectionMenu] Transaction updated:', {
        docChanged: transaction.docChanged,
        selectionSet: transaction.selectionSet
      })

      if (tippyInstance.value) {
        const { view, state } = newEditor
        if (view && state && shouldShow({ view, state })) {
          console.log('[SelectionMenu] Showing menu')
          tippyInstance.value.show()
        } else {
          console.log('[SelectionMenu] Hiding menu')
          tippyInstance.value.hide()
        }
      }
    })

    // 同时监听选择变化事件
    newEditor.on('selectionUpdate', ({ view, state }) => {
      console.log('[SelectionMenu] Selection updated event:', {
        hasState: !!state,
        hasView: !!view,
        state
      })

      if (tippyInstance.value && view && state) {
        if (shouldShow({ view, state })) {
          console.log('[SelectionMenu] Showing menu from selectionUpdate')
          tippyInstance.value.show()
        } else {
          console.log('[SelectionMenu] Hiding menu from selectionUpdate')
          tippyInstance.value.hide()
        }
      }
    })

    // 监听视图的鼠标选择事件
    const view = newEditor.view
    view.dom.addEventListener('mouseup', () => {
      console.log('[SelectionMenu] Mouse up detected')
      setTimeout(() => {
        if (tippyInstance.value) {
          const { view, state } = newEditor
          if (shouldShow({ view, state })) {
            console.log('[SelectionMenu] Showing menu from mouseup')
            tippyInstance.value.show()
          } else {
            console.log('[SelectionMenu] Hiding menu from mouseup')
            tippyInstance.value.hide()
          }
        }
      }, 10)
    })

    view.dom.addEventListener('keyup', (e) => {
      if (e.shiftKey) {
        console.log('[SelectionMenu] Key up with shift detected')
        setTimeout(() => {
          if (tippyInstance.value) {
            const { view, state } = newEditor
            if (shouldShow({ view, state })) {
              console.log('[SelectionMenu] Showing menu from keyup')
              tippyInstance.value.show()
            } else {
              console.log('[SelectionMenu] Hiding menu from keyup')
              tippyInstance.value.hide()
            }
          }
        }, 10)
      }
    })
  }, 100)
}

// 监听编辑器选择变化
watch(() => props.editor, (newEditor) => {
  console.log('[SelectionMenu] Editor changed:', newEditor ? 'exists' : 'null')
  setupEditorListeners(newEditor)
}, { immediate: true })

// 组件卸载时清理 tippy 实例
onBeforeUnmount(() => {
  console.log('[SelectionMenu] Cleaning up...')
  if (tippyInstance.value) {
    tippyInstance.value.destroy()
    tippyInstance.value = null
  }
  if (referenceElement.value) {
    referenceElement.value.remove()
    referenceElement.value = null
  }
})
</script>

<template>
  <!-- SelectionMenu 组件不需要渲染任何内容，悬浮菜单是通过 DOM 操作创建的 -->
  <div></div>
</template>

<style>
/* 悬浮菜单样式 */
.selection-menu {
  z-index: 1000;
}

.selection-menu button {
  position: relative;
}

/* 激活状态的样式 */
.selection-menu button.active {
  color: hsl(var(--p));
  background-color: hsl(var(--p) / 0.1);
}
</style>