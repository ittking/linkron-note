import { Node } from '@tiptap/core'

export const ResizableImage = Node.create({
  name: 'resizableImage',

  group: 'block',

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
        tag: 'img',
        // 获取所有 img 标签，使其支持 resize
        getAttrs: node => {
          const img = node
          return {
            src: img.getAttribute('src'),
            alt: img.getAttribute('alt'),
            title: img.getAttribute('title'),
            width: img.getAttribute('data-width') || img.style.width || img.getAttribute('width'),
          }
        },
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['img', { ...HTMLAttributes, 'data-resizable': 'true', draggable: 'false' }]
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      // 最外层块级容器
      const block = document.createElement('div')
      block.className = 'resizable-image-block'
      block.style.display = 'block'

      // 内层包装容器
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
      img.style.borderRadius = '0'

      if (node.attrs.width) {
        img.style.width = node.attrs.width
      }

      // 创建拖拽手柄
      const handle = document.createElement('div')
      handle.className = 'resize-handle'

      dom.appendChild(img)
      dom.appendChild(handle)
      block.appendChild(dom)

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
        dom: block,
        destroy: () => {
          // 清理事件监听器
        },
      }
    }
  },
})