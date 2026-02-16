import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'

/**
 * AI 聊天 Composable
 * 提供通用的模型调用方法
 */
export function useAIChat() {
  const settingStore = useSettingStore()

  /**
   * 获取当前激活的模型配置
   * @returns {Promise<Object>} 模型配置
   */
  async function getActiveModelConfig() {
    const providers = await settingStore.get('model.providers', [])
    const activeProviderId = await settingStore.get('model.activeProviderId', null)

    if (!activeProviderId || providers.length === 0) {
      throw new Error('请先配置模型供应商')
    }

    const activeProvider = providers.find(p => p.id === activeProviderId)
    if (!activeProvider || !activeProvider.currentModel) {
      throw new Error('请先选择模型')
    }

    return activeProvider
  }

  /**
   * 通用聊天接口
   * @param {string} prompt - 用户输入
   * @param {Object} provider - 模型供应商配置（可选，不传则使用当前激活的）
   * @returns {Promise<string>} AI 响应
   */
  async function chat(prompt, provider = null) {
    try {
      const config = provider || await getActiveModelConfig()

      const result = await invoke('chat_completion', {
        provider: config.provider,
        apiKey: config.apiKey,
        apiUrl: config.apiUrl || '',
        model: config.currentModel,
        messages: [{ role: 'user', content: prompt }],
        maxTokens: null
      })

      if (result && result.trim()) {
        return result.trim()
      } else {
        throw new Error('AI 响应为空')
      }
    } catch (error) {
      console.error('AI 调用失败:', error)
      throw error
    }
  }

  /**
   * 测试模型连接
   * @param {Object} provider - 模型供应商配置
   * @returns {Promise<{success: boolean, latency: number, message: string}>}
   */
  async function testConnection(provider) {
    const startTime = Date.now()
    
    try {
      await invoke('chat_completion', {
        provider: provider.provider,
        apiKey: provider.apiKey,
        apiUrl: provider.apiUrl || '',
        model: provider.currentModel,
        messages: [{ role: 'user', content: 'Hi' }],
        maxTokens: 1
      })

      const latency = Date.now() - startTime
      return {
        success: true,
        latency,
        message: `连接成功，延时: ${latency}ms`
      }
    } catch (error) {
      const latency = Date.now() - startTime
      return {
        success: false,
        latency,
        message: error.message || '连接失败'
      }
    }
  }

  /**
   * 生成内容（兼容旧接口）
   * @param {string} prompt - 提示词
   * @returns {Promise<string>} AI 生成的文本
   */
  async function generateContent(prompt) {
    return chat(prompt)
  }

  return {
    chat,
    testConnection,
    generateContent,
    getActiveModelConfig
  }
}