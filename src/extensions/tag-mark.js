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
          handleTextInput: (view, from, to, text) => {
            // 只处理空格和回车
            if (text !== ' ' && text !== '\n') {
              return false
            }

            const { state } = view
            const { tr } = state

            // 获取光标前的文本
            const $from = state.doc.resolve(from)
            const textBefore = state.doc.textBetween(
              Math.max(0, $from.start() - 50),
              from,
              ' '
            )

            // 检查是否匹配 #+内容 的模式
            // 匹配 # 后面跟着至少一个非空格字符
            const tagMatch = textBefore.match(/(#\S+)$/)

            if (tagMatch) {
              const tag = tagMatch[1]
              const tagStart = from - tag.length

              // 创建事务，将标签转换为 tag mark
              const newTr = tr
                .addMark(tagStart, from, state.schema.marks.tag.create())

              // 插入空格或回车
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