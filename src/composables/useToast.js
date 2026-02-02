import { ref } from 'vue'

/**
 * Toast 提示管理 Composable
 * 统一管理 Toast 提示状态
 */
export function useToast() {
  const toastVisible = ref(false)
  const toastMessage = ref('')
  const toastType = ref('info')

  /**
   * 显示 Toast 提示
   * @param {string} message - 提示消息
   * @param {'success' | 'error' | 'info'} type - 提示类型
   */
  function showToast(message, type = 'info') {
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true
    setTimeout(() => {
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
