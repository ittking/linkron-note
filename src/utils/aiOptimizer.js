import { useAIChat } from '../composables/useAIChat'
import { useSettingStore } from '../store/settingStore'

/**
 * AI 优化工具
 * 根据网址匹配提示词，调用 AI 生成优化后的文本
 */

/**
 * 优化网页内容
 * @param {string} url - 网址
 * @param {string} content - 网页内容
 * @returns {Promise<{content: string, optimized: boolean}>} 优化后的内容
 */
export async function optimizeWebContent(url, content) {
  try {
    // 检查是否启用 AI 介入优化
    const settingStore = useSettingStore()
    const aiOptimizationEnabled = await settingStore.get('aiOptimizationEnabled', false)

    if (!aiOptimizationEnabled) {
      return { content, optimized: false }
    }

    // 获取提示词列表
    const prompts = await settingStore.get('model.prompts', [])

    if (!prompts || prompts.length === 0) {
      return { content, optimized: false }
    }

    // 查找匹配的提示词
    let matchedPrompt = null
    for (const prompt of prompts) {
      if (prompt.type === 'url' && prompt.urlPattern) {
        try {
          const regex = new RegExp(prompt.urlPattern)
          if (regex.test(url)) {
            matchedPrompt = prompt
            break
          }
        } catch (e) {
          console.error('正则表达式错误:', e)
        }
      }
    }

    // 如果没有匹配到，使用系统默认提示词
    if (!matchedPrompt) {
      matchedPrompt = prompts.find(p => p.isSystem)
    }

    // 如果找到提示词，调用 AI 生成文章
    if (matchedPrompt) {
      try {
        // 替换占位符
        const promptTemplate = matchedPrompt.template.replace('{content}', content)

        // 调用 AI 生成文章
        const { generateContent } = useAIChat()
        const result = await generateContent(promptTemplate)

        return { content: result, optimized: true }
      } catch (error) {
        console.error('AI 生成失败:', error)
        throw error
      }
    }

    return { content, optimized: false }
  } catch (error) {
    console.error('优化网页内容失败:', error)
    throw error
  }
}