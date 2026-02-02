/**
 * 配置存储 Store
 * 专门用于应用配置状态管理
 */
export function useConfigStore() {
  const noteImageMaxCount = ref(4)

  /**
   * 初始化配置
   */
  async function initConfig() {
    noteImageMaxCount.value = 4
  }

  /**
   * 设置笔记图片最大展示数
   * @param {number} count - 最大展示数量
   */
  async function setNoteImageMaxCount(count) {
    noteImageMaxCount.value = count
  }

  return {
    noteImageMaxCount,
    initConfig,
    setNoteImageMaxCount,
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