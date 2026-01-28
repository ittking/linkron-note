/**
 * 文件内容提取工具
 * 使用 Rust 后端处理文件内容提取，支持 txt、md 等格式
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 从文件提取文本内容
 * @param {File} file - 文件对象
 * @param {string} filePath - 已保存的文件路径（相对路径）
 * @param {string} workDirectory - 工作目录
 * @returns {Promise<string>} 提取的文本内容
 */
export async function extractTextFromFile(file, filePath, workDirectory) {
  const extension = file.name.split('.').pop().toLowerCase()
  
  // 获取文件的完整路径
  const fullPath = getFullPath(filePath, workDirectory)
  
  try {
    const text = await invoke('read_file_text', { filePath: fullPath })
    return text
  } catch (error) {
    console.error('文件内容提取失败:', error)
    throw new Error(`文件内容提取失败: ${error}`)
  }
}

/**
 * 读取文本文件（txt、md）
 * @param {string} filePath - 文件路径
 * @returns {Promise<string>} 文本内容
 */
export async function readTextFile(filePath) {
  try {
    return await invoke('read_text_file', { filePath })
  } catch (error) {
    console.error('读取文本文件失败:', error)
    throw new Error(`读取文本文件失败: ${error}`)
  }
}

/**
 * 获取文件元数据
 * @param {string} filePath - 文件路径
 * @returns {Promise<FileMetadata>} 文件元数据
 */
export async function getFileMetadata(filePath) {
  try {
    return await invoke('get_file_metadata', { filePath })
  } catch (error) {
    console.error('获取文件元数据失败:', error)
    throw new Error(`获取文件元数据失败: ${error}`)
  }
}

/**
 * 获取文件的完整路径
 * @param {string} relativePath - 相对路径
 * @param {string} workDirectory - 工作目录
 * @returns {string} 完整路径
 */
function getFullPath(filePath, workDirectory) {
  // 如果是完整 URL (http://iterm.localhost/resources/...)，提取相对路径
  if (filePath.startsWith('http://iterm.localhost/resources/')) {
    const relativePath = filePath.replace('http://iterm.localhost/resources/', '')
    const separator = workDirectory.endsWith('/') || workDirectory.endsWith('\\') ? '' : '/'
    return `${workDirectory}${separator}resources/${relativePath}`
  }
  
  // 如果是绝对路径，直接返回
  if (filePath.startsWith('/') || filePath.match(/^[A-Za-z]:\\/)) {
    return filePath
  }
  
  // 否则拼接工作目录（相对路径格式）
  const separator = workDirectory.endsWith('/') || workDirectory.endsWith('\\') ? '' : '/'
  return `${workDirectory}${separator}${filePath}`
}

/**
 * 检查文件类型是否支持
 * @param {string} fileName - 文件名
 * @returns {boolean} 是否支持
 */
export function isSupportedFileType(fileName) {
  const extension = fileName.split('.').pop().toLowerCase()
  const supportedExtensions = ['txt', 'md', 'markdown']
  return supportedExtensions.includes(extension)
}

/**
 * 获取文件类型描述
 * @param {string} fileName - 文件名
 * @returns {string} 文件类型描述
 */
export function getFileTypeDescription(fileName) {
  const extension = fileName.split('.').pop().toLowerCase()
  
  const typeMap = {
    'txt': '文本文件',
    'md': 'Markdown 文档',
    'markdown': 'Markdown 文档'
  }
  
  return typeMap[extension] || '未知文件类型'
}