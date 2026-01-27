/**
 * 网页抓取工具
 * 提供网页元数据提取和内容抓取功能
 */

/**
 * 抓取网页信息
 * @param {string} url - 网页 URL
 * @returns {Promise<WebPageInfo>} 网页信息对象
 */
export async function scrapeWebPage(url) {
  if (!isValidUrl(url)) {
    throw new Error('无效的 URL 格式')
  }

  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
        'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
      },
    })

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const html = await response.text()
    return parseWebPage(html)
  } catch (error) {
    console.error('网页抓取失败:', error)
    throw new Error(`网页抓取失败: ${error.message}`)
  }
}

/**
 * 解析网页 HTML 提取元数据
 * @param {string} html - HTML 内容
 * @returns {WebPageInfo} 网页信息对象
 */
function parseWebPage(html) {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  // 提取基础元数据
  const title = doc.querySelector('title')?.textContent?.trim() || ''
  const description = doc.querySelector('meta[name="description"]')?.content?.trim() || ''
  const keywords = doc.querySelector('meta[name="keywords"]')?.content?.trim() || ''

  // 提取 Open Graph 标签
  const ogTitle = doc.querySelector('meta[property="og:title"]')?.content?.trim() || title
  const ogDescription = doc.querySelector('meta[property="og:description"]')?.content?.trim() || description
  const ogImage = doc.querySelector('meta[property="og:image"]')?.content?.trim() || ''
  const ogType = doc.querySelector('meta[property="og:type"]')?.content?.trim() || 'website'
  const ogSiteName = doc.querySelector('meta[property="og:site_name"]')?.content?.trim() || ''

  // 提取 Twitter Card 标签
  const twitterTitle = doc.querySelector('meta[name="twitter:title"]')?.content?.trim() || ogTitle
  const twitterDescription = doc.querySelector('meta[name="twitter:description"]')?.content?.trim() || ogDescription
  const twitterImage = doc.querySelector('meta[name="twitter:image"]')?.content?.trim() || ogImage

  // 提取正文内容（移除脚本和样式）
  const bodyText = extractBodyText(doc)

  // 提取链接
  const links = extractLinks(doc)

  // 提取图片
  const images = extractImages(doc)

  return {
    title: ogTitle,
    description: ogDescription,
    ogTitle,
    ogDescription,
    ogImage,
    ogType,
    ogSiteName,
    twitterTitle,
    twitterDescription,
    twitterImage,
    bodyText: bodyText.substring(0, 2000), // 限制正文长度
    links: links.slice(0, 10), // 限制链接数量
    images: images.slice(0, 5), // 限制图片数量
    metadata: {
      keywords,
      charset: doc.characterSet || 'UTF-8',
      language: doc.documentElement.lang || 'en'
    }
  }
}

/**
 * 提取网页正文文本
 * @param {Document} doc - DOM 文档对象
 * @returns {string} 正文文本
 */
function extractBodyText(doc) {
  // 移除脚本和样式标签
  const scripts = doc.querySelectorAll('script, style, noscript, iframe')
  scripts.forEach(el => el.remove())

  // 移除隐藏元素
  const hiddenElements = doc.querySelectorAll('[style*="display:none"], [style*="display: none"], [hidden]')
  hiddenElements.forEach(el => el.remove())

  // 提取文本
  const bodyText = doc.body?.textContent?.trim() || ''
  
  // 移除多余空白
  return bodyText.replace(/\s+/g, ' ').substring(0, 2000)
}

/**
 * 提取页面中的链接
 * @param {Document} doc - DOM 文档对象
 * @returns {string[]} 链接数组
 */
function extractLinks(doc) {
  const links = []
  const anchorElements = doc.querySelectorAll('a[href]')

  anchorElements.forEach(a => {
    const href = a.getAttribute('href')
    if (href && !href.startsWith('#') && !href.startsWith('javascript:')) {
      links.push(href.trim())
    }
  })

  return links
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

/**
 * 从 URL 提取域名
 * @param {string} url - URL 字符串
 * @returns {string} 域名
 */
export function extractDomain(url) {
  try {
    const urlObj = new URL(url)
    return urlObj.hostname
  } catch (_) {
    return url
  }
}

/**
 * 格式化网页信息为笔记内容
 * @param {WebPageInfo} pageInfo - 网页信息对象
 * @returns {string} 格式化的 HTML 内容
 */
export function formatWebPageToNote(pageInfo) {
  const parts = []

  // 标题
  if (pageInfo.title) {
    parts.push(`<h3>${pageInfo.title}</h3>`)
  }

  // 描述
  if (pageInfo.description) {
    parts.push(`<p>${pageInfo.description}</p>`)
  }

  // 正文摘要
  if (pageInfo.bodyText) {
    parts.push(`<p>${pageInfo.bodyText}</p>`)
  }

  // 分隔线
  parts.push('<hr>')

  // 来源链接
  parts.push(`<p><small>来源：${pageInfo.title}</small></p>`)

  return parts.join('')
}