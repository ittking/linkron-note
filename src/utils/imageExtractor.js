/**
 * 从 HTML 内容中提取图片链接
 * @param {string} html - HTML 内容
 * @returns {string[]} 图片链接数组
 */
export function extractImagesFromHtml(html) {
  if (!html) return []

  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')
  const images = []

  // 提取 img 标签的 src
  doc.querySelectorAll('img[src]').forEach(img => {
    const src = img.getAttribute('src')
    if (src && !src.startsWith('data:')) {
      images.push(src.trim())
    }
  })

  // 提取 TipTap Image 扩展的图片
  doc.querySelectorAll('img').forEach(img => {
    const src = img.getAttribute('src')
    if (src && !images.includes(src.trim()) && !src.startsWith('data:')) {
      images.push(src.trim())
    }
  })

  return images
}

/**
 * 将 HTML 内容中的 linkron:// 协议转换为平台特定的 URL
 * @param {string} html - HTML 内容
 * @returns {string} 转换后的 HTML 内容
 */
export async function convertImageUrlsInHtml(html) {
  if (!html) return html

  // 先将相对路径 resources/ 恢复为 linkron:// 协议 URL
  // DB 中存储的是 convertUrlsForExport 转换后的相对路径格式
  let result = html.replace(/(src=["']| )resources\//g, '$1linkron://localhost/resources/')

  const platform = await getPlatform()

  // macOS 可以直接使用 linkron:// 协议
  if (platform === 'macos') {
    return result
  }

  // Windows/Linux 替换 linkron:// 为 http://linkron.localhost/
  return result.replace(/linkron:\/\/localhost\//g, 'http://linkron.localhost/')
}

/**
 * 获取当前平台（缓存版本）
 * @returns {Promise<string>} 'windows' | 'macos' | 'linux'
 */
let cachedPlatform = null

async function getPlatform() {
  if (cachedPlatform) return cachedPlatform

  try {
    // 通过检测 UserAgent 或其他方式判断平台
    const userAgent = navigator.userAgent
    if (userAgent.includes('Mac')) {
      cachedPlatform = 'macos'
    } else if (userAgent.includes('Win')) {
      cachedPlatform = 'windows'
    } else {
      cachedPlatform = 'linux'
    }
    return cachedPlatform
  } catch {
    return 'windows'
  }
}