<script setup>
import { ref, watch, onBeforeUnmount, h, render, nextTick } from 'vue'
import { Highlighter, Underline, Italic, Bold, Link as LinkIcon } from 'lucide-vue-next'
import tippy from 'tippy.js'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'

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

// 链接弹窗状态
const linkDialog = ref({
  visible: false,
  url: ''
})
const linkInputRef = ref(null)

// 检查选择是否在编辑器内
const isSelectionInEditor = (view) => {
  if (isUnmounting.value || !view || !view.dom) return false

  try {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) {
      return false
    }

    const range = selection.getRangeAt(0)
    const startNode = range.startContainer
    const endNode = range.endContainer

    // 检查选择范围的节点是否在编辑器 DOM 内
    return view.dom.contains(startNode) || view.dom.contains(endNode)
  } catch {
    return false
  }
}

// 检查是否有选中的文本
const shouldShow = ({ view, state }) => {
  if (isUnmounting.value) return false
  if (!view || !view.dom || !state || !state.selection) {
    return false
  }

  try {
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
  } catch {
    return false
  }
}

// 更新按钮状态
const updateButtonStates = () => {
  if (!tippyInstance.value || !tippyInstance.value.popper) return

  const menu = tippyInstance.value.popper.firstElementChild
  if (!menu) return

  const highlightBtn = menu.querySelector('button[data-action="highlight"]')
  const underlineBtn = menu.querySelector('button[data-action="underline"]')
  const italicBtn = menu.querySelector('button[data-action="italic"]')
  const boldBtn = menu.querySelector('button[data-action="bold"]')
  const linkBtn = menu.querySelector('button[data-action="link"]')

  if (highlightBtn && highlightBtn.updateState) {
    highlightBtn.updateState()
  }
  if (underlineBtn && underlineBtn.updateState) {
    underlineBtn.updateState()
  }
  if (italicBtn && italicBtn.updateState) {
    italicBtn.updateState()
  }
  if (boldBtn && boldBtn.updateState) {
    boldBtn.updateState()
  }
  if (linkBtn && linkBtn.updateState) {
    linkBtn.updateState()
  }
}

// 显示菜单
const showMenu = () => {
  if (isUnmounting.value || !tippyInstance.value || !props.editor || props.editor.isDestroyed) return

  try {
    const { view, state } = props.editor
    if (view && view.dom && state && shouldShow({ view, state })) {
      tippyInstance.value.show()
      updateButtonStates()
    }
  } catch {
    // 忽略错误，可能编辑器已销毁
  }
}

// 隐藏菜单
const hideMenu = () => {
  if (isUnmounting.value || !tippyInstance.value) return

  try {
    tippyInstance.value.hide()
  } catch {
    // 忽略错误
  }
}

// 设置链接
function setLink() {
  const previousUrl = props.editor.getAttributes('link').href
  linkDialog.value.url = previousUrl || ''
  linkDialog.value.visible = true
  
  nextTick(() => {
    if (linkInputRef.value) {
      linkInputRef.value.focus()
      linkInputRef.value.select()
    }
  })
}

// 确认设置链接
function confirmLink() {
  const url = linkDialog.value.url.trim()
  if (url === '') {
    props.editor.chain().focus().unsetLink().run()
  } else {
    props.editor.chain().focus().setLink({ href: url }).run()
  }
  linkDialog.value.visible = false
}

// 取消设置链接
function cancelLink() {
  linkDialog.value.visible = false
  linkDialog.value.url = ''
}

// 移除链接
function removeLink() {
  props.editor.chain().focus().unsetLink().run()
  linkDialog.value.visible = false
}

// 处理链接输入框键盘事件
function handleLinkKeyDown(e) {
  if (e.key === 'Enter') {
    confirmLink()
  } else if (e.key === 'Escape') {
    cancelLink()
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

  // 加粗按钮
  const boldBtn = document.createElement('button')
  boldBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  boldBtn.title = '加粗'
  boldBtn.dataset.action = 'bold'
  boldBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleBold().run()
    setTimeout(() => boldBtn.updateState(), 10)
  })

  // 斜体按钮
  const italicBtn = document.createElement('button')
  italicBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  italicBtn.title = '斜体'
  italicBtn.dataset.action = 'italic'
  italicBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleItalic().run()
    setTimeout(() => italicBtn.updateState(), 10)
  })

  // 下划线按钮
  const underlineBtn = document.createElement('button')
  underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  underlineBtn.title = '下划线'
  underlineBtn.dataset.action = 'underline'
  underlineBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleUnderline().run()
    setTimeout(() => underlineBtn.updateState(), 10)
  })

  // 背景高亮按钮
  const highlightBtn = document.createElement('button')
  highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  highlightBtn.title = '背景高亮'
  highlightBtn.dataset.action = 'highlight'
  highlightBtn.addEventListener('click', (e) => {
    e.preventDefault()
    props.editor.chain().focus().toggleHighlight().run()
    setTimeout(() => highlightBtn.updateState(), 10)
  })

  // 链接按钮
  const linkBtn = document.createElement('button')
  linkBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
  linkBtn.title = '设为链接'
  linkBtn.dataset.action = 'link'
  linkBtn.addEventListener('click', (e) => {
    e.preventDefault()
    setLink()
    setTimeout(() => linkBtn.updateState(), 10)
  })

  // 渲染图标
  render(h(Bold, { size: 14 }), boldBtn)
  render(h(Italic, { size: 14 }), italicBtn)
  render(h(Underline, { size: 14 }), underlineBtn)
  render(h(Highlighter, { size: 14 }), highlightBtn)
  render(h(LinkIcon, { size: 14 }), linkBtn)

  // 更新按钮选中状态的函数
  const updateButtonStates = () => {
    if (!props.editor) return

    const isBoldActive = props.editor.isActive('bold')
    const isItalicActive = props.editor.isActive('italic')
    const isUnderlineActive = props.editor.isActive('underline')
    const isHighlightActive = props.editor.isActive('highlight')
    const isLinkActive = props.editor.isActive('link')

    if (isBoldActive) {
      boldBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      boldBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }

    if (isItalicActive) {
      italicBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      italicBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }

    if (isUnderlineActive) {
      underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      underlineBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }

    if (isHighlightActive) {
      highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      highlightBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }

    if (isLinkActive) {
      linkBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-primary bg-primary/10 transition-all duration-200'
    } else {
      linkBtn.className = 'w-7 h-7 rounded-md flex items-center justify-center text-base-content/50 hover:text-base-content hover:bg-base-200 transition-all duration-200'
    }
  }

  // 将更新函数存储到按钮上，以便后续调用
  boldBtn.updateState = updateButtonStates
  italicBtn.updateState = updateButtonStates
  underlineBtn.updateState = updateButtonStates
  highlightBtn.updateState = updateButtonStates
  linkBtn.updateState = updateButtonStates

  menu.appendChild(boldBtn)
  menu.appendChild(italicBtn)
  menu.appendChild(underlineBtn)
  menu.appendChild(highlightBtn)
  menu.appendChild(linkBtn)

  // 创建一个虚拟的 reference 元素
  const reference = document.createElement('div')
  reference.style.position = 'fixed'
  reference.style.top = '0'
  reference.style.left = '0'
  referenceElement.value = reference
  document.body.appendChild(reference)

  // 创建 Tippy 实例
  // 将悬浮菜单添加到 body 中
  const appendTarget = document.body

  tippyInstance.value = tippy(reference, {
    content: menu,
    trigger: 'manual',
    interactive: true,
    placement: 'top',
    appendTo: appendTarget,
    getReferenceClientRect: () => {
      if (isUnmounting.value || !props.editor || props.editor.isDestroyed) {
        return { left: 0, right: 0, top: 0, bottom: 0, width: 0, height: 0 }
      }

      try {
        const { view, state } = props.editor
        if (!view || !state || !state.selection) {
          return { left: 0, right: 0, top: 0, bottom: 0, width: 0, height: 0 }
        }

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
      } catch {
        return { left: 0, right: 0, top: 0, bottom: 0, width: 0, height: 0 }
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

    // 只监听选择变化事件，不监听 update 事件
    // update 事件在输入时也会触发，而 selectionUpdate 只在选择变化时触发
    newEditor.on('selectionUpdate', ({ view, state }) => {
      if (isUnmounting.value || !tippyInstance.value) return

      try {
        if (view && view.dom && state && shouldShow({ view, state })) {
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

  <!-- 链接弹窗 -->
  <dialog :open="linkDialog.visible" class="modal">
    <div class="modal-box bg-base-200 border border-base-300">
      <h3 class="font-bold text-sm text-base-content">设置链接</h3>
      <div class="py-4">
        <Input
          ref="linkInputRef"
          v-model="linkDialog.url"
          type="text"
          placeholder="请输入链接地址"
          size="sm"
          @keyup.enter="confirmLink"
          @keydown="handleLinkKeyDown"
        />
      </div>
      <div class="modal-action">
        <Button variant="ghost" size="sm" @click="cancelLink">
          取消
        </Button>
        <Button variant="error" size="sm" @click="removeLink">
          移除链接
        </Button>
        <Button variant="primary" size="sm" @click="confirmLink">
          确定
        </Button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop bg-black/50" @click="cancelLink">
      <button></button>
    </form>
  </dialog>
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