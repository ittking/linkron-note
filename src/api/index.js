/**
 * API 模块统一导出
 * 用于管理所有 API 接口
 */

import http from './http'

// 示例：用户相关接口
export const userApi = {
  // 获取用户信息
  getUserInfo: (id) => http.get(`/user/${id}`),
  
  // 更新用户信息
  updateUserInfo: (id, data) => http.put(`/user/${id}`, data),
  
  // 删除用户
  deleteUser: (id) => http.delete(`/user/${id}`)
}

// 示例：笔记相关接口
export const noteApi = {
  // 获取笔记列表
  getNoteList: (params) => http.get('/notes', { params }),
  
  // 获取笔记详情
  getNoteDetail: (id) => http.get(`/notes/${id}`),
  
  // 创建笔记
  createNote: (data) => http.post('/notes', data),
  
  // 更新笔记
  updateNote: (id, data) => http.put(`/notes/${id}`, data),
  
  // 删除笔记
  deleteNote: (id) => http.delete(`/notes/${id}`)
}

// 示例：标签相关接口
export const tagApi = {
  // 获取标签列表
  getTagList: (params) => http.get('/tags', { params }),
  
  // 创建标签
  createTag: (data) => http.post('/tags', data),
  
  // 更新标签
  updateTag: (id, data) => http.put(`/tags/${id}`, data),
  
  // 删除标签
  deleteTag: (id) => http.delete(`/tags/${id}`)
}

// 导出默认配置
export default {
  http,
  userApi,
  noteApi,
  tagApi
}