import { ref, watch } from 'vue'
import { useNoteStore } from '@/store/noteStore'
import { useSettingStore } from '@/store/settingStore'

/**
 * 工作目录管理 Composable
 * 封装工作目录的获取和缓存逻辑
 *
 * 支持两种获取方式：
 * 1. 从 noteStore.getWorkDirectory() 获取（用于笔记相关操作）
 * 2. 从 settingStore.get('workDirectory') 获取（用于设置相关操作）
 *
 * @param {'note' | 'setting'} source - 数据源类型，默认 'note'
 */
export function useWorkDirectory(source = 'note') {
  const workDirectoryCache = ref(null)

  let store = null
  if (source === 'note') {
    store = useNoteStore()
  } else if (source === 'setting') {
    store = useSettingStore()
  }

  /**
   * 获取工作目录（带缓存）
   * @returns {Promise<string>} 工作目录路径
   */
  async function getWorkDirectory() {
    if (workDirectoryCache.value) {
      return workDirectoryCache.value
    }

    if (source === 'note') {
      workDirectoryCache.value = await store.getWorkDirectory()
    } else if (source === 'setting') {
      workDirectoryCache.value = await store.get('workDirectory', '')
    }

    return workDirectoryCache.value
  }

  // 监听 store 变化，清空缓存
  if (store) {
    watch(() => store.$state, () => {
      workDirectoryCache.value = null
    }, { deep: true })
  }

  return {
    getWorkDirectory
  }
}
