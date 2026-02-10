import { Extension } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'
import { invoke } from '@tauri-apps/api/core'

const pluginKey = new PluginKey('tagSuggestion')

let searchTimer = null
let lastQuery = ''

export const TagSuggestion = Extension.create({
  name: 'tagSuggestion',

  addProseMirrorPlugins() {
    const editor = this.editor
    let getWorkDirectoryFn = null

    // 获取 getWorkDirectory 函数
    if (editor && editor.options && editor.options.editorProps) {
      getWorkDirectoryFn = editor.options.editorProps.getWorkDirectory
    }

    return [
      new Plugin({
        key: pluginKey,
        state: {
          init() {
            return {
              active: false,
              query: '',
              items: [],
              selectedIndex: 0,
              range: { from: 0, to: 0 },
              popup: null,
            }
          },
          apply(tr, pluginState, oldState, newState) {
            // 只在文档内容真正变化时才处理
            if (!tr.docChanged) {
              return pluginState
            }

            // 使用事务后的新状态
            const { selection } = newState
            const { $from } = selection
            
            // 检查是否在标签内
            const marks = newState.storedMarks || $from.marks()
            const tagMark = marks.find(mark => mark.type.name === 'tag')
            
            if (tagMark) {
              // 在标签内，不显示建议
              if (pluginState.popup) {
                destroyPopup(pluginState)
              }
              return {
                ...pluginState,
                active: false,
                query: '',
                items: [],
              }
            }
            
            // 获取光标前的文本（最多50个字符）
            const maxLookback = 50
            const startPos = Math.max(0, $from.pos - maxLookback)
            const textBefore = newState.doc.textBetween(startPos, $from.pos)
            
            // 查找最近的 # 字符
            const match = textBefore.match(/#([^\s#]+)$/)

            if (!match || match[1] === '') {
              // 清除定时器
              if (searchTimer) {
                clearTimeout(searchTimer)
                searchTimer = null
              }
              if (pluginState.popup) {
                destroyPopup(pluginState)
              }
              lastQuery = ''
              return {
                ...pluginState,
                active: false,
                query: '',
                items: [],
              }
            }

            const query = match[1]
            const from = $from.pos - query.length - 1
            const to = $from.pos

            // 如果查询没有变化，不重新搜索
            if (query === lastQuery) {
              return {
                ...pluginState,
                active: true,
                query: query,
                range: { from, to },
              }
            }

            lastQuery = query
            console.log('[标签建议] #+' + query)

            // 清除之前的定时器
            if (searchTimer) {
              clearTimeout(searchTimer)
            }

            // 如果弹窗不存在，先创建
            if (!pluginState.popup) {
              pluginState.popup = createPopupDOM()
              document.body.appendChild(pluginState.popup)
            }

            // 防抖：延迟 200ms 后执行搜索
            searchTimer = setTimeout(() => {
              // 触发搜索
              const workDirectory = typeof getWorkDirectoryFn === 'function' 
                ? getWorkDirectoryFn() 
                : localStorage.getItem('workDirectory') || ''

              if (typeof workDirectory === 'object' && workDirectory.then) {
                workDirectory.then(dir => {
                  const currentPluginState = pluginKey.getState(editor.state)
                  if (currentPluginState && currentPluginState.popup) {
                    invoke('search_tags', { workDirectory: dir || '', query })
                      .then(items => {
                        currentPluginState.items = items.slice(0, 5)
                        currentPluginState.selectedIndex = 0
                        updatePopup(currentPluginState, editor)
                      })
                      .catch(err => console.error('搜索标签失败:', err))
                  }
                })
              } else {
                const currentPluginState = pluginKey.getState(editor.state)
                if (currentPluginState && currentPluginState.popup) {
                  invoke('search_tags', { workDirectory: workDirectory || '', query })
                    .then(items => {
                      currentPluginState.items = items.slice(0, 5)
                      currentPluginState.selectedIndex = 0
                      updatePopup(currentPluginState, editor)
                    })
                    .catch(err => console.error('搜索标签失败:', err))
                }
              }
            }, 200)

            return {
              ...pluginState,
              active: true,
              query: query,
              range: { from, to },
            }
          },
        },
        props: {
          handleKeyDown(view, event) {
            const pluginState = pluginKey.getState(view.state)
            
            if (!pluginState || !pluginState.active || !pluginState.popup) {
              return false
            }

            if (event.key === 'Escape') {
              destroyPopup(pluginState)
              return true
            }

            if (event.key === 'ArrowDown') {
              event.preventDefault()
              pluginState.selectedIndex = (pluginState.selectedIndex + 1) % pluginState.items.length
              updatePopup(pluginState, editor)
              return true
            }

            if (event.key === 'ArrowUp') {
              event.preventDefault()
              pluginState.selectedIndex = (pluginState.selectedIndex - 1 + pluginState.items.length) % pluginState.items.length
              updatePopup(pluginState, editor)
              return true
            }

            if (event.key === 'Enter') {
              event.preventDefault()
              selectItem(pluginState, editor)
              return true
            }

            return false
          },
        },
        view(view) {
          return {
            update(view, prevState) {
              const prevPluginState = pluginKey.getState(prevState)
              const pluginState = pluginKey.getState(view.state)

              if (!pluginState) return

              // 从活跃状态变为非活跃状态，销毁弹窗
              if (prevPluginState?.active && !pluginState.active) {
                destroyPopup(pluginState)
              }
              // 保持活跃状态，更新位置
              else if (pluginState.active) {
                updatePopupPosition(pluginState, view)
              }
            },
            destroy() {
              const pluginState = pluginKey.getState(view.state)
              if (pluginState && pluginState.popup) {
                destroyPopup(pluginState)
              }
            }
          }
        },
      }),
    ]
  },
})

function createPopupDOM() {
  const popup = document.createElement('div')
  popup.className = 'tag-suggestion-popup'
  popup.style.cssText = `
    position: fixed;
    z-index: 10000;
    display: none;
    min-width: 150px;
    max-height: 200px;
    overflow-y: auto;
  `
  popup.innerHTML = `
    <ul class="tag-suggestion-list" style="
      list-style: none;
      margin: 0;
      padding: 4px;
      background: var(--color-base-100, #ffffff);
      border: 1px solid var(--color-base-300, #e5e5e5);
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    ">
    </ul>
    <style>
      .tag-suggestion-list li {
        padding: 8px 12px;
        cursor: pointer;
        font-size: 14px;
        color: var(--color-base-content, #1f2937);
        border-radius: 4px;
        transition: background-color 0.15s ease;
      }
      .tag-suggestion-list li:hover,
      .tag-suggestion-list li.selected {
        background-color: var(--color-base-200, #f5f5f5);
      }
      .tag-suggestion-list li.selected {
        color: var(--color-primary, #3b82f6);
      }
    </style>
  `
  return popup
}

function updatePopup(pluginState, editor) {
  if (!pluginState.popup) return

  const list = pluginState.popup.querySelector('ul')
  if (list) {
    list.innerHTML = pluginState.items.map((item, index) => `
      <li data-index="${index}" class="${index === pluginState.selectedIndex ? 'selected' : ''}">
        ${item.fullName || item.name}
      </li>
    `).join('')

    list.querySelectorAll('li').forEach(li => {
      li.addEventListener('click', (e) => {
        e.preventDefault()
        e.stopPropagation()
        const index = parseInt(li.dataset.index)
        pluginState.selectedIndex = index
        selectItem(pluginState, editor)
      })
    })
  }

  // 更新位置
  if (editor && pluginState.range) {
    const view = editor.view
    if (view) {
      const coords = view.coordsAtPos(pluginState.range.from)
      pluginState.popup.style.top = `${coords.bottom + 5}px`
      pluginState.popup.style.left = `${coords.left}px`
      pluginState.popup.style.display = pluginState.items.length > 0 ? 'block' : 'none'
    }
  }
}

function updatePopupPosition(pluginState, view) {
  if (!pluginState.popup || !pluginState.range) return

  const coords = view.coordsAtPos(pluginState.range.from)
  pluginState.popup.style.top = `${coords.bottom + 5}px`
  pluginState.popup.style.left = `${coords.left}px`
  pluginState.popup.style.display = pluginState.items.length > 0 ? 'block' : 'none'
}

function destroyPopup(pluginState) {
  if (searchTimer) {
    clearTimeout(searchTimer)
    searchTimer = null
  }
  if (pluginState.popup && pluginState.popup.parentNode) {
    pluginState.popup.parentNode.removeChild(pluginState.popup)
  }
  pluginState.popup = null
  pluginState.active = false
  pluginState.items = []
  lastQuery = ''
}

function selectItem(pluginState, editor) {
  const item = pluginState.items[pluginState.selectedIndex]
  if (item && editor) {
    const { from, to } = pluginState.range
    const view = editor.view
    if (view) {
      // 插入纯文本（#标签全名），不带标签 mark
      const tagText = '#' + (item.fullName || item.name)
      const tr = view.state.tr.replaceWith(
        from,
        to,
        view.state.schema.text(tagText)
      )
      view.dispatch(tr)
      destroyPopup(pluginState)
    }
  }
}