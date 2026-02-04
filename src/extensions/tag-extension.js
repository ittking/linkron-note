import { Node, mergeAttributes } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import tippy from 'tippy.js'

export const TagExtension = Node.create({
  name: 'tag',

  group: 'inline',
  inline: true,
  selectable: true,
  draggable: true,
  atom: true, // 不可编辑的原子单元

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: element => element.getAttribute('data-id'),
        renderHTML: attributes => {
          if (!attributes.id) return {}
          return { 'data-id': attributes.id }
        },
      },
      name: {
        default: null,
        parseHTML: element => element.getAttribute('data-name'),
        renderHTML: attributes => {
          if (!attributes.name) return {}
          return { 'data-name': attributes.name }
        },
      },
      displayName: {
        default: null,
        parseHTML: element => element.getAttribute('data-display-name'),
        renderHTML: attributes => {
          if (!attributes.displayName) return {}
          return { 'data-display-name': attributes.displayName }
        },
      },
      path: {
        default: null,
        parseHTML: element => element.getAttribute('data-path'),
        renderHTML: attributes => {
          if (!attributes.path) return {}
          return { 'data-path': attributes.path }
        },
      },
      level: {
        default: 1,
        parseHTML: element => parseInt(element.getAttribute('data-level') || '1', 10),
        renderHTML: attributes => {
          return { 'data-level': attributes.level }
        },
      },
    }
  },

  parseHTML() {
    return [{
      tag: 'span[data-type="tag"]',
    }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes({
        'data-type': 'tag',
        class: 'inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors select-none',
      }, HTMLAttributes),
      `#${HTMLAttributes.name || HTMLAttributes.displayName || ''}`,
    ]
  },

  addNodeView() {
    return ({ node }) => {
      const span = document.createElement('span')
      span.className = 'inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors select-none'
      span.dataset.type = 'tag'
      span.dataset.id = node.attrs.id || ''
      span.dataset.name = node.attrs.name || ''
      span.dataset.displayName = node.attrs.displayName || ''
      span.dataset.path = node.attrs.path || ''
      span.dataset.level = node.attrs.level || 1
      span.contentEditable = 'false'
      span.textContent = `#${node.attrs.name || node.attrs.displayName || ''}`

      // 点击事件：派发自定义事件
      span.addEventListener('click', () => {
        const event = new CustomEvent('tag-click', {
          detail: { tag: node.attrs },
          bubbles: true
        })
        span.dispatchEvent(event)
      })

      return { dom: span }
    }
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        char: '#',
        allowedChars: 'a-zA-Z0-9_\\u4e00-\\u9fa5/',
        pluginKey: new PluginKey('tag-suggestion'),

        items: async ({ query }) => {
          const { useNoteStore } = await import('@/store/noteStore')
          const noteStore = useNoteStore()

          if (!query) {
            return await noteStore.searchTags('', 5)
          }

          return await noteStore.searchTags(query, 5)
        },

        render: () => {
          let component
          let popup

          return {
            onStart: (props) => {
              component = document.createElement('div')
              component.className = 'tag-suggestions-dropdown bg-base-100 border border-base-300 rounded-lg shadow-lg p-1 max-h-60 overflow-y-auto z-50'

              popup = tippy('body', {
                getReferenceClientRect: props.clientRect,
                appendTo: () => document.body,
                content: component,
                showOnCreate: true,
                interactive: true,
                trigger: 'manual',
                placement: 'bottom-start',
              })
            },

            onUpdate: (props) => {
              component.innerHTML = ''

              if (props.items.length === 0) {
                const empty = document.createElement('div')
                empty.className = 'px-3 py-2 text-sm text-base-content/60'
                empty.textContent = '按空格创建新标签'
                component.appendChild(empty)
                return
              }

              props.items.forEach((item, index) => {
                const button = document.createElement('button')

                const indent = '  '.repeat(item.level - 1)
                const count = item.count || 0

                button.className = `w-full text-left px-3 py-2 text-sm rounded-md transition-colors ${
                  index === props.selectedIndex
                    ? 'bg-primary text-primary-content'
                    : 'hover:bg-base-200 text-base-content'
                }`
                button.innerHTML = `
                  <span class="whitespace-nowrap">${indent}${item.name}</span>
                  <span class="ml-2 text-xs opacity-60">(${count})</span>
                `

                button.onclick = () => props.command({ item })
                component.appendChild(button)
              })
            },

            onKeyDown: (props) => {
              if (props.event.key === 'Escape') {
                popup?.hide()
                return true
              }

              if (props.event.key === 'Enter') {
                const selected = props.items[props.selectedIndex]
                if (selected) {
                  props.command({ item: selected })
                  return true
                }
              }

              return false
            },

            onExit: () => {
              popup?.destroy()
              component?.remove()
            },
          }
        },

        command: ({ editor, range, props }) => {
          const tag = props.item

          // 替换输入的文本为标签路径
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertText(tag.name)
            .run()
        },
      }),
    ]
  },
})
