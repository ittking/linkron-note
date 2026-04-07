/**
 * 授权登录 Store
 * 管理用户登录状态、授权码生成、轮询等逻辑
 */
import { ref, computed } from 'vue'
import { generateAuthCode, getQRCode, pollAuthStatus, getAppKey } from '@/api/auth'

// 状态
const user = ref(null)
const token = ref(null)
const isLoggedIn = computed(() => !!token.value)
const isLoading = ref(false)
const authCode = ref('')
const qrCodeData = ref(null)
const pollingTimer = ref(null)
const authStatus = ref('idle') // idle, pending, authorized, expired, error
const isNewUser = ref(false)

// 从 localStorage 恢复登录状态
function restoreAuth() {
  const savedToken = localStorage.getItem('token')
  const savedUser = localStorage.getItem('user')

  if (savedToken) {
    token.value = savedToken
  }
  if (savedUser) {
    try {
      user.value = JSON.parse(savedUser)
    } catch {
      user.value = null
    }
  }
}

// 保存登录状态到 localStorage
function saveAuth(tokenValue, userValue) {
  if (tokenValue) {
    localStorage.setItem('token', tokenValue)
    token.value = tokenValue
  }
  if (userValue) {
    localStorage.setItem('user', JSON.stringify(userValue))
    user.value = userValue
  }
}

// 清除登录状态
function clearAuth() {
  localStorage.removeItem('token')
  localStorage.removeItem('user')
  token.value = null
  user.value = null
}

/**
 * 初始化授权登录流程
 * 生成授权码并获取小程序码
 */
async function initAuth() {
  // 检查 AppKey 配置
  const appKey = getAppKey()
  if (!appKey) {
    authStatus.value = 'error'
    throw new Error('应用 AppKey 未配置')
  }

  isLoading.value = true
  authStatus.value = 'pending'

  try {
    // 生成授权码
    authCode.value = generateAuthCode()

    // 获取小程序码
    const response = await getQRCode(authCode.value, appKey)

    if (response.success) {
      qrCodeData.value = response.data
      // 开始轮询
      startPolling()
    } else {
      authStatus.value = 'error'
      throw new Error(response.message || '获取小程序码失败')
    }
  } catch (error) {
    authStatus.value = 'error'
    throw error
  } finally {
    isLoading.value = false
  }
}

/**
 * 开始轮询授权状态
 */
function startPolling() {
  stopPolling() // 先停止之前的轮询

  // 立即执行一次
  poll()

  // 每 2 秒轮询一次
  pollingTimer.value = setInterval(() => {
    poll()
  }, 2000)
}

/**
 * 停止轮询
 */
function stopPolling() {
  if (pollingTimer.value) {
    clearInterval(pollingTimer.value)
    pollingTimer.value = null
  }
}

/**
 * 轮询授权状态
 */
async function poll() {
  if (!authCode.value) return

  try {
    const response = await pollAuthStatus(authCode.value)

    if (response.success && response.data) {
      const { status, user: userData, token: tokenValue, isNewUser: isNew } = response.data

      switch (status) {
        case 'pending':
          // 继续等待
          authStatus.value = 'pending'
          break

        case 'authorized':
          // 授权成功
          authStatus.value = 'authorized'
          isNewUser.value = isNew || false
          saveAuth(tokenValue, userData)
          stopPolling()
          break

        case 'expired':
          // 授权码过期
          authStatus.value = 'expired'
          stopPolling()
          break

        default:
          authStatus.value = 'error'
          stopPolling()
      }
    }
  } catch (error) {
    console.error('轮询授权状态失败:', error)
    // 不停止轮询，可能是网络波动
  }
}

/**
 * 重置授权状态
 */
function resetAuth() {
  stopPolling()
  authCode.value = ''
  qrCodeData.value = null
  authStatus.value = 'idle'
  isNewUser.value = false
}

/**
 * 退出登录
 */
function logout() {
  clearAuth()
  resetAuth()
}

// 初始化时恢复登录状态
restoreAuth()

export function useAuthStore() {
  return {
    // 状态
    user,
    token,
    isLoggedIn,
    isLoading,
    authCode,
    qrCodeData,
    authStatus,
    isNewUser,

    // 方法
    initAuth,
    resetAuth,
    logout,
    saveAuth,
    clearAuth,
    stopPolling
  }
}
