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
const globalMouseUpHandler = ref(null)
const isUnmounting = ref(false) // 标志：组件是否正在卸载

// 检查选择是否在编辑器内
const isSelectionInEditor = (view) => {
  if (!view || !view.dom) return false

  const selection = window.getSelection()
  if (!selection || selection.rangeCount === 0) {
    return false
  }

  const range = selection.getRangeAt(0)
  const startNode = range.startContainer
  const endNode = range.endContainer

  // 检查选择范围的节点是否在编辑器 DOM 内
  return view.dom.contains(startNode) || view.dom.contains(endNode)
}

// 检查是否有选中的文本
const shouldShow = ({ view, state }) => {
  if (isUnmounting.value) return false
  if (!state || !state.selection) {
    return false
  }

  const { selection } = state
  const { empty } = selection

  // 空选择不显示
  if (empty) {
    return false
  }

  // 获取选中文本
  const selectedText = state.doc.textBetween(selection.from, selection.to)

  // 检查选中文本长度，至少需要 2 个字符才显示菜单
  if (selectedText.length < 2) {
    return false
  }

  // 检查选中文本是否在代码块中
  const { $from } = state.selection
  // 检查父节点是否是代码块
  let node = $from.node($from.depth)
  if (node && node.type.name === 'codeBlock') {
    return false
  }
  // 检查标记中是否包含代码标记
  if ($from.marks().some(mark => mark.type.name === 'code')) {
    return false
  }

  return true
}

// 更新按钮状态
const updateButtonStates = () => {
  if (!tippyInstance.value || !tippyInstance.value.popper) return

  const menu = tippyInstance.value.popper.firstElementChild
  if (!menu) return

  const highlightBtn = menu.querySelector('button[data-action="highlight"]')
  const underlineBtn = menu.querySelector('button[data-action="underline"]')

  if (highlightBtn && highlightBtn.updateState) {
    highlightBtn.updateState()
  }
  if (underlineBtn && underlineBtn.updateState) {
    underlineBtn.updateState()
  }
}

// 显示菜单
const showMenu = () => {
  if (isUnmounting.value || !tippyInstance.value || !props.editor || props.editor.isDestroyed) return

  try {
    const { view, state } = props.editor
    if (view && state && shouldShow({ view, state })) {
      tippyInstance.value.show()
      updateButtonStates()
    }
  } catch {
    // 忽略错误，可能编辑器已销毁
  }
}

// 隐藏菜单
const hideMenu = () => {
  if (!isUnmounting.value && tippyInstance.value) {
    try {
      tippyInstance.value.hide()
    } catch {
      // 忽略错误
    }
  }
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
  // 获取 .iterm-root 元素，确保悬浮菜单在命名空间内
  const itermPanel = document.querySelector('.iterm-root')
  const appendTarget = itermPanel || document.body

  tippyInstance.value = tippy(reference, {
    content: menu,
    trigger: 'manual',
    interactive: true,
    placement: 'top',
    appendTo: appendTarget,
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
  if (!newEditor || isUnmounting.value) return

  // 等待编辑器完全初始化
  setTimeout(() => {
    if (isUnmounting.value) return

    createSelectionMenu()

    const view = newEditor.view

    // 监听所有事务变化
    newEditor.on('update', ({ transaction }) => {
      if (isUnmounting.value || !tippyInstance.value) return

      try {
        const { view, state } = newEditor
        if (view && state && shouldShow({ view, state })) {
          tippyInstance.value.show()
        } else {
          tippyInstance.value.hide()
        }
      } catch {
        // 忽略错误，可能编辑器已销毁
      }
    })

    // 同时监听选择变化事件
    newEditor.on('selectionUpdate', ({ view, state }) => {
      if (isUnmounting.value || !tippyInstance.value) return

      try {
        if (view && state && shouldShow({ view, state })) {
          tippyInstance.value.show()
        } else {
          tippyInstance.value.hide()
        }
      } catch {
        // 忽略错误，可能编辑器已销毁
      }
    })

    // 监听视图的鼠标选择事件（编辑器内）
    view.dom.addEventListener('mouseup', () => {
      if (isUnmounting.value) return
      setTimeout(() => {
        if (!isUnmounting.value) showMenu()
      }, 10)
    })

    // 监听键盘 Shift 键选择
    view.dom.addEventListener('keyup', (e) => {
      if (isUnmounting.value) return
      if (e.shiftKey) {
        setTimeout(() => {
          if (!isUnmounting.value) showMenu()
        }, 10)
      }
    })

    // 添加全局 mouseup 监听器，捕获在编辑器外抬起鼠标的情况
    globalMouseUpHandler.value = (e) => {
      if (isUnmounting.value) return
      // 延迟检查，确保选择已经更新
      setTimeout(() => {
        if (isUnmounting.value) return
        // 检查选择是否在编辑器内
        if (isSelectionInEditor(view)) {
          showMenu()
        } else {
          hideMenu()
        }
      }, 10)
    }

    // 使用捕获模式监听，确保在其他处理之前捕获
    document.addEventListener('mouseup', globalMouseUpHandler.value, { capture: true })
  }, 100)
}

// 监听编辑器选择变化
watch(() => props.editor, (newEditor) => {
  setupEditorListeners(newEditor)
}, { immediate: true })

// 组件卸载时清理 tippy 实例
onBeforeUnmount(() => {
  isUnmounting.value = true

  // 移除全局 mouseup 监听器
  if (globalMouseUpHandler.value) {
    document.removeEventListener('mouseup', globalMouseUpHandler.value, { capture: true })
    globalMouseUpHandler.value = null
  }

  if (tippyInstance.value) {
    try {
      tippyInstance.value.destroy()
    } catch {
      // 忽略销毁错误
    }
    tippyInstance.value = null
  }
  if (referenceElement.value) {
    try {
      referenceElement.value.remove()
    } catch {
      // 忽略移除错误
    }
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