import { invoke } from '@tauri-apps/api/core'
import { isValidUrl } from './validator'

export async function scrapeWebPage(input) {
  let url = input

  // 检测是否为 .url 文件
  if (input.toLowerCase().endsWith('.url')) {
    url = await extractUrlFromUrlFile(input)
  } else if (!isValidUrl(input)) {
    throw new Error('无效的输入格式，请提供 .url 文件或有效的 HTTPS 链接')
  }

  try {
    const html = await invoke('fetch_webpage_html', { url })
    return parseWebPage(html)
  } catch (error) {
    throw new Error(`网页抓取失败: ${error.message}`)
  }
}

/**
 * 从 .url 文件中提取 URL
 * @param {string} filePath - .url 文件路径
 * @returns {Promise<string>} 提取的 URL
 */
async function extractUrlFromUrlFile(filePath) {
  try {
    const content = await invoke('read_text_file', { filePath })
    const urlMatch = content.match(/^URL=(.+)$/m)
    
    if (urlMatch && urlMatch[1]) {
      const url = urlMatch[1].trim()
      if (!isValidUrl(url)) {
        throw new Error('.url 文件中包含无效的 URL')
      }
      return url
    } else {
      throw new Error('.url 文件格式不正确，未找到 URL')
    }
  } catch (error) {
    throw new Error(`读取 .url 文件失败: ${error.message}`)
  }
}

function parseWebPage(html) {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  const scripts = doc.querySelectorAll('script, style, noscript, iframe, [style*="display:none"], [style*="display: none"], [hidden]')
  scripts.forEach(el => el.remove())

  removeHashTags(doc)

  return {
    content: extractBodyHtml(doc)
  }
}

function removeHashTags(doc) {
  const walker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT, null, false)
  const textNodes = []
  let node
  while (node = walker.nextNode()) {
    textNodes.push(node)
  }
  textNodes.forEach(node => {
    const text = node.textContent
    if (text && /#[a-zA-Z0-9_\u4e00-\u9fa5/]+/.test(text)) {
      node.textContent = text.replace(/#[a-zA-Z0-9_\u4e00-\u9fa5/]+/g, '')
    }
  })
}

function extractBodyHtml(doc) {
  return doc.body?.innerHTML?.trim()?.replace(/\s+/g, ' ') || ''
}