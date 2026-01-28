/**
 * URL 正则表达式
 * 支持 http/https 协议，支持查询参数
 */
export const URL_REGEX = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.\-?=&%]*)*\/?$/

/**
 * .url 文件 URL 提取正则
 */
export const URL_FILE_REGEX = /^URL=(.+)$/m

/**
 * 文件类型支持列表
 */
export const SUPPORTED_FILE_TYPES = ['.txt', '.md', '.markdown']

/**
 * 图片类型列表
 */
export const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp', 'image/svg+xml']