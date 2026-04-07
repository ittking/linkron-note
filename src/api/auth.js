/**
 * 授权登录 API
 * 用于小程序扫码登录功能
 */
import { get, post } from './http'

// 生成随机授权码（10位小写字母+数字）
export function generateAuthCode() {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  let code = ''
  for (let i = 0; i < 10; i++) {
    code += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return code
}

/**
 * 获取小程序码
 * @param {string} code - 授权码（10位小写字母+数字）
 * @param {string} appKey - 应用 AppKey
 * @returns {Promise} 返回小程序码信息
 */
export function getQRCode(code, appKey) {
  return post('/api/miniauth/qrcode', { code, appKey })
}

/**
 * 轮询验证登录状态
 * @param {string} code - 授权码
 * @returns {Promise} 返回授权状态
 */
export function pollAuthStatus(code) {
  return get(`/api/miniauth/poll`, { code })
}

/**
 * 根据环境变量获取 AppKey
 * @returns {string} 应用 AppKey
 */
export function getAppKey() {
  return import.meta.env.VITE_APP_APP_KEY || ''
}

/**
 * 根据环境变量获取 AppId
 * @returns {string} 应用 AppId
 */
export function getAppId() {
  return import.meta.env.VITE_APP_APP_ID || ''
}
