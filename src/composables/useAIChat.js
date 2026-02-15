import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'

/**
 * AI 聊天 Composable
 * 提供与大模型对话的公共方法
 */
export function useAIChat() {
  const settingStore = useSettingStore()

  /**
   * 调用 AI 生成内容
   * @param {string} prompt - 提示词
   * @returns {Promise<string>} AI 生成的文本
   */
  async function generateContent(prompt) {
    try {
      // 获取当前激活的模型配置
      const providers = await settingStore.get('model.providers', [])
      const activeProviderId = await settingStore.get('model.activeProviderId', null)

      if (!activeProviderId || providers.length === 0) {
        throw new Error('请先配置模型供应商')
      }

      const activeProvider = providers.find(p => p.id === activeProviderId)
      if (!activeProvider || !activeProvider.currentModel) {
        throw new Error('请先选择模型')
      }

      // 调用 AI 生成内容
      const result = await invoke('generate_regex', {
        prompt: prompt,
        provider: activeProvider.provider,
        apiKey: activeProvider.apiKey,
        apiUrl: activeProvider.apiUrl,
        model: activeProvider.currentModel
      })

      if (result && result.trim()) {
        return result.trim()
      } else {
        throw new Error('AI 生成内容为空')
      }
    } catch (error) {
      console.error('AI 生成失败:', error)
      throw error
    }
  }

  return {
    generateContent
  }
}