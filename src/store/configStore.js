import { ref, watch } from 'vue'
import { useSettingStore } from './settingStore'

const settingStore = useSettingStore()

/**
 * 配置存储 Store
 * 专门用于应用配置状态管理
 */
export function useConfigStore() {
  const isMaximized = ref(false)

  /**
   * 初始化配置
   */
  async function initConfig() {
    try {
      const maximized = await settingStore.get('isMaximized', false)
      isMaximized.value = maximized
    } catch (error) {
      console.error('Failed to load config:', error)
    }
  }

  /**
   * 设置窗口状态
   * @param {boolean} maximized - 是否最大化
   */
  async function setMaximized(maximized) {
    isMaximized.value = maximized
    try {
      await settingStore.set('isMaximized', maximized)
    } catch (error) {
      console.error('Failed to save config:', error)
    }
  }

  /**
   * 切换窗口状态
   */
  async function toggleMaximized() {
    await setMaximized(!isMaximized.value)
  }

  // 监听状态变化并自动保存
  watch(isMaximized, async (newVal) => {
    try {
      await settingStore.set('isMaximized', newVal)
    } catch (error) {
      console.error('Failed to save config:', error)
    }
  })

  return {
    isMaximized,
    initConfig,
    setMaximized,
    toggleMaximized,
  }
}

// 创建单例实例
let configStoreInstance = null

export function useConfig() {
  if (!configStoreInstance) {
    configStoreInstance = useConfigStore()
  }
  return configStoreInstance
}