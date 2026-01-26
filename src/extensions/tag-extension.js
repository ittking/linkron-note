import { Node } from "@tiptap/core";
import Suggestion from "@tiptap/suggestion";

export const TagExtension = Node.create({
  name: "tag",

  group: "inline",
  inline: true,
  selectable: true,
  draggable: true,
  atom: false, // 允许编辑

  addAttributes() {
    return {
      id: {
        default: null,
      },
      name: {
        default: null,
      },
      displayName: {
        default: null,
      },
      path: {
        default: null,
      },
      level: {
        default: 1,
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="tag"]',
        getAttrs: (node) => ({
          id: node.getAttribute("data-id"),
          name: node.getAttribute("data-name"),
          displayName: node.getAttribute("data-display-name"),
          path: node.getAttribute("data-path"),
          level: parseInt(node.getAttribute("data-level") || "1", 10),
        }),
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      {
        "data-type": "tag",
        class:
          "inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors",
        "data-id": HTMLAttributes.id,
        "data-name": HTMLAttributes.name,
        "data-display-name": HTMLAttributes.displayName,
        "data-path": HTMLAttributes.path,
        "data-level": HTMLAttributes.level,
      },
      "#" + (HTMLAttributes.displayName || HTMLAttributes.name),
    ];
  },

  addNodeView() {
    return ({ node }) => {
      const span = document.createElement("span");
      span.className =
        "inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors";
      span.dataset.type = "tag";
      span.dataset.id = node.attrs.id || "";
      span.dataset.name = node.attrs.name || "";
      span.contentEditable = "false"; // 设置为 false，防止用户直接编辑标签内容
      span.textContent = "#" + (node.attrs.displayName || node.attrs.name);
      return {
        dom: span,
      };
    };
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        char: "#",
        allowedChars: "a-zA-Z0-9_\\u4e00-\\u9fa5/",
        items: this.options.suggestion?.items || (() => []),
        render: this.options.suggestion?.render || (() => ({})),
        command: ({ editor, range, props }) => {
          console.log("TagExtension command called with props:", props);
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertContent({
              type: this.name,
              attrs: props,
            })
            .insertContent(" ") // 插入一个空格，让用户可以继续输入
            .run();
        },
        // 配置空格键触发
        startOfLine: false,
      }),
    ];
  },
});
