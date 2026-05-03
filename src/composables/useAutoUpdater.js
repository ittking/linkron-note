import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { useSettingStore } from '@/store/settingStore'

const settingStore = useSettingStore()

// 更新状态
const updateAvailable = ref(false)
const latestVersion = ref(null)
const updateBody = ref(null)

// 自动检查更新定时器
let autoCheckTimer = null
const ONE_HOUR = 60 * 60 * 1000

/**
 * 检查更新（静默模式，不显示加载状态）
 */
async function checkUpdateSilent() {
  try {
    const autoUpdate = await settingStore.get('autoUpdate', true)
    if (!autoUpdate) {
      return
    }

    console.log('[AutoUpdater] Checking for updates...')
    const update = await check()

    if (update) {
      console.log('[AutoUpdater] New version available:', update.version)
      updateAvailable.value = true
      latestVersion.value = update.version
      updateBody.value = update.body
    } else {
      console.log('[AutoUpdater] No update available')
      updateAvailable.value = false
      latestVersion.value = null
      updateBody.value = null
    }
  } catch (error) {
    console.error('[AutoUpdater] Check update failed:', error)
  }
}

/**
 * 启动自动检查更新
 */
function startAutoCheck() {
  // 立即检查一次
  setTimeout(() => {
    checkUpdateSilent()
  }, 5000) // 启动后 5 秒开始检查

  // 每小时检查一次
  if (autoCheckTimer) {
    clearInterval(autoCheckTimer)
  }
  autoCheckTimer = setInterval(checkUpdateSilent, ONE_HOUR)
}

/**
 * 停止自动检查更新
 */
function stopAutoCheck() {
  if (autoCheckTimer) {
    clearInterval(autoCheckTimer)
    autoCheckTimer = null
  }
}

export function useAutoUpdater() {
  return {
    updateAvailable,
    latestVersion,
    updateBody,
    startAutoCheck,
    stopAutoCheck,
    checkUpdateSilent
  }
}
