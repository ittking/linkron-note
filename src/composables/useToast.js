import { ref } from 'vue'

// 创建全局单例的 toast 状态
const toastVisible = ref(false)
const toastMessage = ref('')
const toastType = ref('info')

let toastTimeout = null

/**
 * Toast 提示管理 Composable
 * 统一管理 Toast 提示状态（全局单例）
 */
export function useToast() {
  /**
   * 显示 Toast 提示
   * @param {string} message - 提示消息
   * @param {'success' | 'error' | 'info'} type - 提示类型
   */
  function showToast(message, type = 'info') {
    console.log('Toast:', { message, type, toastVisible: toastVisible.value })
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true
    
    // 清除之前的定时器
    if (toastTimeout) {
      clearTimeout(toastTimeout)
    }
    
    // 3秒后自动隐藏
    toastTimeout = setTimeout(() => {
      toastVisible.value = false
    }, 3000)
  }

  return {
    toastVisible,
    toastMessage,
    toastType,
    showToast
  }
}
