import { ref } from 'vue'

/**
 * 确认对话框管理 Composable
 * 管理确认对话框状态
 */
export function useConfirmDialog() {
  const confirmVisible = ref(false)
  const confirmTitle = ref('')
  const confirmContent = ref('')
  const confirmOnOk = ref(null)

  /**
   * 显示确认对话框
   * @param {string} title - 对话框标题
   * @param {string} content - 对话框内容
   * @param {Function} onOk - 确认回调函数
   */
  function showConfirm(title, content, onOk) {
    confirmTitle.value = title
    confirmContent.value = content
    confirmOnOk.value = onOk
    confirmVisible.value = true
  }

  /**
   * 处理确认按钮点击
   */
  function handleConfirmOk() {
    if (confirmOnOk.value) {
      confirmOnOk.value()
    }
    confirmVisible.value = false
  }

  return {
    confirmVisible,
    confirmTitle,
    confirmContent,
    showConfirm,
    handleConfirmOk
  }
}
