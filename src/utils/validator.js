import { URL_REGEX, SUPPORTED_FILE_TYPES, IMAGE_TYPES } from '@/constants/regex'

/**
 * 验证 URL 是否有效
 * @param {string} url - 要验证的 URL
 * @returns {boolean} 是否有效
 */
export function isValidUrl(url) {
  try {
    const urlObj = new URL(url)
    return urlObj.protocol === 'http:' || urlObj.protocol === 'https:'
  } catch {
    return URL_REGEX.test(url)
  }
}

/**
 * 验证文件类型是否支持
 * @param {string} filename - 文件名
 * @returns {boolean} 是否支持
 */
export function isSupportedFileType(filename) {
  if (!filename) return false
  const ext = filename.toLowerCase().substring(filename.lastIndexOf('.'))
  return SUPPORTED_FILE_TYPES.includes(ext)
}

/**
 * 验证文件是否为图片
 * @param {File} file - 文件对象
 * @returns {boolean} 是否为图片
 */
export function isImageFile(file) {
  return file && file.type && IMAGE_TYPES.includes(file.type)
}

/**
 * 验证文件是否为 .url 文件
 * @param {string} filename - 文件名
 * @returns {boolean} 是否为 .url 文件
 */
export function isUrlFile(filename) {
  return filename && filename.toLowerCase().endsWith('.url')
}

/**
 * 从文件名中提取扩展名
 * @param {string} filename - 文件名
 * @returns {string} 扩展名（包含点）
 */
export function getFileExtension(filename) {
  if (!filename) return ''
  const lastDotIndex = filename.lastIndexOf('.')
  return lastDotIndex !== -1 ? filename.substring(lastDotIndex).toLowerCase() : ''
}

/**
 * 从文件路径中提取文件名
 * @param {string} filepath - 文件路径
 * @returns {string} 文件名
 */
export function getFileName(filepath) {
  if (!filepath) return ''
  const lastSlashIndex = Math.max(filepath.lastIndexOf('/'), filepath.lastIndexOf('\\'))
  return filepath.substring(lastSlashIndex + 1)
}