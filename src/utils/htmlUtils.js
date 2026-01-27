/**
 * HTML 工具函数
 */

/**
 * 解码 HTML 实体
 * @param {string} str - 包含 HTML 实体的字符串
 * @returns {string} 解码后的字符串
 */
export function decodeHtmlEntities(str) {
  const textarea = document.createElement('textarea')
  textarea.innerHTML = str
  return textarea.value
}

/**
 * 从 HTML 内容中提取图片 URL
 * @param {string} htmlContent - HTML 内容
 * @returns {string[]} 图片 URL 数组
 */
export function extractImagesFromContent(htmlContent) {
  const imgRegex = /<img[^>]+src="([^"]+)"/g
  const extractedImages = []
  let match
  while ((match = imgRegex.exec(htmlContent)) !== null) {
    extractedImages.push(match[1])
  }
  return extractedImages
}