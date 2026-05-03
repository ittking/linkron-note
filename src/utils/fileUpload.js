/**
 * 文件上传工具
 * 提供统一的文件上传和资源 URL 获取接口
 *
 * 协议说明：
 * - 数据库统一存储为 linkron://localhost/resources/... 格式
 * - 前端渲染时根据平台转换为可用协议
 * - 同步导出时转换为 resources/... 相对路径
 * - 同步导入时转换回 linkron://localhost/resources/... 格式
 */

import { invoke } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'

/**
 * 平台检测缓存
 */
let currentPlatform = null

/**
 * 获取当前平台
 * @returns {Promise<string>} 'windows' | 'macos' | 'linux'
 */
async function getPlatform() {
  if (currentPlatform) return currentPlatform

  try {
    const platform = await invoke('get_os')
    currentPlatform = platform.toLowerCase()
    return currentPlatform
  } catch {
    // 如果无法获取平台，默认返回 windows
    return 'windows'
  }
}

/**
 * 将 linkron:// 协议转换为平台特定的 URL
 * @param {string} url - 原始 URL (linkron://localhost/resources/...)
 * @returns {Promise<string>} 平台特定的 URL
 *
 * 规则：
 * - macOS: 保持 linkron://localhost/resources/...
 * - Windows/Linux: 转换为 http://linkron.localhost/resources/...
 */
export async function convertResourceUrl(url) {
  // 如果不是 linkron:// 协议，直接返回
  if (!url || !url.startsWith('linkron://localhost/resources/')) {
    return url
  }

  const platform = await getPlatform()

  // macOS 可以使用 linkron:// 协议
  if (platform === 'macos') {
    return url
  }

  // Windows/Linux 使用 http 协议
  return url.replace('linkron://localhost/', 'http://linkron.localhost/')
}

/**
 * 批量转换资源 URL
 * @param {string[]} urls - URL 数组
 * @returns {Promise<string[]>} 转换后的 URL 数组
 */
export async function convertResourceUrls(urls) {
  return Promise.all(urls.map(url => convertResourceUrl(url)))
}

/**
 * 处理 HTML 内容中的资源 URL（用于笔记内容渲染）
 * @param {string} html - HTML 内容
 * @returns {Promise<string>} 处理后的 HTML 内容
 */
export async function processHtmlResourceUrls(html) {
  if (!html) return html

  const platform = await getPlatform()

  // macOS 不需要转换
  if (platform === 'macos') {
    return html
  }

  // Windows/Linux 替换 linkron:// 为 http://linkron.localhost/
  return html.replace(/linkron:\/\/localhost\//g, 'http://linkron.localhost/')
}

/**
 * 保存文件到工作目录
 * @param {File} file - 文件对象
 * @param {string} type - 文件类型 ('image' | 'file')
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<string>} 文件完整 URL (linkron://localhost/resources/...)
 */
export async function saveFile(file, type = 'file', workDirectory) {
  try {
    const arrayBuffer = await file.arrayBuffer()
    const uint8Array = new Uint8Array(arrayBuffer)
    const dataArray = Array.from(uint8Array)
    
    // 直接传递原始文件名，Rust 端会使用时间戳+随机数生成新文件名
    const fileName = file.name
    
    const result = await invoke('save_file', {
      fileName,
      fileData: dataArray,
      fileType: type,
      workDirectory
    })

    // result 现在是完整 URL: linkron://localhost/resources/...
    return result
  } catch (error) {
    console.error('文件保存失败:', error)
    throw new Error(`文件保存失败: ${error.message}`)
  }
}

/**
 * 保存图片文件（向后兼容）
 * @param {File} file - 图片文件对象
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<string>} 图片完整 URL (linkron://localhost/resources/images/...)
 */
export async function saveImage(file, workDirectory) {
  return saveFile(file, 'image', workDirectory)
}

/**
 * 批量保存文件
 * @param {File[]} files - 文件数组
 * @param {string} type - 文件类型
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<string[]>} 文件完整 URL 数组 (linkron://localhost/resources/...)
 */
export async function saveFiles(files, type = 'file', workDirectory) {
  const promises = files.map(file => saveFile(file, type, workDirectory))
  return Promise.all(promises)
}

/**
 * 删除资源文件
 * @param {string} url - 资源 URL (如: linkron://localhost/resources/images/xxx.png 或 http://linkron.localhost/resources/images/xxx.png)
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<void>}
 */
export async function deleteResource(url, workDirectory) {
  try {
    // 如果是 http://linkron.localhost/ 格式，转换为 linkron://localhost/ 格式
    let targetUrl = url
    if (url && url.startsWith('http://linkron.localhost/')) {
      targetUrl = url.replace('http://linkron.localhost/', 'linkron://localhost/')
    }

    await invoke('delete_resource_by_url', {
      url: targetUrl,
      workDirectory
    })
  } catch (error) {
    console.error('文件删除失败:', error)
    throw new Error(`文件删除失败: ${error.message}`)
  }
}

/**
 * 批量删除资源文件
 * @param {string[]} urls - 资源 URL 数组
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<void>}
 */
export async function deleteResources(urls, workDirectory) {
  const promises = urls.map(url => deleteResource(url, workDirectory))
  await Promise.all(promises)
}

/**
 * 获取资源 URL（已废弃，后端直接返回完整 URL）
 * @param {string} relativePath - 文件相对路径
 * @returns {Promise<string>} 资源 URL (linkron://localhost/resources/...)
 * @deprecated 后端 save_file/save_image 现在直接返回完整 URL，不再需要此方法
 */
export async function getResourceUrl(relativePath) {
  try {
    return await invoke('get_resource_url', { relativePath })
  } catch (error) {
    console.error('获取资源 URL 失败:', error)
    throw new Error(`获取资源 URL 失败: ${error.message}`)
  }
}

/**
 * 在文件夹中显示文件
 * @param {string} protocolUrl - 协议 URL (linkron://localhost/resources/files/xxx.txt 或 http://linkron.localhost/resources/files/xxx.txt)
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<void>}
 */
export async function revealFile(protocolUrl, workDirectory) {
  try {
    // 如果是 http://linkron.localhost/ 格式，转换为 linkron://localhost/ 格式
    let targetUrl = protocolUrl
    if (protocolUrl && protocolUrl.startsWith('http://linkron.localhost/')) {
      targetUrl = protocolUrl.replace('http://linkron.localhost/', 'linkron://localhost/')
    }

    // 将协议 URL 转换为本地文件路径
    const localPath = await invoke('get_local_path_from_protocol', {
      protocolUrl: targetUrl,
      workDirectory
    })
    // 使用本地路径打开文件夹并选中文件
    await revealItemInDir(localPath)
  } catch (error) {
    console.error('显示文件失败:', error)
    throw new Error(`显示文件失败: ${error.message}`)
  }
}

/**
 * 导出转换：将存储的 linkron:// 协议 URL 转换为相对路径（用于同步到 git）
 * @param {string} content - 笔记 HTML 内容
 * @returns {string} 转换后的内容
 */
export function convertUrlsForExport(content) {
  if (!content) return content
  // 统一转为相对路径 resources/xxx（同时处理 linkron:// 和 http://linkron.localhost/ 两种格式）
  return content
    .replace(/linkron:\/\/localhost\/resources\//g, 'resources/')
    .replace(/http:\/\/linkron\.localhost\/resources\//g, 'resources/')
}

/**
 * 导入转换：将相对路径转换为 linkron:// 协议 URL（从 git 同步导入）
 * @param {string} content - 笔记 HTML 内容
 * @returns {string} 转换后的内容
 */
export function convertUrlsFromImport(content) {
  if (!content) return content
  // resources/xxx → linkron://localhost/resources/xxx
  // 匹配 src="resources/ 或 空格 resources/ 确保只匹配完整路径
  return content.replace(/(src=["']| )resources\//g, '$1linkron://localhost/resources/')
}
