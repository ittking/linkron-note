import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'

// 创建全局单例的同步状态
const isSyncing = ref(false)
const lastSyncTime = ref(null)
let syncTimer = null
let syncPending = false

/**
 * 云同步管理 Composable
 * 提供自动同步功能和同步状态管理（全局单例）
 */
export function useSync() {
  const settingStore = useSettingStore()

  /**
   * 加载上次同步时间
   */
  async function loadSyncTime() {
    try {
      const syncTime = await settingStore.get('lastSyncTime', null)
      if (syncTime) {
        lastSyncTime.value = syncTime
      }
    } catch (error) {
      console.error('Failed to load sync time:', error)
    }
  }

  /**
   * 格式化同步时间
   */
  const formattedLastSyncTime = computed(() => {
    if (!lastSyncTime.value || lastSyncTime.value === 0) return ''
    const date = new Date(lastSyncTime.value * 1000)
    if (isNaN(date.getTime())) return ''
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  })

  /**
   * 检查是否配置了云同步
   */
  async function isSyncConfigured() {
    try {
      const config = await settingStore.get('syncConfig', null)
      return config && config.repo_url && config.token
    } catch (error) {
      return false
    }
  }

  /**
   * 触发同步（使用防抖，避免频繁同步）
   */
  async function triggerSync() {
    // 检查是否配置了云同步
    const configured = await isSyncConfigured()
    if (!configured) {
      return
    }

    // 获取自动同步延时配置（0 表示关闭自动同步），默认 5000ms（5秒）
    const autoSyncDelay = await settingStore.get('autoSyncDelay', 5000)
    if (autoSyncDelay === 0) {
      // 自动同步已关闭
      return
    }

    // 标记有待处理的同步
    syncPending = true

    // 清除之前的定时器
    if (syncTimer) {
      clearTimeout(syncTimer)
    }

    // 延迟后执行同步（防抖）
    syncTimer = setTimeout(async () => {
      if (!syncPending || isSyncing.value) {
        return
      }

      syncPending = false
      await performSync()
    }, autoSyncDelay)
  }

  /**
   * 执行同步
   */
  async function performSync() {
    const configured = await isSyncConfigured()
    if (!configured) {
      return
    }

    if (isSyncing.value) {
      return
    }

    isSyncing.value = true

    try {
      const config = await settingStore.get('syncConfig')
      const workDirectory = await settingStore.get('workDirectory')

      const result = await invoke('sync_to_remote', {
        config,
        workDirectory
      })

      if (result.success) {
        // 更新同步时间
        lastSyncTime.value = Math.floor(Date.now() / 1000)
        await settingStore.set('lastSyncTime', lastSyncTime.value)
        console.log('[Sync] Auto-sync success')
      } else {
        console.error('[Sync] Auto-sync failed:', result.message)
      }
    } catch (error) {
      console.error('[Sync] Auto-sync error:', error)
    } finally {
      isSyncing.value = false
    }
  }

  /**
   * 立即同步（不使用防抖）
   */
  async function syncNow() {
    // 清除待处理的同步
    syncPending = false
    if (syncTimer) {
      clearTimeout(syncTimer)
      syncTimer = null
    }

    await performSync()
  }

  return {
    isSyncing,
    lastSyncTime,
    formattedLastSyncTime,
    loadSyncTime,
    triggerSync,
    syncNow
  }
}
