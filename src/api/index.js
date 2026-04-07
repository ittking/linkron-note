/**
 * API 模块统一导出
 * 用于管理所有 API 接口
 */

import http from "./http";
import * as auth from "./auth";

// 导出默认配置
export default {
  http,
};

// 导出各模块
export { auth };
