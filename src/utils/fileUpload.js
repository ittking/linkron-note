/**
 * 文件上传工具
 * 提供统一的文件上传和资源 URL 获取接口
 */

import { invoke } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'

/**
 * 保存文件到工作目录
 * @param {File} file - 文件对象
 * @param {string} type - 文件类型 ('image' | 'file')
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<string>} 文件完整 URL (http://iterm.localhost/resources/...)
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
    
    // result 现在是完整 URL: http://iterm.localhost/resources/...
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
 * @returns {Promise<string>} 图片完整 URL (http://iterm.localhost/resources/images/...)
 */
export async function saveImage(file, workDirectory) {
  return saveFile(file, 'image', workDirectory)
}

/**
 * 批量保存文件
 * @param {File[]} files - 文件数组
 * @param {string} type - 文件类型
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<string[]>} 文件完整 URL 数组
 */
export async function saveFiles(files, type = 'file', workDirectory) {
  const promises = files.map(file => saveFile(file, type, workDirectory))
  return Promise.all(promises)
}

/**
 * 删除资源文件
 * @param {string} url - 资源 URL (如: http://iterm.localhost/resources/images/1234567890-1234.png)
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<void>}
 */
export async function deleteResource(url, workDirectory) {
  try {
    await invoke('delete_resource_by_url', {
      url,
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
 * @returns {Promise<string>} 资源 URL (http://iterm.localhost/resources/...)
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
 * @param {string} protocolUrl - 协议 URL (http://iterm.localhost/resources/files/xxx.txt)
 * @param {string} workDirectory - 工作目录路径
 * @returns {Promise<void>}
 */
export async function revealFile(protocolUrl, workDirectory) {
  try {
    // 将协议 URL 转换为本地文件路径
    const localPath = await invoke('get_local_path_from_protocol', {
      protocolUrl,
      workDirectory
    })
    // 使用本地路径打开文件夹并选中文件
    await revealItemInDir(localPath)
  } catch (error) {
    console.error('显示文件失败:', error)
    throw new Error(`显示文件失败: ${error.message}`)
  }
}