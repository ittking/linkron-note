import axios from 'axios'

// 创建 axios 实例
const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器
http.interceptors.request.use(
  config => {
    return config
  },
  error => {
    console.error('Request error:', error)
    return Promise.reject(error)
  }
)

// 响应拦截器
http.interceptors.response.use(
  response => {
    const res = response.data

    if (response.status === 200) {
      if (res.code !== undefined) {
        if (res.code === 0) {
          return res.data
        } else {
          return Promise.reject({
            code: res.code,
            message: res.message || '请求失败',
            data: res.data
          })
        }
      }
      return res
    }
    return response
  },
  error => {
    console.error('Response error:', error)

    const errorInfo = {
      message: '',
      code: null,
      status: null,
      data: null
    }

    if (error.response) {
      const { status, data } = error.response
      errorInfo.status = status
      errorInfo.data = data

      switch (status) {
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
      errorInfo.message = '网络错误，请检查网络连接'
    } else {
      errorInfo.message = error.message || '请求配置错误'
    }

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
