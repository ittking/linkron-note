import { reactive, computed } from 'vue'
import { ulid } from 'ulid'

/**
 * 终端会话状态管理
 * 管理多个终端 Tab 的创建、切换和关闭
 */
const state = reactive({
  tabs: [],
  activeTabId: null,
  tabCounter: 0
})

const activeTab = computed(() =>
  state.tabs.find(tab => tab.id === state.activeTabId)
)

/**
 * 创建新的终端 Tab
 */
function createTab() {
  state.tabCounter++
  const newTab = {
    id: `terminal-${ulid()}`,
    title: `Terminal ${state.tabCounter}`,
    shell: 'powershell.exe'
  }
  state.tabs.push(newTab)
  state.activeTabId = newTab.id
  console.log('createTab: new tab created', newTab)
  console.log('createTab: activeTabId set to', state.activeTabId)
  return newTab
}

/**
 * 关闭终端 Tab
 */
function closeTab(id) {
  const index = state.tabs.findIndex(tab => tab.id === id)
  if (index > -1) {
    state.tabs.splice(index, 1)
    if (state.activeTabId === id) {
      state.activeTabId = state.tabs.length > 0
        ? state.tabs[Math.max(0, index - 1)].id
        : null
    }
  }
}

/**
 * 选择终端 Tab
 */
function selectTab(id) {
  console.log('selectTab called with id:', id, 'current activeTabId:', state.activeTabId)
  state.activeTabId = id
  console.log('selectTab completed, new activeTabId:', state.activeTabId)
}

/**
 * 更新 Tab 标题
 */
function updateTabTitle(id, title) {
  const tab = state.tabs.find(t => t.id === id)
  if (tab) tab.title = title
}

/**
 * 清空所有 Tab
 */
function clearTabs() {
  state.tabs = []
  state.activeTabId = null
  state.tabCounter = 0
}

export function useTerminalStore() {
  return {
    tabs: state.tabs,
    activeTabId: state.activeTabId,
    activeTab,
    createTab,
    closeTab,
    selectTab,
    updateTabTitle,
    clearTabs
  }
}