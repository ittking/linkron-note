import { Node } from '@tiptap/core'

export const ResizableImage = Node.create({
  name: 'resizableImage',

  group: 'block',

  atom: true,

  addOptions() {
    return {
      editable: true,
    }
  },

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
      displayMode: {
        default: 'auto',
        parseHTML: element => element.getAttribute('data-display-mode') || 'auto',
        renderHTML: attributes => {
          return {
            'data-display-mode': attributes.displayMode,
          }
        },
      },
      hasBorder: {
        default: false,
        parseHTML: element => element.getAttribute('data-has-border') === 'true',
        renderHTML: attributes => {
          if (!attributes.hasBorder) {
            return {}
          }
          return {
            'data-has-border': 'true',
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
            displayMode: img.getAttribute('data-display-mode') || 'auto',
            hasBorder: img.getAttribute('data-has-border') === 'true',
          }
        },
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['img', { ...HTMLAttributes, 'data-resizable': 'true', draggable: 'false' }]
  },

  addNodeView() {
    return ({ node, getPos, editor, options }) => {
      const isEditable = this.options.editable !== false

      // 最外层块级容器
      const block = document.createElement('div')
      block.className = 'resizable-image-block'
      block.style.display = 'block'

      // 内层包装容器
      const dom = document.createElement('div')
      dom.className = 'resizable-image-wrapper'
      if (isEditable) {
        dom.classList.add('editable')
      }
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
      img.style.position = 'relative'

      // 应用显示模式
      const displayMode = node.attrs.displayMode || 'auto'
      const hasBorder = node.attrs.hasBorder || false

      // 默认宽度为auto
      if (node.attrs.width && displayMode === 'fixed') {
        img.style.width = node.attrs.width
      } else if (displayMode === 'full') {
        img.style.width = '100%'
      } else {
        img.style.width = 'auto'
      }

      // 应用边框
      if (hasBorder) {
        img.style.border = '1px solid rgba(0, 0, 0, 0.1)'
        img.style.borderRadius = '8px'
        img.style.padding = '8px'
        img.style.backgroundColor = 'var(--color-base-300, #ffffff)'
      }

      dom.appendChild(img)
      block.appendChild(dom)

      // 只有在可编辑模式下才创建控件
      if (isEditable) {
        // 创建拖拽手柄
        const handle = document.createElement('div')
        handle.className = 'resize-handle'

        // 创建底部控制栏（在图片内部左下角）
        const toolbar = document.createElement('div')
        toolbar.className = 'image-toolbar'
        toolbar.style.cssText = `
          position: absolute;
          bottom: 8px;
          left: 8px;
          display: flex;
          gap: 4px;
          background: rgba(0, 0, 0, 0.6);
          backdrop-filter: blur(4px);
          padding: 4px;
          border-radius: 6px;
          opacity: 0;
          transition: opacity 0.2s;
        `

        // 创建图标按钮
        function createIconButton(icon, title, isActive, onClick) {
          const button = document.createElement('button')
          button.type = 'button'
          button.title = title
          button.style.cssText = `
            width: 28px;
            height: 28px;
            border: none;
            border-radius: 4px;
            background: ${isActive ? 'rgba(59, 130, 246, 0.4)' : 'rgba(255, 255, 255, 0.1)'};
            color: ${isActive ? '#3b82f6' : 'rgba(255, 255, 255, 0.9)'};
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 14px;
            transition: all 0.2s;
          `
          button.innerHTML = icon
          button.addEventListener('click', (e) => {
            e.preventDefault()
            e.stopPropagation()
            onClick()
          })
          button.addEventListener('mouseenter', () => {
            button.style.background = 'rgba(255, 255, 255, 0.2)'
          })
          button.addEventListener('mouseleave', () => {
            button.style.background = isActive ? 'rgba(59, 130, 246, 0.4)' : 'rgba(255, 255, 255, 0.1)'
          })
          return button
        }

        // 撑满图标
        const fullIcon = createIconButton(
          '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/></svg>',
          '撑满宽度',
          displayMode === 'full',
          () => {
            // 如果当前是撑满模式，切换到auto模式
            if (displayMode === 'full') {
              updateImageAttributes({
                displayMode: 'auto',
                width: null,
                hasBorder: hasBorder
              })
            } else {
              // 否则切换到撑满模式
              updateImageAttributes({
                displayMode: 'full',
                width: null,
                hasBorder: hasBorder
              })
            }
          }
        )

        // 边框图标
        const borderIcon = createIconButton(
          '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg>',
          '带边框',
          hasBorder,
          () => {
            updateImageAttributes({
              displayMode: displayMode,
              width: node.attrs.width,
              hasBorder: !hasBorder
            })
          }
        )

        toolbar.appendChild(fullIcon)
        toolbar.appendChild(borderIcon)

        dom.appendChild(handle)
        dom.appendChild(toolbar)

        // 鼠标悬停时显示手柄和工具栏
        dom.addEventListener('mouseenter', () => {
          handle.style.display = 'block'
          toolbar.style.opacity = '1'
        })

        dom.addEventListener('mouseleave', () => {
          handle.style.display = 'none'
          toolbar.style.opacity = '0'
        })

        // 更新图片属性
        function updateImageAttributes(attrs) {
          if (typeof getPos === 'function') {
            const pos = getPos()
            if (pos !== undefined) {
              const tr = editor.view.state.tr.setNodeMarkup(pos, null, {
                ...node.attrs,
                ...attrs
              })
              editor.view.dispatch(tr)
            }
          }
        }

        // 鼠标悬停时显示手柄和工具栏
        dom.addEventListener('mouseenter', () => {
          handle.style.display = 'block'
          toolbar.style.opacity = '1'
        })

        dom.addEventListener('mouseleave', () => {
          handle.style.display = 'none'
          toolbar.style.opacity = '0'
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

            // 更新节点属性，设置为固定宽度模式
            updateImageAttributes({
              displayMode: 'fixed',
              width: newWidth + 'px',
              hasBorder: hasBorder
            })
          }

          document.addEventListener('mousemove', onMouseMove)
          document.addEventListener('mouseup', onMouseUp)
        })
      }

      return {
        dom: block,
        destroy: () => {
          // 清理事件监听器
        },
      }
    }
  },
})