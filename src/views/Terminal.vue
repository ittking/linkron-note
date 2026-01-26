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

const addTerminal = () => {
  terminalStore.createTab()
}

const closeTerminal = (id) => {
  terminalStore.closeTab(id)
}

const handleTabSelect = (id) => {
  terminalStore.selectTab(id)
}

const handleTabRename = (id) => {
  const tab = terminalStore.tabs.value.find(t => t.id === id)
  if (tab) {
    const newTitle = prompt('请输入新的 Tab 名称:', tab.title)
    if (newTitle && newTitle.trim()) {
      terminalStore.updateTabTitle(id, newTitle.trim())
    }
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
  background: #1e1e1e;
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
  color: #888;
  font-size: 14px;
  background: #1e1e1e;
}
</style>