import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, currentMonitor as getCurrentMonitor, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'

/**
 * 窗口控制 Composable
 * 实现窗口放大和缩小功能，支持多显示器环境
 * 根据操作系统类型使用不同的实现方式
 */
export function useWindowControl() {
  const isFullscreen = ref(false)
  const isMaximized = ref(false)
  const currentOS = ref('unknown')
  const previousWindowState = ref({
    width: 0,
    height: 0,
    x: 0,
    y: 0,
    resizable: false
  })
  const previousMaximizedState = ref(null)

  /**
   * 获取当前窗口
   */
  async function getCurrentWebviewWindow() {
    return getCurrentWindow()
  }

  /**
   * 初始化，获取操作系统类型
   */
  async function init() {
    try {
      currentOS.value = await invoke('get_os')
    } catch (error) {
      console.error('获取操作系统类型失败:', error)
    }
  }

  /**
   * 切换窗口全屏状态
   */
  async function toggleFullscreen() {
    const window = await getCurrentWebviewWindow()
    
    if (isFullscreen.value) {
      // 退出全屏
      await exitFullscreen(window)
    } else {
      // 进入全屏
      await enterFullscreen(window)
    }
    
    isFullscreen.value = !isFullscreen.value
  }

  /**
   * 进入全屏模式
   */
  async function enterFullscreen(window) {
    try {
      // macOS 使用手动全屏方式
      if (currentOS.value === 'macos') {
        await enterFullscreenMacOS(window)
      } else {
        // 其他平台使用原生全屏 API
        await enterFullscreenNative(window)
      }
    } catch (error) {
      console.error('进入全屏失败:', error)
      throw error
    }
  }

  /**
   * macOS 专用：手动全屏实现
   * 参考：https://juejin.cn/post/7585006378533453866
   */
  async function enterFullscreenMacOS(window) {
    // 获取当前窗口的 resizable 状态
    const resizable = await window.isResizable()
    const scaleFactor = await window.scaleFactor()

    // 1. 只在未保存状态时保存当前窗口状态
    if (!previousWindowState.value || previousWindowState.value.width === 0) {
      const size = await window.innerSize()
      const position = await window.outerPosition()

      // 将物理像素转换为逻辑像素
      const logicalSize = size.toLogical(scaleFactor)
      const logicalPosition = position.toLogical(scaleFactor)

      previousWindowState.value = {
        width: logicalSize.width,
        height: logicalSize.height,
        x: logicalPosition.x,
        y: logicalPosition.y,
        resizable
      }
    }

    // 2. 获取当前鼠标所在的显示器
    const currentMonitor = await getCurrentMonitor()
    if (!currentMonitor) {
      console.error('无法获取当前显示器信息')
      return
    }

    // 3. 转换坐标：物理像素 -> 逻辑像素
    const { size: monitorSize, position: monitorPosition } = currentMonitor
    const monitorLogicalSize = monitorSize.toLogical(scaleFactor)
    const monitorLogicalPosition = monitorPosition.toLogical(scaleFactor)

    // 4. 先设置窗口为可调整大小
    await window.setResizable(true)

    // 5. 设置窗口大小和位置，铺满当前显示器
    await window.setSize(new LogicalSize(monitorLogicalSize.width, monitorLogicalSize.height))
    await window.setPosition(new LogicalPosition(monitorLogicalPosition.x, monitorLogicalPosition.y))

    // 6. 恢复原来的 resizable 状态
    await window.setResizable(resizable)
  }

  /**
   * 原生全屏实现（Windows、Linux）
   */
  async function enterFullscreenNative(window) {
    // 只在未保存状态时保存当前窗口状态
    if (!previousWindowState.value || previousWindowState.value.width === 0) {
      const size = await window.innerSize()
      const position = await window.outerPosition()
      const resizable = await window.isResizable()
      const scaleFactor = await window.scaleFactor()

      // 将物理像素转换为逻辑像素
      const logicalSize = size.toLogical(scaleFactor)
      const logicalPosition = position.toLogical(scaleFactor)

      previousWindowState.value = {
        width: logicalSize.width,
        height: logicalSize.height,
        x: logicalPosition.x,
        y: logicalPosition.y,
        resizable
      }
    }

    // 使用原生全屏 API
    await window.setFullscreen(true)
  }

  /**
   * 退出全屏模式
   */
  async function exitFullscreen(window) {
    try {
      // macOS 使用手动恢复方式
      if (currentOS.value === 'macos') {
        await exitFullscreenMacOS(window)
      } else {
        // 其他平台使用原生 API
        await exitFullscreenNative(window)
      }
    } catch (error) {
      console.error('退出全屏失败:', error)
      throw error
    }
  }

  /**
   * macOS 专用：手动恢复全屏
   */
  async function exitFullscreenMacOS(window) {
    const { width, height, x, y, resizable } = previousWindowState.value

    // 验证值是否有效
    if (width === 0 || height === 0 || x === undefined || y === undefined) {
      console.error('保存的窗口状态无效')
      return
    }

    // 先设置窗口为可调整大小
    await window.setResizable(true)

    // 恢复窗口大小
    await window.setSize(new LogicalSize(width, height))

    // 恢复窗口位置
    await window.setPosition(new LogicalPosition(x, y))

    // 恢复窗口可调整大小状态
    await window.setResizable(resizable)

    // 清空保存的状态
    previousWindowState.value = null
  }

  /**
   * 原生退出全屏（Windows、Linux）
   */
  async function exitFullscreenNative(window) {
    await window.setFullscreen(false)

    // 恢复之前的窗口状态
    const { width, height, x, y, resizable } = previousWindowState.value

    // 先设置窗口为可调整大小
    await window.setResizable(true)

    await window.setSize(new LogicalSize(width, height))
    await window.setPosition(new LogicalPosition(x, y))

    // 恢复窗口可调整大小状态
    await window.setResizable(resizable)

    // 清空保存的状态
    previousWindowState.value = null
  }

  /**
   * 设置窗口大小
   */
  async function setWindowSize(width, height) {
    const window = await getCurrentWebviewWindow()
    await window.setSize(new LogicalSize(width, height))
  }

  /**
   * 设置窗口位置
   */
  async function setWindowPosition(x, y) {
    const window = await getCurrentWebviewWindow()
    await window.setPosition(new LogicalPosition(x, y))
  }

  /**
   * 获取窗口大小
   */
  async function getWindowSize() {
    const window = await getCurrentWebviewWindow()
    return await window.innerSize()
  }

  /**
   * 获取窗口位置
   */
  async function getWindowPosition() {
    const window = await getCurrentWebviewWindow()
    return await window.outerPosition()
  }

  /**
   * 设置窗口是否可调整大小
   */
  async function setWindowResizable(resizable) {
    const window = await getCurrentWebviewWindow()
    await window.setResizable(resizable)
  }

  /**
   * 窗口居中
   */
  async function centerWindow() {
    const window = await getCurrentWebviewWindow()
    await window.center()
  }

  /**
   * 最小化窗口
   */
  async function minimizeWindow() {
    const window = await getCurrentWebviewWindow()
    await window.minimize()
  }

  /**
   * macOS 专用：手动最大化实现
   * 将窗口铺满当前显示器，位置为显示器左上角
   */
  async function enterMaximizeMacOS(window) {
    // 获取当前窗口的 resizable 状态和缩放因子
    const resizable = await window.isResizable()
    const scaleFactor = await window.scaleFactor()

    // 1. 只在未保存状态时保存当前窗口状态
    if (!previousMaximizedState.value) {
      const size = await window.innerSize()
      const position = await window.outerPosition()

      // 将物理像素转换为逻辑像素
      const logicalSize = size.toLogical(scaleFactor)
      const logicalPosition = position.toLogical(scaleFactor)

      // 确保 position 对象有效
      const posX = logicalPosition?.x ?? 0
      const posY = logicalPosition?.y ?? 0

      previousMaximizedState.value = {
        width: logicalSize.width,
        height: logicalSize.height,
        x: posX,
        y: posY,
        resizable
      }
    }

    // 2. 获取当前窗口所在的显示器
    const currentMonitor = await getCurrentMonitor()
    if (!currentMonitor) {
      console.error('无法获取当前显示器信息')
      return
    }

    // 3. 转换坐标：物理像素 -> 逻辑像素
    const { size: monitorSize, position: monitorPosition } = currentMonitor
    const monitorLogicalSize = monitorSize.toLogical(scaleFactor)
    const monitorLogicalPosition = monitorPosition.toLogical(scaleFactor)

    // 4. 先设置窗口为可调整大小
    await window.setResizable(true)

    // 5. 设置窗口大小和位置，铺满当前显示器
    // 窗口位置应设置为显示器的左上角位置，而不是 (0, 0)
    await window.setSize(new LogicalSize(monitorLogicalSize.width, monitorLogicalSize.height))
    await window.setPosition(new LogicalPosition(monitorLogicalPosition.x, monitorLogicalPosition.y))

    // 6. 恢复原来的 resizable 状态
    await window.setResizable(resizable)
  }

  /**
   * macOS 专用：手动恢复最大化
   */
  async function exitMaximizeMacOS(window) {
    // 检查是否有保存的状态
    if (!previousMaximizedState.value) {
      console.error('没有保存的窗口状态，无法恢复')
      return
    }

    const { width, height, x, y, resizable } = previousMaximizedState.value

    // 验证值是否有效
    if (width === 0 || height === 0 || x === undefined || y === undefined) {
      console.error('保存的窗口状态无效')
      return
    }

    // 先设置窗口为可调整大小
    await window.setResizable(true)

    // 恢复窗口大小
    await window.setSize(new LogicalSize(width, height))

    // 恢复窗口位置
    await window.setPosition(new LogicalPosition(x, y))

    // 恢复窗口可调整大小状态
    await window.setResizable(resizable)

    // 清空保存的状态
    previousMaximizedState.value = null
  }

  /**
   * 最大化窗口
   * macOS: 手动实现最大化（铺满当前显示器，坐标 0,0）
   * Windows: 使用原生 toggleMaximize API
   */
  async function maximizeWindow() {
    const window = await getCurrentWebviewWindow()
    
    if (isMaximized.value) {
      // 退出最大化
      if (currentOS.value === 'macos') {
        await exitMaximizeMacOS(window)
      } else {
        await window.toggleMaximize()
      }
    } else {
      // 进入最大化
      if (currentOS.value === 'macos') {
        await enterMaximizeMacOS(window)
      } else {
        await window.toggleMaximize()
      }
    }
    
    isMaximized.value = !isMaximized.value
  }

  /**
   * 关闭窗口
   */
  async function closeWindow() {
    const window = await getCurrentWebviewWindow()
    await window.close()
  }

  // 初始化时获取操作系统类型
  init()

  return {
    isFullscreen,
    isMaximized,
    currentOS,
    toggleFullscreen,
    enterFullscreen,
    exitFullscreen,
    setWindowSize,
    setWindowPosition,
    getWindowSize,
    getWindowPosition,
    setWindowResizable,
    centerWindow,
    minimizeWindow,
    maximizeWindow,
    closeWindow
  }
}