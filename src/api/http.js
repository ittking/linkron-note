import axios from 'axios'

// 创建 axios 实例
const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器
http.interceptors.request.use(
  config => {
    // 在发送请求之前做些什么
    // 可以在这里添加 token 等认证信息
    const token = localStorage.getItem('token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  error => {
    // 对请求错误做些什么
    console.error('Request error:', error)
    return Promise.reject(error)
  }
)

// 响应拦截器
http.interceptors.response.use(
  response => {
    // 对响应数据做点什么
    const res = response.data

    // 根据后端返回的状态码进行处理
    if (response.status === 200) {
      // 如果返回的数据结构中包含 code 字段，进行统一处理
      if (res.code !== undefined) {
        // 假设 code: 0 表示成功，其他表示失败
        if (res.code === 0) {
          return res.data
        } else {
          // 业务错误，返回完整的错误信息给调用方处理
          return Promise.reject({
            code: res.code,
            message: res.message || '请求失败',
            data: res.data
          })
        }
      }
      // 如果没有 code 字段，直接返回数据
      return res
    }
    return response
  },
  error => {
    // 对响应错误做点什么
    console.error('Response error:', error)

    // 构造错误对象，返回给调用方处理
    const errorInfo = {
      message: '',
      code: null,
      status: null,
      data: null
    }

    if (error.response) {
      // 服务器返回了错误状态码
      const { status, data } = error.response
      errorInfo.status = status
      errorInfo.data = data

      switch (status) {
        case 401:
          errorInfo.message = '登录已过期，请重新登录'
          // 未授权，清除 token
          localStorage.removeItem('token')
          break
        case 403:
          errorInfo.message = '没有权限访问'
          break
        case 404:
          errorInfo.message = '请求的资源不存在'
          break
        case 500:
          errorInfo.message = '服务器错误'
          break
        default:
          errorInfo.message = data.message || `请求失败，状态码: ${status}`
      }
    } else if (error.request) {
      // 请求已发出但没有收到响应
      errorInfo.message = '网络错误，请检查网络连接'
    } else {
      // 请求配置出错
      errorInfo.message = error.message || '请求配置错误'
    }

    // 返回错误对象给调用方处理
    return Promise.reject(errorInfo)
  }
)

// 导出常用的请求方法
export const get = (url, params = {}, config = {}) => {
  return http.get(url, { params, ...config })
}

export const post = (url, data = {}, config = {}) => {
  return http.post(url, data, config)
}

export const put = (url, data = {}, config = {}) => {
  return http.put(url, data, config)
}

export const del = (url, config = {}) => {
  return http.delete(url, config)
}

export const patch = (url, data = {}, config = {}) => {
  return http.patch(url, data, config)
}

// 导出 axios 实例
export default http