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
    return
  }

  // 创建菜单容器
  const menu = document.createElement('div')
  menu.className = 'selection-menu flex items-center gap-1 bg-base-100 border border-base-200 rounded-lg shadow-xl p-1'

  // 创建 Vue 应用来渲染按钮
  const highlightBtn = document.createElement('button')
  highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  highlightBtn.title = '背景高亮'
  highlightBtn.dataset.action = 'highlight'
  highlightBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleHighlight().run()
    // 更新按钮状态
    setTimeout(() => highlightBtn.updateState(), 10)
  })

  const underlineBtn = document.createElement('button')
  underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  underlineBtn.title = '下划线'
  underlineBtn.dataset.action = 'underline'
  underlineBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleUnderline().run()
    // 更新按钮状态
    setTimeout(() => underlineBtn.updateState(), 10)
  })

  // 渲染图标
  render(h(Highlighter, { size: 14 }), highlightBtn)
  render(h(Underline, { size: 14 }), underlineBtn)

  // 更新按钮选中状态的函数
  const updateButtonStates = () => {
    if (!props.editor) return

    const isHighlightActive = props.editor.isActive('highlight')
    const isUnderlineActive = props.editor.isActive('underline')

    if (isHighlightActive) {
      highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }

    if (isUnderlineActive) {
      underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }
  }

  // 将更新函数存储到按钮上，以便后续调用
  highlightBtn.updateState = updateButtonStates
  underlineBtn.updateState = updateButtonStates

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
  })
}

// 设置编辑器监听
function setupEditorListeners(newEditor) {
  if (!newEditor) return

  // 等待编辑器完全初始化
  setTimeout(() => {
    createSelectionMenu()

    // 监听所有事务变化
    newEditor.on('update', ({ transaction }) => {
      if (tippyInstance.value) {
        const { view, state } = newEditor
        if (view && state && shouldShow({ view, state })) {
          tippyInstance.value.show()
        } else {
          tippyInstance.value.hide()
        }
      }
    })

    // 同时监听选择变化事件
    newEditor.on('selectionUpdate', ({ view, state }) => {
      if (tippyInstance.value && view && state) {
        if (shouldShow({ view, state })) {
          tippyInstance.value.show()
        } else {
          tippyInstance.value.hide()
        }
      }
    })

    // 监听视图的鼠标选择事件
    const view = newEditor.view
    view.dom.addEventListener('mouseup', () => {
      setTimeout(() => {
        if (tippyInstance.value) {
          const { view, state } = newEditor
          if (shouldShow({ view, state })) {
            tippyInstance.value.show()
            // 更新按钮状态
            if (tippyInstance.value.popper && tippyInstance.value.popper.firstElementChild) {
              const menu = tippyInstance.value.popper.firstElementChild
              const highlightBtn = menu.querySelector('button[data-action="highlight"]')
              const underlineBtn = menu.querySelector('button[data-action="underline"]')
              if (highlightBtn && highlightBtn.updateState) {
                highlightBtn.updateState()
              }
              if (underlineBtn && underlineBtn.updateState) {
                underlineBtn.updateState()
              }
            }
          } else {
            tippyInstance.value.hide()
          }
        }
      }, 10)
    })

    view.dom.addEventListener('keyup', (e) => {
      if (e.shiftKey) {
        setTimeout(() => {
          if (tippyInstance.value) {
            const { view, state } = newEditor
            if (shouldShow({ view, state })) {
              tippyInstance.value.show()
              // 更新按钮状态
              if (tippyInstance.value.popper && tippyInstance.value.popper.firstElementChild) {
                const menu = tippyInstance.value.popper.firstElementChild
                const highlightBtn = menu.querySelector('button[data-action="highlight"]')
                const underlineBtn = menu.querySelector('button[data-action="underline"]')
                if (highlightBtn && highlightBtn.updateState) {
                  highlightBtn.updateState()
                }
                if (underlineBtn && underlineBtn.updateState) {
                  underlineBtn.updateState()
                }
              }
            } else {
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
  setupEditorListeners(newEditor)
}, { immediate: true })

// 组件卸载时清理 tippy 实例
onBeforeUnmount(() => {
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