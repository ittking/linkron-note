import { reactive, computed, toRefs } from 'vue'
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
 * 获取下一个可用的终端编号
 * 从 1 开始，找到第一个未被使用的编号
 */
function getNextAvailableTabNumber() {
  // 获取所有当前终端的编号
  const usedNumbers = state.tabs
    .map(tab => {
      // 从标题中提取数字（如 T1 -> 1, T2 -> 2）
      const match = tab.title.match(/^T(\d+)$/)
      return match ? parseInt(match[1], 10) : null
    })
    .filter(num => num !== null && num > 0)
    .sort((a, b) => a - b)

  // 如果没有终端，返回 1
  if (usedNumbers.length === 0) {
    return 1
  }

  // 找到第一个可用的编号（从 1 开始递增检查）
  let nextNumber = 1
  for (const usedNum of usedNumbers) {
    if (usedNum === nextNumber) {
      nextNumber++
    } else if (usedNum > nextNumber) {
      // 发现了空缺，直接使用这个编号
      return nextNumber
    }
  }

  // 所有编号都被使用了，使用下一个最大的编号
  return nextNumber
}

/**
 * 创建新的终端 Tab
 */
function createTab() {
  const tabNumber = getNextAvailableTabNumber()
  const newTab = {
    id: `terminal-${ulid()}`,
    title: `T${tabNumber}`,
    shell: 'powershell.exe'
  }
  state.tabs.push(newTab)
  state.activeTabId = newTab.id
  return newTab
}

/**
 * 关闭终端 Tab
 * @param {string} id - 要关闭的 tab ID
 * @param {Function} onCloseCallback - 关闭后的回调函数，用于执行清理操作
 */
function closeTab(id, onCloseCallback) {
  const index = state.tabs.findIndex(tab => tab.id === id)
  if (index > -1) {
    state.tabs.splice(index, 1)
    if (state.activeTabId === id) {
      state.activeTabId = state.tabs.length > 0
        ? state.tabs[Math.max(0, index - 1)].id
        : null
    }
    // 执行关闭回调（如关闭终端进程）
    if (onCloseCallback && typeof onCloseCallback === 'function') {
      onCloseCallback(id)
    }
  }
}

/**
 * 选择终端 Tab
 */
function selectTab(id) {
  state.activeTabId = id
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
    ...toRefs(state),
    activeTab,
    createTab,
    closeTab,
    selectTab,
    updateTabTitle,
    clearTabs
  }
}