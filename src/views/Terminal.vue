<template>
  <div class="terminal-view">
    <TerminalTab
      :tabs="terminalStore.tabs.value"
      :activeTabId="terminalStore.activeTabId.value"
      @select="handleTabSelect"
      @add="addTerminal"
      @close="closeTerminal"
      @rename="handleTabRename"
      @openInNewWindow="handleTabOpenInNewWindow"
    />
    <div class="terminal-content">
      <keep-alive>
        <XTerm
          v-if="visibleTabId"
          :key="visibleTabId"
          :sessionId="visibleTabId"
          :shell="terminalStore.activeTab.value?.shell"
          :workingDir="currentWorkingDir"
          @data="handleTerminalInput"
        />
      </keep-alive>
      <div v-if="terminalStore.tabs.value.length === 0" class="empty-state">
        <p>点击 + 创建新终端</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref, computed, watch } from 'vue'
import { useTerminalStore } from '@/store/terminalStore'
import { invoke } from '@tauri-apps/api/core'
import TerminalTab from '@/components/TerminalTab.vue'
import XTerm from '@/components/XTerm.vue'

const terminalStore = useTerminalStore()
const currentWorkingDir = ref(null)

// 计算当前应该显示的 Tab
const visibleTabId = computed(() => {
  return terminalStore.activeTabId.value
})

// 获取当前工作目录
const getCurrentDirectory = async () => {
  try {
    const dir = await invoke('get_current_directory')
    currentWorkingDir.value = dir
  } catch (error) {
    // 设置默认工作目录
    currentWorkingDir.value = null
  }
}

const addTerminal = async () => {
  await terminalStore.createTab()
}

const closeTerminal = async (id) => {
  // 关闭 PTY 会话
  try {
    await invoke('close_pty_session', { sessionId: id })
  } catch (error) {
    console.error('Failed to close PTY session:', error)
  }
  // 关闭 tab
  terminalStore.closeTab(id)
}

const handleTabSelect = (id) => {
  terminalStore.selectTab(id)
}

const handleTabRename = (id, newTitle) => {
  if (newTitle && newTitle.trim()) {
    terminalStore.updateTabTitle(id, newTitle.trim())
  }
}

const handleTabOpenInNewWindow = (id) => {
  // TODO: 实现新窗口打开功能
  alert('新窗口打开功能暂未实现')
}

const handleTerminalInput = async (data) => {
  if (terminalStore.activeTabId.value) {
    await invoke('write_to_pty', {
      sessionId: terminalStore.activeTabId.value,
      data
    })
  }
}

onMounted(async () => {
  // 获取当前工作目录
  await getCurrentDirectory()
  
  // 如果没有 Tab，则创建一个
  if (terminalStore.tabs.value.length === 0) {
    addTerminal()
  }
})
</script>

<style scoped>
.terminal-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: hsl(var(--b1));
  overflow: hidden;
}

.terminal-content {
  flex: 1;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: hsl(var(--bc) / 0.6);
  font-size: 14px;
  background: hsl(var(--b1));
}
</style>