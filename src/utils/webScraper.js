/**
 * 网页抓取工具
 * 使用后端 Rust 抓取网页 HTML 内容，前端负责解析
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 抓取网页信息
 * @param {string} url - 网页 URL
 * @returns {Promise<{content: string, images: string[]}>} 包含 HTML 内容和图片数组的对象
 */
export async function scrapeWebPage(url) {
  if (!isValidUrl(url)) {
    throw new Error('无效的 URL 格式')
  }

  try {
    // 使用后端 Rust 抓取网页 HTML
    const html = await invoke('fetch_webpage_html', { url })
    return parseWebPage(html)
  } catch (error) {
    console.error('网页抓取失败:', error)
    throw new Error(`网页抓取失败: ${error.message}`)
  }
}

/**
 * 解析网页 HTML 提取内容和图片
 * @param {string} html - HTML 内容
 * @returns {{content: string, images: string[]}} 包含 HTML 内容和图片数组的对象
 */
function parseWebPage(html) {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  // 移除脚本、样式、隐藏元素等不需要的内容
  const scripts = doc.querySelectorAll('script, style, noscript, iframe, [style*="display:none"], [style*="display: none"], [hidden]')
  scripts.forEach(el => el.remove())

  // 移除所有 #标签名 格式的内容
  removeHashTags(doc)

  // 提取图片
  const images = extractImages(doc)

  // 提取 HTML 内容
  const content = extractBodyHtml(doc)

  return {
    content,
    images
  }
}

/**
 * 移除文档中所有 #标签名 格式的文本内容
 * @param {Document} doc - DOM 文档对象
 */
function removeHashTags(doc) {
  // 使用 TreeWalker 遍历所有文本节点
  const walker = doc.createTreeWalker(
    doc.body,
    NodeFilter.SHOW_TEXT,
    null,
    false
  )

  const textNodes = []

  let node
  while (node = walker.nextNode()) {
    textNodes.push(node)
  }

  // 移除包含标签的文本节点
  textNodes.forEach(node => {
    const text = node.textContent
    if (text && /#[a-zA-Z0-9_\u4e00-\u9fa5/]+/.test(text)) {
      node.textContent = text.replace(/#[a-zA-Z0-9_\u4e00-\u9fa5/]+/g, '')
    }
  })
}

/**
 * 提取网页正文 HTML
 * @param {Document} doc - DOM 文档对象
 * @returns {string} HTML 内容
 */
function extractBodyHtml(doc) {
  // 获取 body 的 HTML 内容
  const bodyHtml = doc.body?.innerHTML?.trim() || ''
  
  // 清理多余的空白
  return bodyHtml.replace(/\s+/g, ' ')
}

/**
 * 提取页面中的图片
 * @param {Document} doc - DOM 文档对象
 * @returns {string[]} 图片 URL 数组
 */
function extractImages(doc) {
  const images = []
  const imgElements = doc.querySelectorAll('img[src]')

  imgElements.forEach(img => {
    const src = img.getAttribute('src')
    if (src && !src.startsWith('data:')) {
      images.push(src.trim())
    }
  })

  return images
}

/**
 * 验证 URL 格式
 * @param {string} string - 待验证的字符串
 * @returns {boolean} 是否为有效 URL
 */
export function isValidUrl(string) {
  try {
    const url = new URL(string)
    return url.protocol === 'http:' || url.protocol === 'https:'
  } catch (_) {
    return false
  }
}