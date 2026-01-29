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
    isMaximized.value = false
  }

  /**
   * 设置窗口状态
   * @param {boolean} maximized - 是否最大化
   */
  async function setMaximized(maximized) {
    isMaximized.value = maximized
  }

  /**
   * 切换窗口状态
   */
  async function toggleMaximized() {
    await setMaximized(!isMaximized.value)
  }

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