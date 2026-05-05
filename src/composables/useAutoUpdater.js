import { ref } from 'vue'
import { fetch } from '@tauri-apps/plugin-http'

const UPDATE_JSON_URL = 'https://github.com/ittking/linkron-note/releases/latest/download/latest.json'

// 更新状态（模块级共享）
const updateAvailable = ref(false)
const latestVersion = ref('')
const checking = ref(false)

// 自动检查更新定时器
let autoCheckTimer = null
const ONE_HOUR = 60 * 60 * 1000

function parseVersion(v) {
  return (v || '').replace(/^v/, '').split('.').map(Number)
}

function isNewer(latest, current, appVersion) {
  const a = parseVersion(latest)
  const b = parseVersion(current || appVersion)
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const an = a[i] || 0
    const bn = b[i] || 0
    if (an > bn) return true
    if (an < bn) return false
  }
  return false
}

async function doCheck(appVersion) {
  try {
    const resp = await fetch(UPDATE_JSON_URL)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const data = await resp.json()

    if (data.version && isNewer(data.version, appVersion)) {
      latestVersion.value = data.version
      updateAvailable.value = true
    } else {
      updateAvailable.value = false
      latestVersion.value = ''
    }
  } catch (error) {
    console.error('[AutoUpdater] Check update failed:', error)
  } finally {
    checking.value = false
  }
}

/**
 * 模拟更新 — 用于测试更新弹窗 UI
 * 在浏览器控制台调用: window.__mockUpdate__()
 * 取消模拟:            window.__clearMock__()
 */
if (typeof window !== 'undefined') {
  window.__mockUpdate__ = (ver = '99.9.9') => {
    latestVersion.value = ver
    updateAvailable.value = true
    console.log(`[MockUpdate] 已模拟新版本 v${ver}，弹窗应出现`)
  }
  window.__clearMock__ = () => {
    latestVersion.value = ''
    updateAvailable.value = false
    console.log('[MockUpdate] 已清除模拟状态')
  }
}

export function useAutoUpdater(appVersionRef) {
  function startAutoCheck() {
    if (!appVersionRef?.value) {
      // 如果还没拿到版本号，延迟重试
      setTimeout(() => startAutoCheck(), 2000)
      return
    }

    const check = () => {
      checking.value = true
      doCheck(appVersionRef.value)
    }

    // 启动后 5 秒检查一次
    setTimeout(check, 5000)

    // 每小时检查一次
    if (autoCheckTimer) clearInterval(autoCheckTimer)
    autoCheckTimer = setInterval(check, ONE_HOUR)
  }

  function stopAutoCheck() {
    if (autoCheckTimer) {
      clearInterval(autoCheckTimer)
      autoCheckTimer = null
    }
  }

  async function manualCheck(version) {
    checking.value = true
    await doCheck(version)
    return { updateAvailable, latestVersion, checking }
  }

  return {
    updateAvailable,
    latestVersion,
    checking,
    startAutoCheck,
    stopAutoCheck,
    manualCheck
  }
}
