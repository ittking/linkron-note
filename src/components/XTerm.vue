<template>
  <div ref="terminalContainer" class="xterm-container" @click="focusTerminal"></div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, onUnmounted } from 'vue'
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
  // 初始化xterm.js
  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Consolas, "Courier New", monospace',
    letterSpacing: 0,
    lineHeight: 1.2,
    scrollback: 1000,
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#ffffff',
      cursorAccent: '#000000',
      black: '#000000',
      red: '#cd3131',
      green: '#0dbc79',
      yellow: '#e5e510',
      blue: '#2472c8',
      magenta: '#bc3fbc',
      cyan: '#11a8cd',
      white: '#e5e5e5',
      brightBlack: '#666666',
      brightRed: '#f14c4c',
      brightGreen: '#23d18b',
      brightYellow: '#f5f543',
      brightBlue: '#3b8eea',
      brightMagenta: '#d670d6',
      brightCyan: '#29b8db',
      brightWhite: '#ffffff'
    },
    allowProposedApi: true
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

// 监听窗口大小变化
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

// 监听组件显示状态变化（用于 v-show）
watch(() => terminalContainer.value, (newVal, oldVal) => {
  if (newVal && !oldVal && fitAddon) {
    // 当组件从隐藏变为显示时，重新 fit
    console.log('XTerm component shown, refitting...')
    setTimeout(() => {
      if (fitAddon) {
        fitAddon.fit()
        console.log('XTerm refitted:', { cols: terminal.cols, rows: terminal.rows })
      }
    }, 50)
  }
})

// 监听容器大小变化
onMounted(() => {
  let resizeTimeout = null
  
  const resizeObserver = new ResizeObserver((entries) => {
    // 防抖，避免频繁触发
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    
    resizeTimeout = setTimeout(() => {
      if (fitAddon && terminal) {
        fitAddon.fit()
        // 同步到后端 PTY
        invoke('resize_pty', {
          sessionId: props.sessionId,
          cols: terminal.cols,
          rows: terminal.rows
        }).catch(err => console.error('Failed to resize PTY:', err))
      }
    }, 100)
  })
  
  if (terminalContainer.value) {
    resizeObserver.observe(terminalContainer.value)
  }
  
  // 监听窗口大小变化
  const handleResize = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    
    resizeTimeout = setTimeout(() => {
      if (fitAddon && terminal) {
        fitAddon.fit()
        invoke('resize_pty', {
          sessionId: props.sessionId,
          cols: terminal.cols,
          rows: terminal.rows
        }).catch(err => console.error('Failed to resize PTY:', err))
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
  background: #1e1e1e;
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

.xterm-container :deep(.xterm-viewport) {
  overflow-y: auto !important;
}

.xterm-container :deep(.xterm-screen) {
  padding: 0;
  height: 100%;
}

.xterm-container :deep(.xterm-rows) {
  padding: 0;
}

.xterm-container :deep(.xterm-scroll-layer) {
  height: 100% !important;
}
</style>