# API 模块说明

## 目录结构

```
src/api/
├── http.js        # Axios 实例和拦截器配置
├── index.js       # API 接口统一导出
└── README.md      # 说明文档
```

## 使用方法

### 1. 基本请求

```javascript
import http from '@/api/http'

// GET 请求
http.get('/user/info')

// POST 请求
http.post('/user/login', { username, password })

// PUT 请求
http.put('/user/update', { name: 'John' })

// DELETE 请求
http.delete('/user/123')
```

### 2. 使用封装的方法

```javascript
import { get, post, put, del } from '@/api/http'

// GET 请求
get('/user/info', { page: 1, pageSize: 10 })

// POST 请求
post('/user/login', { username, password })

// PUT 请求
put('/user/update', { name: 'John' })

// DELETE 请求
del('/user/123')
```

### 3. 使用 API 模块

```javascript
import { userApi, noteApi } from '@/api'

// 获取用户信息
const userInfo = await userApi.getUserInfo(123)

// 创建笔记
const note = await noteApi.createNote({ title, content })
```

## 功能特性

### 请求拦截器

- 自动添加 Authorization token
- 统一配置请求头

### 响应拦截器

- 统一处理业务状态码
- 自动处理常见 HTTP 状态码（401, 403, 404, 500）
- 统一错误提示

### 错误处理

- 401: 自动清除 token 并跳转登录页
- 403: 提示无权限
- 404: 提示资源不存在
- 500: 提示服务器错误
- 网络错误: 提示网络连接问题

## 环境变量

在 `.env` 文件中配置：

```bash
# API 基础路径
VITE_API_BASE_URL=https://api.example.com
```

## 扩展 API

在 `index.js` 中添加新的 API 模块：

```javascript
export const customApi = {
  getList: (params) => http.get('/custom/list', { params }),
  create: (data) => http.post('/custom', data)
}
```