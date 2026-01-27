/**
 * URL 文件解析工具
 */

/**
 * 从 .url 文件中提取 URL
 * @param {File} file - .url 文件对象
 * @returns {Promise<string>} URL 字符串
 */
export async function extractUrlFromUrlFile(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    
    reader.onload = (e) => {
      try {
        const content = e.target.result
        
        // .url 文件格式通常是 INI 格式
        // 查找 URL= 这一行
        const urlMatch = content.match(/^URL=(.+)$/m)
        
        if (urlMatch && urlMatch[1]) {
          resolve(urlMatch[1].trim())
          return
        }
        
        // 尝试其他格式：InternetShortcut
        const internetShortcutMatch = content.match(/^\[InternetShortcut\]([\s\S]*?)^\[.*\]$/m)
        if (internetShortcutMatch) {
          const sectionContent = internetShortcutMatch[1]
          const urlLine = sectionContent.match(/^URL=(.+)$/m)
          if (urlLine && urlLine[1]) {
            resolve(urlLine[1].trim())
            return
          }
        }
        
        // 如果都找不到，尝试查找任何包含 http/https 的行
        const httpMatch = content.match(/(https?:\/\/[^\s]+)/)
        if (httpMatch) {
          resolve(httpMatch[1].trim())
        } else {
          reject(new Error('未找到 URL'))
        }
      } catch (error) {
        reject(error)
      }
    }
    
    reader.onerror = () => reject(new Error('读取文件失败'))
    reader.readAsText(file)
  })
}