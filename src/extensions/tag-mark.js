import { Mark, mergeAttributes } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const TagMark = Mark.create({
  name: 'tag',

  // 标签只能包含行内内容
  inline: true,

  // 标签是可分割的（例如：#tag1#tag2 可以分割为两个标签）
  inclusive: false,

  // 定义标签的 DOM 结构
  parseHTML() {
    return [
      {
        tag: 'span.tag',
      },
    ]
  },

  // 渲染为 HTML
  renderHTML({ HTMLAttributes }) {
    return ['span', mergeAttributes(HTMLAttributes, { class: 'tag' }), 0]
  },
})

// 创建扩展来处理标签的自动检测和转换
export const TagInputRuleExtension = Mark.create({
  name: 'tagInputRule',

  addProseMirrorPlugins() {
    const pluginKey = new PluginKey('tagInputRule')

    return [
      new Plugin({
        key: pluginKey,
        props: {
          // 拦截键盘事件
          handleKeyDown: (view, event) => {
            // 不再拦截 Enter 键，允许在任何情况下换行
            // 标签只通过空格创建
            return false
          },
          // 处理文本输入
          handleTextInput: (view, from, to, text) => {
            const { state } = view
            const $from = state.doc.resolve(from)

            // 检查光标是否在标签 mark 内
            const tagMark = state.schema.marks.tag
            let isInTag = false
            let tagRange = null

            // 检查光标位置是否在标签 mark 内，并找到标签的范围
            state.doc.nodesBetween(from, to, (node, pos) => {
              if (node.marks.some(mark => mark.type === tagMark)) {
                isInTag = true
                // 找到标签的起始和结束位置
                const $pos = state.doc.resolve(pos)
                const tagMarkIndex = node.marks.findIndex(mark => mark.type === tagMark)
                if (tagMarkIndex !== -1) {
                  tagRange = {
                    from: pos,
                    to: pos + node.nodeSize
                  }
                }
                return false
              }
            })

            // 如果在标签内，检查输入的内容
            if (isInTag) {
              // 如果输入的是空格，则将空格后面的内容移出标签
              if (text === ' ') {
                const { tr } = state

                // 找到标签的结束位置
                if (tagRange) {
                  // 移除从光标位置到标签结束位置的标签 mark
                  const newTr = tr.removeMark(from, tagRange.to, tagMark)
                  // 插入空格
                  newTr.insertText(' ', from)
                  view.dispatch(newTr)
                  return true
                }
              }
              // 其他字符正常输入，不拦截
              return false
            }

            // 如果不在标签内，检查是否需要创建新标签
            // 只处理空格（不再支持回车创建标签）
            if (text !== ' ') {
              return false
            }

            // 获取光标前的文本
            const textBefore = state.doc.textBetween(
              Math.max(0, $from.start() - 50),
              from,
              ' '
            )

            // 检查是否匹配 #+内容 的模式
            // 匹配 # 后面跟着至少一个非空格字符
            const tagMatch = textBefore.match(/(#\S+)$/)

            if (tagMatch) {
              const { tr } = state
              const tag = tagMatch[1]
              const tagStart = from - tag.length

              // 创建事务，将标签转换为 tag mark
              const newTr = tr
                .addMark(tagStart, from, state.schema.marks.tag.create())

              // 插入空格
              newTr.insertText(text, from)

              view.dispatch(newTr)
              return true
            }

            return false
          },
        },
      }),
    ]
  },
})