import { Node } from '@tiptap/core'

export const ResizableImage = Node.create({
  name: 'resizableImage',

  group: 'inline',

  inline: true,

  atom: true,

  addAttributes() {
    return {
      src: {
        default: null,
      },
      alt: {
        default: null,
      },
      title: {
        default: null,
      },
      width: {
        default: null,
        parseHTML: element => element.getAttribute('data-width'),
        renderHTML: attributes => {
          if (!attributes.width) {
            return {}
          }
          return {
            'data-width': attributes.width,
            width: attributes.width,
          }
        },
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'img[data-resizable]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['img', { ...HTMLAttributes, 'data-resizable': 'true', draggable: 'false' }]
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      const dom = document.createElement('div')
      dom.className = 'resizable-image-wrapper'
      dom.style.display = 'inline-block'
      dom.style.position = 'relative'

      const img = document.createElement('img')
      img.src = node.attrs.src
      img.alt = node.attrs.alt || ''
      img.draggable = false
      img.style.maxWidth = '100%'
      img.style.height = 'auto'
      img.style.display = 'block'

      if (node.attrs.width) {
        img.style.width = node.attrs.width
      }

      // 创建拖拽手柄
      const handle = document.createElement('div')
      handle.className = 'resize-handle'
      handle.style.position = 'absolute'
      handle.style.width = '12px'
      handle.style.height = '12px'
      handle.style.background = 'var(--primary, #3b82f6)'
      handle.style.borderRadius = '50%'
      handle.style.cursor = 'nwse-resize'
      handle.style.bottom = '-6px'
      handle.style.right = '-6px'
      handle.style.border = '2px solid white'
      handle.style.boxShadow = '0 0 4px rgba(0,0,0,0.3)'
      handle.style.zIndex = '10'
      handle.style.display = 'none'

      dom.appendChild(img)
      dom.appendChild(handle)

      // 鼠标悬停时显示手柄
      dom.addEventListener('mouseenter', () => {
        handle.style.display = 'block'
      })

      dom.addEventListener('mouseleave', () => {
        handle.style.display = 'none'
      })

      // 拖拽调整大小
      let startX = 0
      let startWidth = 0

      handle.addEventListener('mousedown', (e) => {
        e.preventDefault()
        e.stopPropagation()
        startX = e.clientX
        startWidth = img.offsetWidth

        const onMouseMove = (moveEvent) => {
          const diff = moveEvent.clientX - startX
          const newWidth = Math.max(50, startWidth + diff)
          img.style.width = newWidth + 'px'
        }

        const onMouseUp = (upEvent) => {
          document.removeEventListener('mousemove', onMouseMove)
          document.removeEventListener('mouseup', onMouseUp)

          const diff = upEvent.clientX - startX
          const newWidth = Math.max(50, startWidth + diff)

          // 更新节点属性
          if (typeof getPos === 'function') {
            const pos = getPos()
            if (pos !== undefined) {
              const tr = editor.view.state.tr.setNodeMarkup(pos, null, {
                ...node.attrs,
                width: newWidth + 'px',
              })
              editor.view.dispatch(tr)
            }
          }
        }

        document.addEventListener('mousemove', onMouseMove)
        document.addEventListener('mouseup', onMouseUp)
      })

      return {
        dom,
        destroy: () => {
          // 清理事件监听器
        },
      }
    }
  },
})