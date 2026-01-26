<template>
  <div class="terminal-view">
    <TerminalTab
      :tabs="terminalStore.tabs"
      :activeTabId="terminalStore.activeTabId"
      @select="handleTabSelect"
      @add="addTerminal"
      @close="closeTerminal"
    />
    <div class="terminal-content">
      <!-- 调试信息 -->
      <div style="color: #666; padding: 4px; font-size: 12px;">
        activeTabId: {{ terminalStore.activeTabId }} | visibleTabId: {{ visibleTabId }}
      </div>
      
      <template v-for="tab in terminalStore.tabs" :key="tab.id">
        <XTerm
          v-if="tab.id === visibleTabId"
          :sessionId="tab.id"
          :shell="tab.shell"
          :workingDir="currentWorkingDir"
          @data="handleTerminalInput"
        />
      </template>
      <div v-if="terminalStore.tabs.length === 0" class="empty-state">
        <p>点击 + 创建新终端</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref, computed } from 'vue'
import { useTerminalStore } from '@/store/terminalStore'
import { invoke } from '@tauri-apps/api/core'
import TerminalTab from '@/components/TerminalTab.vue'
import XTerm from '@/components/XTerm.vue'

const terminalStore = useTerminalStore()
const currentWorkingDir = ref(null)

// 计算当前应该显示的 Tab
const visibleTabId = computed(() => {
  console.log('visibleTabId computed:', terminalStore.activeTabId)
  return terminalStore.activeTabId
})

// 获取当前工作目录
const getCurrentDirectory = async () => {
  try {
    const dir = await invoke('get_current_directory')
    currentWorkingDir.value = dir
    console.log('Current directory:', dir)
  } catch (error) {
    console.error('Failed to get current directory:', error)
    // 设置默认工作目录
    currentWorkingDir.value = null
  }
}

const addTerminal = () => {
  terminalStore.createTab()
}

const closeTerminal = (id) => {
  console.log('Closing terminal:', id)
  terminalStore.closeTab(id)
}

const handleTabSelect = (id) => {
  console.log('Tab selected:', id)
  terminalStore.selectTab(id)
}

const handleTerminalInput = async (data) => {
  if (terminalStore.activeTabId) {
    await invoke('write_to_pty', {
      sessionId: terminalStore.activeTabId,
      data
    })
  }
}

onMounted(async () => {
  // 获取当前工作目录
  await getCurrentDirectory()
  
  // 如果没有 Tab，则创建一个
  if (terminalStore.tabs.length === 0) {
    console.log('Creating first terminal...')
    addTerminal()
    console.log('After addTerminal, tabs:', terminalStore.tabs.length, 'activeTabId:', terminalStore.activeTabId)
  }
  
  console.log('Terminal store state:', {
    tabs: terminalStore.tabs,
    activeTabId: terminalStore.activeTabId,
    activeTab: terminalStore.activeTab
  })
  
  // 添加定时器检查状态
  setTimeout(() => {
    console.log('Terminal store state after 500ms:', {
      tabs: terminalStore.tabs,
      activeTabId: terminalStore.activeTabId,
      activeTab: terminalStore.activeTab
    })
  }, 500)
  
  // 再添加一个定时器
  setTimeout(() => {
    console.log('Terminal store state after 1000ms:', {
      tabs: terminalStore.tabs,
      activeTabId: terminalStore.activeTabId,
      activeTab: terminalStore.activeTab
    })
  }, 1000)
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