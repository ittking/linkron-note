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