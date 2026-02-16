import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'

/**
 * AI 聊天 Composable
 * 提供通用的模型调用方法
 */
const DEFAULT_TIMEOUT = 60 // 默认超时时间（秒）

export function useAIChat() {
  const settingStore = useSettingStore()
  let currentRequestAbort = false // 当前请求是否已取消
  let currentRequestId = 0 // 请求ID，用于区分请求

  /**
   * 获取超时配置
   * @returns {Promise<number>} 超时时间（秒）
   */
  async function getTimeout() {
    const timeout = await settingStore.get('model.timeout', DEFAULT_TIMEOUT)
    return Math.min(timeout, 60) // 限制最大60秒
  }

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
   * @param {Object} options - 配置选项
   * @param {Object} options.provider - 模型供应商配置（可选，不传则使用当前激活的）
   * @param {number} options.timeout - 超时时间（秒），默认从设置读取
   * @returns {Promise<{result: string, cancel: Function}>} AI 响应和取消函数
   */
  async function chat(prompt, options = {}) {
    const { provider: providerConfig, timeout: customTimeout } = options

    // 生成新的请求ID
    currentRequestId++
    const requestId = currentRequestId
    currentRequestAbort = false

    try {
      const config = providerConfig || await getActiveModelConfig()
      const timeout = customTimeout !== undefined ? customTimeout : await getTimeout()

      // 检查是否已取消
      if (currentRequestAbort || requestId !== currentRequestId) {
        throw new Error('请求已取消')
      }

      const result = await invoke('chat_completion', {
        provider: config.provider,
        apiKey: config.apiKey,
        apiUrl: config.apiUrl || '',
        model: config.currentModel,
        messages: [{ role: 'user', content: prompt }],
        maxTokens: null,
        timeout: timeout
      })

      // 再次检查是否已取消
      if (currentRequestAbort || requestId !== currentRequestId) {
        throw new Error('请求已取消')
      }

      if (result && result.trim()) {
        return {
          result: result.trim(),
          cancel: () => {
            if (requestId === currentRequestId) {
              currentRequestAbort = true
            }
          }
        }
      } else {
        throw new Error('AI 响应为空')
      }
    } catch (error) {
      if (error.message === '请求已取消') {
        throw error
      }
      console.error('AI 调用失败:', error)
      throw error
    }
  }

  /**
   * 测试模型连接
   * @param {Object} provider - 模型供应商配置
   * @param {number} timeout - 超时时间（秒），默认30秒
   * @returns {Promise<{success: boolean, latency: number, message: string}>}
   */
  async function testConnection(provider, timeout = 30) {
    const startTime = Date.now()

    try {
      await invoke('chat_completion', {
        provider: provider.provider,
        apiKey: provider.apiKey,
        apiUrl: provider.apiUrl || '',
        model: provider.currentModel,
        messages: [{ role: 'user', content: 'Hi' }],
        maxTokens: 1,
        timeout: timeout
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
   * 取消当前请求
   */
  function cancelRequest() {
    currentRequestAbort = true
  }

  /**
   * 生成内容（兼容旧接口）
   * @param {string} prompt - 提示词
   * @returns {Promise<string>} AI 生成的文本
   */
  async function generateContent(prompt) {
    const { result } = await chat(prompt)
    return result
  }

  return {
    chat,
    testConnection,
    generateContent,
    getActiveModelConfig,
    cancelRequest,
    getTimeout
  }
}