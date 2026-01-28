import { invoke } from '@tauri-apps/api/core'

export async function scrapeWebPage(url) {
  if (!isValidUrl(url)) {
    throw new Error('无效的 URL 格式')
  }

  try {
    const html = await invoke('fetch_webpage_html', { url })
    return parseWebPage(html)
  } catch (error) {
    throw new Error(`网页抓取失败: ${error.message}`)
  }
}

function parseWebPage(html) {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  const scripts = doc.querySelectorAll('script, style, noscript, iframe, [style*="display:none"], [style*="display: none"], [hidden]')
  scripts.forEach(el => el.remove())

  removeHashTags(doc)

  return {
    content: extractBodyHtml(doc),
    images: extractImages(doc)
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

function extractImages(doc) {
  const images = []
  doc.querySelectorAll('img[src]').forEach(img => {
    const src = img.getAttribute('src')
    if (src && !src.startsWith('data:')) {
      images.push(src.trim())
    }
  })
  return images
}

export function isValidUrl(string) {
  try {
    const url = new URL(string)
    return url.protocol === 'http:' || url.protocol === 'https:'
  } catch (_) {
    return false
  }
}