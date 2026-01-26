<template>
  <div ref="terminalContainer" class="xterm-container" @click="focusTerminal"></div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, onUnmounted, onActivated, onDeactivated } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import 'xterm/css/xterm.css'

const props = defineProps({
  sessionId: { type: String, required: true },
  shell: { type: String, default: 'powershell.exe' },
  workingDir: { type: String, default: null }
})

const emit = defineEmits(['data'])

const terminalContainer = ref(null)
let terminal = null
let fitAddon = null
let unlisten = null

onMounted(async () => {
  // 获取主题颜色 - 使用计算后的 RGB 值
  const getComputedColor = (varName) => {
    // 创建一个临时元素来获取计算后的颜色
    const temp = document.createElement('div')
    temp.style.color = `var(${varName})`
    temp.style.display = 'none'
    document.body.appendChild(temp)
    
    const computed = getComputedStyle(temp).color
    document.body.removeChild(temp)
    
    return computed || ''
  }

  // 调色板（16色）
  const palette = [
    getComputedColor('--b1') || '#1e1e1e',      // black
    getComputedColor('--er') || '#cd3131',      // red
    getComputedColor('--su') || '#0dbc79',      // green
    getComputedColor('--wa') || '#e5e510',      // yellow
    getComputedColor('--in') || '#2472c8',      // blue
    getComputedColor('--ac') || '#bc3fbc',      // magenta
    getComputedColor('--in') || '#11a8cd',      // cyan
    getComputedColor('--bc') || '#e5e5e5',      // white
    '#666666',                                 // brightBlack
    getComputedColor('--er') || '#f14c4c',      // brightRed
    getComputedColor('--su') || '#23d18b',      // brightGreen
    getComputedColor('--wa') || '#f5f543',      // brightYellow
    getComputedColor('--in') || '#3b8eea',      // brightBlue
    getComputedColor('--ac') || '#d670d6',      // brightMagenta
    getComputedColor('--in') || '#29b8db',      // brightCyan
    '#ffffff'                                  // brightWhite
  ]

  // 初始化xterm.js
  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'JetBrains Mono, "Fira Code", "Cascadia Code", Consolas, "Courier New", monospace, "Nerd Font Symbols", "Nerd Font", "Font Awesome 6 Free", "Font Awesome 6 Brands", "Font Awesome 6 Solid", "Apple Symbols", "Segoe UI Symbol", "Segoe UI Emoji"',
    letterSpacing: 0,
    lineHeight: 1.2,
    scrollback: 1000,
    allowProposedApi: true,
    allowTransparency: true,
    colors: palette,
    theme: {
      background: 'transparent',
      foreground: getComputedColor('--bc') || '#d4d4d4',
      cursor: getComputedColor('--bc') || '#ffffff',
      cursorAccent: getComputedColor('--b1') || '#000000'
    }
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(terminalContainer.value)
  
  // 等待 DOM 更新后再 fit
  setTimeout(() => {
    if (fitAddon) {
      fitAddon.fit()
    }
  }, 100)

  // 监听用户输入
  terminal.onData((data) => {
    emit('data', data)
  })

  // 监听后端输出事件
  unlisten = await listen(`terminal-output-${props.sessionId}`, (event) => {
    terminal.write(event.payload)
  })

  // 创建后端PTY会话
  try {
    await invoke('create_pty_session', {
      sessionId: props.sessionId,
      shell: props.shell,
      cols: terminal.cols,
      rows: terminal.rows,
      workingDir: props.workingDir || null
    })
  } catch (error) {
    const errorMsg = String(error)
    terminal.write('\r\n\x1b[31m========================================\x1b[0m\r\n')
    terminal.write('\r\n\x1b[31mError: Failed to create terminal session\x1b[0m\r\n')
    terminal.write(`\r\n\x1b[31mDetails: ${errorMsg}\x1b[0m\r\n`)
    terminal.write('\r\n\x1b[31m========================================\x1b[0m\r\n')
    terminal.write('\r\nPlease check the Rust console for more details.\r\n')
  }

  // 聚焦终端
  setTimeout(() => {
    focusTerminal()
  }, 200)
})

onBeforeUnmount(async () => {
  if (unlisten) unlisten()
  if (terminal) terminal.dispose()
  if (props.sessionId) {
    await invoke('close_pty_session', { sessionId: props.sessionId })
  }
})

// 组件被 keep-alive 激活时
onActivated(() => {
  // 聚焦终端
  setTimeout(() => {
    focusTerminal()
  }, 50)
})

// 组件被 keep-alive 停用时
onDeactivated(() => {
  // 可以在这里做一些清理工作
})

// 监听 sessionId 变化
watch(() => props.sessionId, async (newId) => {
  if (terminal && fitAddon) {
    fitAddon.fit()
    await invoke('resize_pty', {
      sessionId: newId,
      cols: terminal.cols,
      rows: terminal.rows
    })
  }
})

// 监听容器大小变化
onMounted(() => {
  let resizeTimeout = null
  let lastWidth = 0
  let lastHeight = 0
  
  const resizeObserver = new ResizeObserver((entries) => {
    // 防抖，避免频繁触发
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    
    resizeTimeout = setTimeout(() => {
      if (fitAddon && terminal && terminalContainer.value) {
        const newWidth = terminalContainer.value.offsetWidth
        const newHeight = terminalContainer.value.offsetHeight
        
        // 只有当尺寸真正改变时才 fit
        if (newWidth !== lastWidth || newHeight !== lastHeight) {
          lastWidth = newWidth
          lastHeight = newHeight
          fitAddon.fit()
          // 同步到后端 PTY
          invoke('resize_pty', {
            sessionId: props.sessionId,
            cols: terminal.cols,
            rows: terminal.rows
          }).catch(err => {})
        }
      }
    }, 100)
  })
  
  if (terminalContainer.value) {
    lastWidth = terminalContainer.value.offsetWidth
    lastHeight = terminalContainer.value.offsetHeight
    resizeObserver.observe(terminalContainer.value)
  }
  
  // 监听窗口大小变化
  const handleResize = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    
    resizeTimeout = setTimeout(() => {
      if (fitAddon && terminal && terminalContainer.value) {
        const newWidth = terminalContainer.value.offsetWidth
        const newHeight = terminalContainer.value.offsetHeight
        
        // 只有当尺寸真正改变时才 fit
        if (newWidth !== lastWidth || newHeight !== lastHeight) {
          lastWidth = newWidth
          lastHeight = newHeight
          fitAddon.fit()
          invoke('resize_pty', {
            sessionId: props.sessionId,
            cols: terminal.cols,
            rows: terminal.rows
          }).catch(err => {})
        }
      }
    }, 100)
  }
  
  window.addEventListener('resize', handleResize)
  
  onUnmounted(() => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    resizeObserver.disconnect()
    window.removeEventListener('resize', handleResize)
  })
})

// 聚焦终端
const focusTerminal = () => {
  if (terminal) {
    terminal.focus()
  }
}
</script>

<style scoped>
.xterm-container {
  width: 100%;
  height: 100%;
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.xterm-container :deep(.xterm) {
  flex: 1;
  width: 100%;
  height: 100%;
  padding: 8px;
  box-sizing: border-box;
}

/* 确保字体正确加载 */
.xterm-container :deep(.xterm-viewport) {
  overflow-y: auto !important;
  overflow-x: hidden !important;
  width: 100% !important;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, 'Courier New', monospace, 'Nerd Font Symbols', 'Nerd Font', 'Font Awesome 6 Free', 'Font Awesome 6 Brands', 'Font Awesome 6 Solid', 'Apple Symbols', 'Segoe UI Symbol', 'Segoe UI Emoji', sans-serif !important;
  background-color: transparent !important;
}

/* 隐藏滚动条 */
.xterm-container :deep(.xterm-viewport)::-webkit-scrollbar {
  width: 0px !important;
  background: transparent !important;
}

.xterm-container :deep(.xterm-viewport) {
  scrollbar-width: none !important;
  -ms-overflow-style: none !important;
}

.xterm-container :deep(.xterm-screen) {
  padding: 0;
  height: 100%;
  width: 100%;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, 'Courier New', monospace, 'Nerd Font Symbols', 'Nerd Font', 'Font Awesome 6 Free', 'Font Awesome 6 Brands', 'Font Awesome 6 Solid', 'Apple Symbols', 'Segoe UI Symbol', 'Segoe UI Emoji', sans-serif !important;
}

.xterm-container :deep(.xterm-rows) {
  padding: 0;
  width: 100%;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, 'Courier New', monospace, 'Nerd Font Symbols', 'Nerd Font', 'Font Awesome 6 Free', 'Font Awesome 6 Brands', 'Font Awesome 6 Solid', 'Apple Symbols', 'Segoe UI Symbol', 'Segoe UI Emoji', sans-serif !important;
}

.xterm-container :deep(.xterm-scroll-layer) {
  height: 100% !important;
  width: 100% !important;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, 'Courier New', monospace, 'Nerd Font Symbols', 'Nerd Font', 'Font Awesome 6 Free', 'Font Awesome 6 Brands', 'Font Awesome 6 Solid', 'Apple Symbols', 'Segoe UI Symbol', 'Segoe UI Emoji', sans-serif !important;
}

.xterm-container :deep(.xterm-char-measure) {
  display: inline-block;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, 'Courier New', monospace, 'Nerd Font Symbols', 'Nerd Font', 'Font Awesome 6 Free', 'Font Awesome 6 Brands', 'Font Awesome 6 Solid', 'Apple Symbols', 'Segoe UI Symbol', 'Segoe UI Emoji', sans-serif !important;
}
</style>