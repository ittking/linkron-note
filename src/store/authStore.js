/**
 * 授权登录 Store
 * 只管理用户登录后的状态
 */
import { ref, computed } from 'vue'

// 状态
const user = ref(null)
const token = ref(null)
const isLoggedIn = computed(() => !!token.value)

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
 * 退出登录
 */
function logout() {
  clearAuth()
}

// 初始化时恢复登录状态
restoreAuth()

export function useAuthStore() {
  return {
    // 状态
    user,
    token,
    isLoggedIn,

    // 方法
    saveAuth,
    clearAuth,
    logout
  }
}
